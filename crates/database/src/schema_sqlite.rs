// @generated automatically by Diesel CLI.

diesel::table! {
    expired_posts (feed_id, url) {
        feed_id -> Integer,
        url -> Text,
        expired_at -> TimestamptzSqlite,
    }
}

diesel::table! {
    feed_icons (feed_id) {
        feed_id -> Integer,
        data -> Nullable<Binary>,
        content_type -> Nullable<Text>,
        checked_at -> TimestamptzSqlite,
    }
}

diesel::table! {
    feed_webhooks (feed_id, webhook_id) {
        feed_id -> Integer,
        webhook_id -> Integer,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::models::feed::DisplayModeMapping;

    feeds (id) {
        id -> Integer,
        title -> Text,
        url -> Text,
        ttl -> Integer,
        filters -> Text,
        last_fetched_at -> Nullable<TimestamptzSqlite>,
        created_at -> TimestamptzSqlite,
        folder_id -> Nullable<Integer>,
        is_paused -> Bool,
        display_mode -> DisplayModeMapping,
    }
}

diesel::table! {
    folders (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    post_archive (id) {
        id -> Integer,
        title -> Nullable<Text>,
        url -> Nullable<Text>,
        authors -> Nullable<Text>,
        published_at -> TimestamptzSqlite,
        description -> Nullable<Text>,
        content -> Nullable<Text>,
        archived_at -> TimestamptzSqlite,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        feed_id -> Integer,
        title -> Nullable<Text>,
        url -> Nullable<Text>,
        authors -> Nullable<Text>,
        published_at -> TimestamptzSqlite,
        updated_at -> Nullable<TimestamptzSqlite>,
        description -> Nullable<Text>,
        content -> Nullable<Text>,
        bookmarked -> Bool,
        is_read -> Bool,
        is_archived -> Bool,
        created_at -> TimestamptzSqlite,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::models::settings::ReadModeMapping;

    settings (is_singleton) {
        is_singleton -> Bool,
        page_size -> Integer,
        feed_poll_interval_secs -> Integer,
        http_timeout -> Integer,
        http_connect_timeout -> Integer,
        http_retries -> Integer,
        http_user_agent -> Nullable<Text>,
        read_mode -> ReadModeMapping,
        expire_read_after_days -> Nullable<Integer>,
        expire_unread_after_days -> Nullable<Integer>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::models::webhook::WebhookKindMapping;

    webhooks (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        kind -> WebhookKindMapping,
        payload_template -> Nullable<Text>,
        created_at -> TimestamptzSqlite,
    }
}

diesel::joinable!(expired_posts -> feeds (feed_id));
diesel::joinable!(feed_icons -> feeds (feed_id));
diesel::joinable!(feed_webhooks -> feeds (feed_id));
diesel::joinable!(feed_webhooks -> webhooks (webhook_id));
diesel::joinable!(feeds -> folders (folder_id));
diesel::joinable!(posts -> feeds (feed_id));

diesel::allow_tables_to_appear_in_same_query!(
    expired_posts,
    feed_icons,
    feed_webhooks,
    feeds,
    folders,
    post_archive,
    posts,
    settings,
    webhooks,
);
