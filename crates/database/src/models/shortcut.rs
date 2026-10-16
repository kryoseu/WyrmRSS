use crate::{DatabasePool, schema::shortcut_overrides};
use diesel::{Selectable, deserialize::Queryable, prelude::*, upsert::excluded};
use diesel_async::RunQueryDsl;
use serde::{
    Deserialize,
    Serialize,
    de::{IntoDeserializer, value::Error as ParseError},
};
use std::str::FromStr;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

/// A rebindable keyboard action. Stored as its snake_case name in
/// `shortcut_overrides.shortcut_action`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub enum ShortcutAction {
    /// Open the next post in the list as displayed.
    NextPost,
    /// Open the previous post in the list as displayed.
    PrevPost,
    /// Open the next post from the same feed as the current one.
    NextPostInFeed,
    /// Open the previous post from the same feed as the current one.
    PrevPostInFeed,
    /// Close the post reader.
    CloseReader,
    /// Archive or unarchive the current post.
    ToggleArchive,
    /// Add or remove the current post from read later.
    ToggleReadLater,
    /// Mark the current post read or unread.
    ToggleRead,
    /// Open the post's original article in a new tab.
    OpenOriginal,
    /// Show the keyboard shortcuts help.
    ShowHelp,
    /// Focus the search box.
    FocusSearch,
    /// Navigate to Home.
    NavigateHome,
    /// Navigate to Read Later.
    NavigateReadLater,
    /// Navigate to Archive.
    NavigateArchive,
    /// Navigate to Radar.
    NavigateRadar,
}

// Parses the stored action name, reusing serde's snake_case names.
impl FromStr for ShortcutAction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

/// The key bound to every action: defaults with the user's overrides applied.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub struct Shortcuts {
    /// Open the next post in the list as displayed.
    pub next_post: String,
    /// Open the previous post in the list as displayed.
    pub prev_post: String,
    /// Open the next post from the same feed as the current one.
    pub next_post_in_feed: String,
    /// Open the previous post from the same feed as the current one.
    pub prev_post_in_feed: String,
    /// Close the post reader.
    pub close_reader: String,
    /// Archive or unarchive the current post.
    pub toggle_archive: String,
    /// Add or remove the current post from read later.
    pub toggle_read_later: String,
    /// Mark the current post read or unread.
    pub toggle_read: String,
    /// Open the post's original article in a new tab.
    pub open_original: String,
    /// Show the keyboard shortcuts help.
    pub show_help: String,
    /// Focus the search box.
    pub focus_search: String,
    /// Navigate to Home.
    pub navigate_home: String,
    /// Navigate to Read Later.
    pub navigate_read_later: String,
    /// Navigate to Archive.
    pub navigate_archive: String,
    /// Navigate to Radar.
    pub navigate_radar: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            next_post: "j".into(),
            prev_post: "k".into(),
            next_post_in_feed: "shift+j".into(),
            prev_post_in_feed: "shift+k".into(),
            close_reader: "Escape".into(),
            toggle_archive: "a".into(),
            toggle_read_later: "shift+r".into(),
            toggle_read: "r".into(),
            open_original: "v".into(),
            show_help: "?".into(),
            focus_search: "/".into(),
            navigate_home: "1".into(),
            navigate_read_later: "2".into(),
            navigate_archive: "3".into(),
            navigate_radar: "4".into(),
        }
    }
}

impl Shortcuts {
    /// Defaults with the user's overrides applied.
    pub fn from_overrides(overrides: Vec<ShortcutOverride>) -> Self {
        let mut shortcuts = Self::default();
        for o in overrides {
            // Unknown action (e.g. removed in a later version): skip it.
            let Ok(action) = o.shortcut_action.parse::<ShortcutAction>() else {
                continue;
            };
            *shortcuts.key_mut(action) = o.key;
        }
        shortcuts
    }

    fn key_mut(&mut self, action: ShortcutAction) -> &mut String {
        match action {
            ShortcutAction::NextPost => &mut self.next_post,
            ShortcutAction::PrevPost => &mut self.prev_post,
            ShortcutAction::NextPostInFeed => &mut self.next_post_in_feed,
            ShortcutAction::PrevPostInFeed => &mut self.prev_post_in_feed,
            ShortcutAction::CloseReader => &mut self.close_reader,
            ShortcutAction::ToggleArchive => &mut self.toggle_archive,
            ShortcutAction::ToggleReadLater => &mut self.toggle_read_later,
            ShortcutAction::ToggleRead => &mut self.toggle_read,
            ShortcutAction::OpenOriginal => &mut self.open_original,
            ShortcutAction::ShowHelp => &mut self.show_help,
            ShortcutAction::FocusSearch => &mut self.focus_search,
            ShortcutAction::NavigateHome => &mut self.navigate_home,
            ShortcutAction::NavigateReadLater => &mut self.navigate_read_later,
            ShortcutAction::NavigateArchive => &mut self.navigate_archive,
            ShortcutAction::NavigateRadar => &mut self.navigate_radar,
        }
    }

    fn bindings(&self) -> Vec<(ShortcutAction, &str)> {
        vec![
            (ShortcutAction::NextPost, &self.next_post),
            (ShortcutAction::PrevPost, &self.prev_post),
            (ShortcutAction::NextPostInFeed, &self.next_post_in_feed),
            (ShortcutAction::PrevPostInFeed, &self.prev_post_in_feed),
            (ShortcutAction::CloseReader, &self.close_reader),
            (ShortcutAction::ToggleArchive, &self.toggle_archive),
            (ShortcutAction::ToggleReadLater, &self.toggle_read_later),
            (ShortcutAction::ToggleRead, &self.toggle_read),
            (ShortcutAction::OpenOriginal, &self.open_original),
            (ShortcutAction::ShowHelp, &self.show_help),
            (ShortcutAction::FocusSearch, &self.focus_search),
            (ShortcutAction::NavigateHome, &self.navigate_home),
            (ShortcutAction::NavigateReadLater, &self.navigate_read_later),
            (ShortcutAction::NavigateArchive, &self.navigate_archive),
            (ShortcutAction::NavigateRadar, &self.navigate_radar),
        ]
    }

    pub fn action_for_key(&self, key: &str) -> Option<ShortcutAction> {
        for (action, bound_key) in self.bindings() {
            if bound_key == key {
                return Some(action);
            }
        }
        None
    }
}

#[derive(Default, Insertable)]
#[diesel(table_name = crate::schema::shortcut_overrides)]
#[diesel(check_for_backend(crate::Backend))]
pub struct ShortcutOverrideInsertForm {
    pub shortcut_action: String,
    pub key: String,
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::shortcut_overrides)]
#[diesel(check_for_backend(crate::Backend))]
#[derive(ts_rs::TS)]
#[ts(export)]
pub struct ShortcutOverride {
    pub shortcut_action: String,
    pub key: String,
}

impl ShortcutOverride {
    pub async fn get_all(pool: &DatabasePool) -> WyrmResult<Vec<Self>> {
        let mut conn = pool.get().await?;
        shortcut_overrides::table
            .select(ShortcutOverride::as_select())
            .load(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn create(pool: &DatabasePool, form: ShortcutOverrideInsertForm) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::insert_into(shortcut_overrides::table)
            .values(&form)
            .on_conflict(shortcut_overrides::shortcut_action)
            .do_update()
            .set(shortcut_overrides::key.eq(excluded(shortcut_overrides::key)))
            .get_result::<Self>(&mut conn)
            .await
            .map_err(WyrmError::from)?;
        Ok(())
    }

    pub async fn delete(pool: &DatabasePool, action: &str) -> WyrmResult<()> {
        let mut conn = pool.get().await?;
        diesel::delete(shortcut_overrides::table.find(action))
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shortcut_override(action: &str, key: &str) -> ShortcutOverride {
        ShortcutOverride {
            shortcut_action: action.into(),
            key: key.into(),
        }
    }

    #[test]
    fn from_overrides_applies_known_and_skips_unknown() {
        let shortcuts = Shortcuts::from_overrides(vec![
            shortcut_override("next_post", "n"),
            shortcut_override("removed_action", "x"),
        ]);

        assert_eq!(shortcuts.next_post, "n");
        // Everything else keeps its default.
        assert_eq!(shortcuts.prev_post, Shortcuts::default().prev_post);
    }

    #[cfg(feature = "postgres")]
    #[tokio::test]
    async fn create_upserts_and_delete_is_idempotent() {
        let pool = crate::setup_test_db().await;
        let key_of = |overrides: Vec<ShortcutOverride>| {
            overrides
                .into_iter()
                .find(|o| o.shortcut_action == "focus_search")
                .map(|o| o.key)
        };
        let form = |key: &str| ShortcutOverrideInsertForm {
            shortcut_action: "focus_search".into(),
            key: key.into(),
        };

        ShortcutOverride::create(&pool, form("ctrl+alt+f11"))
            .await
            .expect("create should succeed");
        // A second create replaces the key instead of erroring on the primary key.
        ShortcutOverride::create(&pool, form("ctrl+alt+f12"))
            .await
            .expect("create should upsert");
        let after_upsert = key_of(ShortcutOverride::get_all(&pool).await.unwrap());

        ShortcutOverride::delete(&pool, "focus_search")
            .await
            .expect("delete should succeed");
        // Resetting an action with no override is a no-op, not a 404.
        ShortcutOverride::delete(&pool, "focus_search")
            .await
            .expect("second delete should succeed");
        let after_delete = key_of(ShortcutOverride::get_all(&pool).await.unwrap());

        assert_eq!(after_upsert.as_deref(), Some("ctrl+alt+f12"));
        assert_eq!(after_delete, None);
    }
}
