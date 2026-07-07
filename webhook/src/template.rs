use crate::error::TemplateError;
use database::models::{feed::Feed, post::Post};
use serde_json::{Value, json};

/// Renders a user-supplied JSON template, substituting `${...}` placeholders
/// against the delivery [`context`]. The template is parsed as JSON first, then
/// walked as a value tree (never as raw text), so all output escaping is handled
/// by serde and a malformed template can never inject structure into the payload.
pub(crate) fn custom_payload(
    template: Option<&str>,
    feed: &Feed,
    posts: &[Post],
) -> Result<Value, TemplateError> {
    let template = template.ok_or(TemplateError::Missing)?;
    render_template(template, &context(feed, posts))
}

/// Validates a custom template without delivering it: checks it is valid JSON
/// and that every `${...}` placeholder resolves against the known variable set.
/// Runs the real [`render`] against a representative sample context so this
/// stays in lockstep with delivery-time behaviour.
pub(crate) fn validate_template(template: &str) -> Result<(), TemplateError> {
    render_template(template, &sample_context())?;
    Ok(())
}

/// Parses a template string as JSON and renders it against `ctx`. Shared by
/// delivery ([`custom_payload`]) and save-time [`validate_template`].
fn render_template(template: &str, ctx: &Value) -> Result<Value, TemplateError> {
    let parsed: Value = serde_json::from_str(template)?;
    render(&parsed, ctx)
}

/// A context with the same *shape* (field names and value types) as [`context`],
/// used only to validate templates when no real feed/posts are available.
fn sample_context() -> Value {
    json!({
        "feed": { "title": "", "url": "", "tag": "" },
        "posts": [ { "title": "", "url": "" } ],
        "posts_count": 0,
    })
}

/// The variables a custom template may reference via `${...}`.
fn context(feed: &Feed, posts: &[Post]) -> Value {
    json!({
        "feed": {
            "title": feed.title,
            "url": feed.url,
            "tag": feed.tag,
        },
        // Custom templates target the user's own endpoint, so we send every
        // post (no platform size limit to respect, unlike Discord/Slack).
        "posts": posts.iter().map(|p| json!({
            "title": p.title.as_deref().unwrap_or("(untitled)"),
            "url": p.url.as_deref().unwrap_or(""),
        })).collect::<Vec<_>>(),
        "posts_count": posts.len(),
    })
}

/// Recursively substitutes placeholders in every string node of the template.
fn render(node: &Value, ctx: &Value) -> Result<Value, TemplateError> {
    match node {
        Value::String(s) => render_string(s, ctx),
        Value::Array(items) => items
            .iter()
            .map(|v| render(v, ctx))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(map) => map
            .iter()
            .map(|(k, v)| Ok((k.clone(), render(v, ctx)?)))
            .collect::<Result<serde_json::Map<_, _>, _>>()
            .map(Value::Object),
        other => Ok(other.clone()),
    }
}

/// Resolves a single string node. A string that is *exactly* one `${path}`
/// token becomes the resolved value with its original type (so `"${posts}"`
/// yields an array); a token embedded in surrounding text is interpolated as
/// its scalar string form.
fn render_string(s: &str, ctx: &Value) -> Result<Value, TemplateError> {
    if let Some(inner) = s.strip_prefix("${").and_then(|r| r.strip_suffix('}'))
        && !inner.is_empty()
        && !inner.contains('$')
        && !inner.contains('}')
    {
        return resolve(ctx, inner)
            .cloned()
            .ok_or_else(|| TemplateError::UnknownVar(inner.to_string()));
    }

    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find('}')
            .ok_or_else(|| TemplateError::UnterminatedToken(s.to_string()))?;
        let path = &after[..end];
        match resolve(ctx, path).ok_or_else(|| TemplateError::UnknownVar(path.to_string()))? {
            Value::String(v) => out.push_str(v),
            Value::Number(n) => out.push_str(&n.to_string()),
            Value::Bool(b) => out.push_str(&b.to_string()),
            Value::Null => {}
            _ => return Err(TemplateError::NonScalarInString(path.to_string())),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(Value::String(out))
}

/// Walks a dotted path (e.g. `feed.title`) into the context object.
fn resolve<'a>(ctx: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = ctx;
    for key in path.split('.') {
        cur = cur.get(key)?;
    }
    Some(cur)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use database::newtypes::{FeedId, PostId};

    fn feed() -> Feed {
        Feed {
            id: FeedId(1),
            title: "MyFeed".to_string(),
            url: "https://feed.test".to_string(),
            ttl: 60,
            filters: vec![],
            last_fetched_at: None,
            created_at: Utc::now(),
            tag: None,
            tag_color: None,
        }
    }

    fn post(title: &str, url: &str) -> Post {
        Post {
            id: PostId(1),
            feed_id: FeedId(1),
            title: Some(title.to_string()),
            url: Some(url.to_string()),
            authors: None,
            published_at: Utc::now(),
            updated_at: None,
            description: None,
            content: None,
            bookmarked: false,
            is_read: false,
            is_archived: false,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn lone_token_preserves_value_type() {
        let template = r#"{"items": "${posts}", "n": "${posts_count}"}"#;
        let posts = [post("A", "https://a"), post("B", "https://b")];

        let value = custom_payload(Some(template), &feed(), &posts).unwrap();

        // Whole-value tokens keep their native JSON type, not a stringified form.
        assert_eq!(value["items"].as_array().unwrap().len(), 2);
        assert_eq!(value["items"][0]["title"], "A");
        assert_eq!(value["n"], 2);
    }

    #[test]
    fn embedded_tokens_interpolate_as_scalars() {
        let template = r#"{"msg": "${feed.title}: ${posts_count} new"}"#;
        let posts = [post("A", "https://a")];

        let value = custom_payload(Some(template), &feed(), &posts).unwrap();

        assert_eq!(value["msg"], "MyFeed: 1 new");
    }

    #[test]
    fn rejects_unknown_and_non_scalar_tokens() {
        let unknown = custom_payload(Some(r#"{"x": "${feed.nope}"}"#), &feed(), &[]);
        assert!(matches!(unknown, Err(TemplateError::UnknownVar(_))));

        // An array can't be spliced into the middle of a string.
        let non_scalar = custom_payload(Some(r#"{"x": "see ${posts}"}"#), &feed(), &[]);
        assert!(matches!(
            non_scalar,
            Err(TemplateError::NonScalarInString(_))
        ));
    }

    #[test]
    fn validate_checks_json_and_variables() {
        assert!(validate_template(r#"{"t": "${feed.title}"}"#).is_ok());
        assert!(matches!(
            validate_template("{not json"),
            Err(TemplateError::InvalidJson(_))
        ));
        assert!(matches!(
            validate_template(r#"{"t": "${foo}"}"#),
            Err(TemplateError::UnknownVar(_))
        ));
    }
}
