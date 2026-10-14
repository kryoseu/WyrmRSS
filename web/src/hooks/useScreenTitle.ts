import { useLocation, useParams } from "react-router";
import { useFeeds } from "./useFeeds";
import type { FeedMeta } from "../components/PostItem";

const SECTION_TITLES: [prefix: string, title: string][] = [
  ["/read-later", "Read Later"],
  ["/archive", "Archive"],
  ["/radar", "Radar"],
  ["/feeds", "Feeds"],
  ["/settings", "Settings"],
];

/**
 * Title for the phone top bar: the open feed's name, or the section's. The
 * subtitle is the unread count where read state is tracked.
 */
export function useScreenTitle(): { title: string; subtitle?: string; feed?: FeedMeta } {
  const { pathname } = useLocation();
  const { feedId } = useParams();
  const { data: feeds = [] } = useFeeds();

  if (feedId) {
    const feed = feeds.find((f) => String(f.id) === feedId);
    // Radar feeds don't track read state.
    const tracked = feed && feed.display_mode !== "radar";
    return {
      title: feed?.title ?? "Feed",
      subtitle: tracked ? unread(feed.unread_count) : undefined,
      feed: feed && { id: feed.id, name: feed.title, hasIcon: feed.has_icon },
    };
  }

  const section = SECTION_TITLES.find(([prefix]) => pathname.startsWith(prefix));
  if (section) return { title: section[1] };

  // Home only shows river feeds' posts, so only those count.
  const riverUnread = feeds
    .filter((f) => f.display_mode === "river")
    .reduce((sum, f) => sum + f.unread_count, 0);
  return { title: "Home", subtitle: unread(riverUnread) };
}

function unread(count: number): string {
  return count === 0 ? "All caught up" : `${count > 999 ? "999+" : count} unread`;
}
