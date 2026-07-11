use actix_web::{
    HttpResponse,
    web::{self, Data, Json},
};
use api_utils::{context::WyrmContext, response::XmlResponse};
use database::{
    models::{
        feed::{Feed, FeedInsertForm},
        folder::Folder,
        settings::Settings,
    },
    newtypes::FolderId,
};
use std::{collections::HashMap, time::Duration};
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
/// all other errors propagate. Triggers a best-effort poll after import and
/// waits for it (bounded by one http timeout) before responding.
pub async fn import(body: web::Bytes, ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    let opml = Opml::from_xml(body.as_ref())?;

    for outline in opml.body.outlines {
        // standalone leaf feed
        if outline.xml_url.is_some() {
            if let Some(url) = outline.xml_url {
                create_feed(&ctx, outline.title, url, None).await?;
            }
        // folder - children are feeds, each resolves/creates the folder by name
        } else {
            for child in outline.children {
                if let Some(url) = child.xml_url {
                    create_feed(&ctx, child.title, url, Some(outline.text.as_str())).await?;
                }
            }
        }
    }

    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = ctx.worker_tx.try_send(WorkerCommand::PollFeeds(tx));

    // The import is complete and a poll is on the way: give it up to one
    // http timeout to land so the frontend's refetch right after this
    // response already sees the imported posts and their icons. A huge
    // import may take longer; the frontend polls again as a safety net.
    let http_timeout = ctx.runtime_settings.read()?.http_timeout;
    let _ = tokio::time::timeout(Duration::from_secs(http_timeout as u64), rx).await;

    Ok(HttpResponse::NoContent().finish())
}

/// Exports all feeds as an OPML file. Feeds in a folder are grouped under a
/// folder outline; standalone feeds are top-level outlines.
pub async fn export(ctx: Data<WyrmContext>) -> WyrmResult<XmlResponse> {
    let feeds = Feed::get_all(&ctx.db_pool).await?;
    let folders = Folder::get_all(&ctx.db_pool).await?;
    let folder_names: HashMap<FolderId, String> =
        folders.into_iter().map(|f| (f.id, f.name)).collect();

    let mut feeds_by_folder: HashMap<String, Vec<Feed>> = HashMap::new();
    let mut standalone: Vec<Feed> = vec![];

    for feed in feeds {
        match feed.folder_id.and_then(|id| folder_names.get(&id)) {
            Some(name) => feeds_by_folder.entry(name.clone()).or_default().push(feed),
            None => standalone.push(feed),
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

    let outlines: Vec<Outline> = feeds_by_folder
        .into_iter()
        .map(|(folder, feeds)| Outline {
            text: folder.clone(),
            title: folder,
            children: feeds.into_iter().map(feed_into_outline).collect(),
            ..Default::default()
        })
        .chain(standalone.into_iter().map(feed_into_outline))
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
    folder: Option<&str>,
) -> WyrmResult<()> {
    match Feed::create(
        &ctx.db_pool,
        FeedInsertForm {
            title,
            url,
            ttl: 60,
            folder: folder.map(String::from),
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
