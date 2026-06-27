use actix_web::{
    HttpResponse,
    web::{Data, Json, Path, Query},
};
use api_utils::context::WyrmContext;
use database::{
    models::{archive::PostArchive, post::Post},
    newtypes::{FeedId, PostId},
    utils::pagination::{PagedResponse, PaginationCursor},
    views::{archive::get_post_archive_insert_form, post::PostQuery},
};
use serde::Deserialize;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct ListPosts {
    pub page: Option<PaginationCursor>,
    pub tag: Option<String>,
    pub search: Option<String>,
    #[serde(default, deserialize_with = "api_utils::posts::de_comma_sep_feed_ids")]
    pub exclude: Option<Vec<FeedId>>,
}

pub async fn get(path: Path<PostId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<Post>> {
    let post_id = path.into_inner();
    let post = Post::get(&ctx.db_pool, post_id).await?;
    Ok(Json(post))
}

pub async fn archive(path: Path<PostId>, ctx: Data<WyrmContext>) -> WyrmResult<Json<PostArchive>> {
    let post_id = path.into_inner();
    let form = get_post_archive_insert_form(&ctx.db_pool, post_id).await?;
    let archived = PostArchive::create(&ctx.db_pool, form).await?;
    Ok(Json(archived))
}

pub async fn unarchive(path: Path<PostId>, ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    let post_id = path.into_inner();
    PostArchive::delete(&ctx.db_pool, post_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list(
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page_size = ctx.runtime_settings.read()?.page_size;
    let page = PostQuery {
        cursor: query.page,
        tag: query.tag,
        search: query.search,
        exclude: query.exclude,
        ..Default::default()
    }
    .list(&ctx.db_pool, page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_by_feed(
    path: Path<i32>,
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page_size = ctx.runtime_settings.read()?.page_size;
    let page = PostQuery {
        feed_id: Some(path.into_inner()),
        cursor: query.page,
        search: query.search,
        ..Default::default()
    }
    .list(&ctx.db_pool, page_size)
    .await?;
    Ok(Json(page))
}

pub async fn list_favorites(
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page_size = ctx.runtime_settings.read()?.page_size;
    let page = PostQuery {
        fav_only: true,
        cursor: query.page,
        search: query.search,
        ..Default::default()
    }
    .list(&ctx.db_pool, page_size)
    .await?;
    Ok(Json(page))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web::Query;

    /// Parse a query string through the same extractor production uses.
    fn parse(qs: &str) -> ListPosts {
        Query::<ListPosts>::from_query(qs)
            .expect("query should parse")
            .into_inner()
    }

    #[test]
    fn exclude_parses_comma_separated_feed_ids() {
        let q = parse("exclude=3,7,12");
        assert_eq!(q.exclude, Some(vec![FeedId(3), FeedId(7), FeedId(12)]));
    }

    #[test]
    fn exclude_single_id() {
        assert_eq!(parse("exclude=5").exclude, Some(vec![FeedId(5)]));
    }

    #[test]
    fn exclude_absent_is_none() {
        assert_eq!(parse("tag=news").exclude, None);
    }

    #[test]
    fn exclude_empty_string_is_none() {
        assert_eq!(parse("exclude=").exclude, None);
    }

    #[test]
    fn exclude_tolerates_surrounding_whitespace() {
        // `%20` decodes to a space around the middle id.
        assert_eq!(
            parse("exclude=3,%207%20,12").exclude,
            Some(vec![FeedId(3), FeedId(7), FeedId(12)])
        );
    }

    #[test]
    fn exclude_rejects_non_numeric() {
        assert!(Query::<ListPosts>::from_query("exclude=3,abc").is_err());
    }

    #[test]
    fn parses_scalar_fields() {
        let q = parse("tag=news&search=rust");
        assert_eq!(q.tag.as_deref(), Some("news"));
        assert_eq!(q.search.as_deref(), Some("rust"));
        assert_eq!(q.exclude, None);
    }
}
