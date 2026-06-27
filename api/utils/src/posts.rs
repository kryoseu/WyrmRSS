use database::newtypes::FeedId;
use serde::{Deserialize, Deserializer};

/// `serde_urlencoded` (the backing extractor for `web::Query`) has no support
/// for sequences, so the `exclude` filter on the post listing is sent as a
/// single comma-separated string of feed ids (e.g. `?exclude=3,7,12`). Split
/// and parse it back into `Vec<FeedId>` so handlers keep the typed value.
pub fn de_comma_sep_feed_ids<'de, D>(de: D) -> Result<Option<Vec<FeedId>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(de)?;
    let Some(raw) = raw.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    raw.split(',')
        .map(|part| part.trim().parse().map(FeedId))
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
        .map_err(serde::de::Error::custom)
}
