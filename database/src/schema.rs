// @generated automatically by Diesel CLI.

diesel::table! {
    feeds (id) {
        id -> Int4,
        title -> Text,
        url -> Text,
        ttl -> Int4,
        tag -> Nullable<Text>,
        tag_color -> Nullable<Text>,
        url_filter -> Array<Nullable<Text>>,
        last_fetched_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
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
        is_favorite -> Bool,
        is_read -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(posts -> feeds (feed_id));

diesel::allow_tables_to_appear_in_same_query!(feeds, posts, users,);
