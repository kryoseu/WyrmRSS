use database::models::post::Post;
use serde::Serialize;
use serde_json::Value;

const MAX_LISTED: usize = 10;

/// Discord webhook body: a `content` summary line plus one embed whose
/// `description` lists the posts. See <https://discord.com/developers/docs/resources/webhook>.
#[derive(Serialize)]
struct DiscordPayload {
    content: String,
    embeds: Vec<DiscordEmbed>,
}

#[derive(Serialize)]
struct DiscordEmbed {
    description: String,
}

/// Slack incoming-webhook body: a single `text` field of mrkdwn.
#[derive(Serialize)]
struct SlackPayload {
    text: String,
}

pub(crate) fn discord_payload(feed_name: &str, posts: &[Post]) -> Value {
    let payload = DiscordPayload {
        content: summary(&escape_discord(feed_name), posts),
        // Escape the title's markdown and wrap the URL in `<…>` so parenthesis or
        // spaces in it can't terminate the `[text](url)` link.
        embeds: vec![DiscordEmbed {
            description: content(posts, |t, u| format!("[{}](<{}>)", escape_discord(t), u)),
        }],
    };
    serde_json::to_value(payload).expect("discord payload serializes")
}

pub(crate) fn slack_payload(feed_name: &str, posts: &[Post]) -> Value {
    let payload = SlackPayload {
        text: format!(
            "{}\n{}",
            summary(&escape_slack(feed_name), posts),
            content(posts, |t, u| format!(
                "<{}|{}>",
                escape_slack(u),
                escape_slack(t)
            )),
        ),
    };
    serde_json::to_value(payload).expect("slack payload serializes")
}

fn summary(feed_name: &str, posts: &[Post]) -> String {
    format!("{} has {} post(s)", feed_name, posts.len())
}

/// Backslash-escapes Discord markdown so a post title or feed name can't break
/// the `[text](url)` link or inject formatting.
fn escape_discord(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(
            c,
            '\\' | '`' | '*' | '_' | '~' | '|' | '[' | ']' | '(' | ')' | '>'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Slack requires `&`, `<`, `>` to be HTML-escaped in all text, including link
/// URLs and labels.
fn escape_slack(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// `link` formats one post into a platform-specific clickable line.
fn content(posts: &[Post], link: impl Fn(&str, &str) -> String) -> String {
    let mut lines: Vec<String> = posts
        .iter()
        .take(MAX_LISTED)
        .map(|p| {
            let title = p.title.as_deref().unwrap_or("(untitled)");
            let url = p.url.as_deref().unwrap_or("");
            link(title, url)
        })
        .collect();
    if posts.len() > MAX_LISTED {
        lines.push(format!("…and {} more", posts.len() - MAX_LISTED));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use database::newtypes::{FeedId, PostId};

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
            is_favorite: false,
            is_read: false,
            is_archived: false,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn discord_escapes_markdown_and_caps_listing() {
        let posts: Vec<Post> = (0..12)
            .map(|i| post(&format!("Title_{i}"), &format!("https://p/{i}")))
            .collect();
        let value = discord_payload("Rust *News*", &posts);

        let content = value["content"].as_str().unwrap();
        assert_eq!(content, r"Rust \*News\* has 12 post(s)");

        let description = value["embeds"][0]["description"].as_str().unwrap();
        // Title markdown escaped, URL wrapped in <…>.
        assert!(description.contains(r"[Title\_0](<https://p/0>)"));
        // Only MAX_LISTED rendered, with an overflow note for the rest.
        assert!(description.contains("…and 2 more"));
    }

    #[test]
    fn slack_html_escapes_links() {
        let value = slack_payload("A & B", &[post("T<x>", "https://p?a=1&b=2")]);
        let text = value["text"].as_str().unwrap();

        assert!(text.contains("A &amp; B has 1 post(s)"));
        assert!(text.contains("<https://p?a=1&amp;b=2|T&lt;x&gt;>"));
    }
}
