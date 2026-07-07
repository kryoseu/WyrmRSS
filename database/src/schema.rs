// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "read_mode"))]
    pub struct ReadMode;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "webhook_kind"))]
    pub struct WebhookKind;
}

diesel::table! {
    feed_webhooks (feed_id, webhook_id) {
        feed_id -> Int4,
        webhook_id -> Int4,
    }
}

diesel::table! {
    feeds (id) {
        id -> Int4,
        title -> Text,
        url -> Text,
        ttl -> Int4,
        url_filter -> Array<Nullable<Text>>,
        last_fetched_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        tag -> Nullable<Text>,
        tag_color -> Nullable<Text>,
    }
}

diesel::table! {
    post_archive (id) {
        id -> Int4,
        title -> Nullable<Text>,
        url -> Nullable<Text>,
        authors -> Nullable<Text>,
        published_at -> Timestamptz,
        description -> Nullable<Text>,
        content -> Nullable<Text>,
        tag -> Nullable<Text>,
        tag_color -> Nullable<Text>,
        archived_at -> Timestamptz,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        feed_id -> Int4,
        title -> Nullable<Text>,
        url -> Nullable<Text>,
        authors -> Nullable<Text>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        description -> Nullable<Text>,
        content -> Nullable<Text>,
        bookmarked -> Bool,
        is_read -> Bool,
        is_archived -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ReadMode;

    settings (is_singleton) {
        is_singleton -> Bool,
        page_size -> Int4,
        feed_poll_interval_secs -> Int4,
        http_timeout -> Int4,
        http_connect_timeout -> Int4,
        http_retries -> Int4,
        http_user_agent -> Nullable<Text>,
        read_mode -> ReadMode,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WebhookKind;

    webhooks (id) {
        id -> Int4,
        name -> Text,
        url -> Text,
        kind -> WebhookKind,
        payload_template -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(feed_webhooks -> feeds (feed_id));
diesel::joinable!(feed_webhooks -> webhooks (webhook_id));
diesel::joinable!(posts -> feeds (feed_id));

diesel::allow_tables_to_appear_in_same_query!(
    feed_webhooks,
    feeds,
    post_archive,
    posts,
    settings,
    webhooks,
);
