//! Webhook payload construction and validation.
//!
//! Given a [`Webhook`] plus the feed and its new posts, it
//! builds the JSON body to POST — the built-in Discord/Slack shapes, or a
//! rendered custom template. Actual delivery lives in the `rss` worker crate,
//! which owns the HTTP client.

mod error;
mod formats;
mod template;

use database::models::{
    feed::Feed,
    post::Post,
    webhook::{Webhook, WebhookKind},
};
pub use error::TemplateError;
use serde_json::Value;

/// Builds the JSON payload for a webhook delivery, dispatching on its kind.
/// `folder` is the feed's resolved folder name (`None` = standalone feed).
pub fn get_payload(
    webhook: &Webhook,
    feed: &Feed,
    folder: Option<&str>,
    new_posts: &[Post],
) -> Result<Value, TemplateError> {
    match webhook.kind {
        WebhookKind::Discord => Ok(formats::discord_payload(&feed.title, new_posts)),
        WebhookKind::Slack => Ok(formats::slack_payload(&feed.title, new_posts)),
        WebhookKind::Custom => Ok(template::custom_payload(
            webhook.payload_template.as_deref(),
            feed,
            folder,
            new_posts,
        )?),
    }
}

/// Rejects a webhook whose payload can't be built, for use in create/update
/// request handling so users get immediate feedback instead of a silent failure
/// at delivery time. `Custom` webhooks must carry a valid template; the built-in
/// formats ignore any template.
pub fn validate(kind: &WebhookKind, template: Option<&str>) -> Result<(), TemplateError> {
    if matches!(kind, WebhookKind::Custom) {
        let template = template.ok_or(TemplateError::Missing)?;
        template::validate_template(template)?;
    }
    Ok(())
}
