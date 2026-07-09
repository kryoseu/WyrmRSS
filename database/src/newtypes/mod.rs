use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};

#[derive(
    DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
/// The Post ID
pub struct PostId(pub i32);

#[derive(
    DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
/// The Feed ID
pub struct FeedId(pub i32);

#[derive(
    DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
/// The Webhook ID
pub struct WebhookId(pub i32);

#[derive(
    DieselNewType, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
/// The Folder ID
pub struct FolderId(pub i32);
