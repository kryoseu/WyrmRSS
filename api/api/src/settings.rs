use actix_web::{
    HttpResponse,
    web::{self, Data, Json},
};
use api_utils::{context::WyrmContext, response::XmlResponse};
use database::models::{
    feed::{Feed, FeedInsertForm},
    settings::Settings,
};
use std::collections::HashMap;
use wyrm_rss::{
    opml::{Opml, Outline},
    worker::WorkerCommand,
};
use wyrm_utils::{
    error::{DatabaseError, WyrmError},
    result::WyrmResult,
};

pub async fn get(ctx: Data<WyrmContext>) -> WyrmResult<Json<Settings>> {
    let settings = Settings::get(&ctx.db_pool).await?;
    Ok(Json(settings))
}

/// Imports feeds from a raw OPML request body. Duplicate URLs are skipped;
/// all other errors propagate. Triggers a best-effort poll after import.
pub async fn import(body: web::Bytes, ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    let opml = Opml::from_xml(body.as_ref())?;

    for outline in opml.body.outlines {
        // leaf feed, no tag
        if outline.xml_url.is_some() {
            if let Some(url) = outline.xml_url {
                create_feed(&ctx, outline.title, url, None).await?;
            }
        // folder - children are feeds, folder name becomes the tag
        } else {
            for child in outline.children {
                if let Some(url) = child.xml_url {
                    create_feed(&ctx, child.title, url, Some(outline.text.clone())).await?;
                }
            }
        }
    }

    let (tx, _) = tokio::sync::oneshot::channel();
    let _ = ctx.worker_tx.try_send(WorkerCommand::PollFeeds(tx));

    Ok(HttpResponse::NoContent().finish())
}

/// Exports all feeds as an OPML file. Tagged feeds are grouped under folder outlines.
pub async fn export(ctx: Data<WyrmContext>) -> WyrmResult<XmlResponse> {
    let feeds = Feed::get_all(&ctx.db_pool).await?;

    let mut feeds_by_tag: HashMap<String, Vec<Feed>> = HashMap::new();
    let mut feeds_wo_tag: Vec<Feed> = vec![];

    for feed in feeds {
        if let Some(tag) = &feed.tag {
            feeds_by_tag.entry(tag.clone()).or_default().push(feed);
        } else {
            feeds_wo_tag.push(feed);
        }
    }

    // Converts a feed into a leaf OPML outline for export.
    let feed_into_outline = |feed: Feed| Outline {
        text: feed.title.clone(),
        title: feed.title,
        kind: Some("rss".to_string()),
        xml_url: Some(feed.url),
        ..Default::default()
    };

    let outlines: Vec<Outline> = feeds_by_tag
        .into_iter()
        .map(|(tag, feeds)| Outline {
            text: tag.clone(),
            title: tag,
            children: feeds.into_iter().map(feed_into_outline).collect(),
            ..Default::default()
        })
        .chain(feeds_wo_tag.into_iter().map(feed_into_outline))
        .collect();

    Ok(XmlResponse {
        body: Opml::new(outlines).to_xml()?,
    })
}

/// Creates a feed, silently skipping it if the URL already exists.
async fn create_feed(
    ctx: &WyrmContext,
    title: String,
    url: String,
    tag: Option<String>,
) -> WyrmResult<()> {
    match Feed::create(
        &ctx.db_pool,
        FeedInsertForm {
            title,
            url,
            ttl: 60,
            tag,
            ..Default::default()
        },
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(WyrmError::Database(DatabaseError::UniqueViolation(_))) => Ok(()),
        Err(e) => Err(e),
    }
}
