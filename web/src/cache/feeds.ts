/**
 * Feed cache: query keys and cache writers for the feeds list, mirroring
 * `cache/posts.ts`. The feeds query is `staleTime: Infinity`, so between
 * refetches (poll, create/edit/delete) these helpers are what keep the cached
 * list current.
 */
import type { QueryClient } from "@tanstack/react-query";
import type { FeedView } from "../types/FeedView";
import type { FeedId } from "../types/FeedId";

export const feedKeys = {
  all: ["feeds"] as const,
};

export const feedIconKey = (feedId?: FeedId) => ["feed-icon", feedId] as const;

// Overwrite one feed's unread_count with the authoritative value a mutation
// response carried back, so the sidebar badge moves without refetching feeds.
export function setFeedUnreadCount(
  queryClient: QueryClient,
  feedId: FeedId,
  unreadCount: number,
) {
  queryClient.setQueryData(feedKeys.all, (cached: FeedView[] | undefined) =>
    cached?.map((f) => (f.id === feedId ? { ...f, unread_count: unreadCount } : f)),
  );
}

// Overwrite one feed's is_paused state, so the sidebar feed can show the new state without fetching data.
export function setFeedPauseState(
  queryClient: QueryClient,
  feedId: FeedId,
  isPaused: boolean,
) {
  queryClient.setQueryData(feedKeys.all, (cached: FeedView[] | undefined) =>
    cached?.map((f) => (f.id === feedId ? { ...f, is_paused: isPaused } : f)),
  );
}

// A changed feed URL is the only edit that can change the resolved icon (the
// backend deletes and re-resolves it on URL change); everything else leaves
// it untouched. `useFeedIcon`'s query never expires on its own (see its
// comment), so a URL edit needs to evict it here or the old feed's icon
// blob would stick around for the rest of the session.
export function evictFeedIconIfUrlChanged(
  queryClient: QueryClient,
  feedId: FeedId,
  newUrl: string | null,
) {
  const previousUrl = queryClient
    .getQueryData<FeedView[]>(feedKeys.all)
    ?.find((f) => f.id === feedId)?.url;
  if (newUrl === null || newUrl === previousUrl) return;

  const key = feedIconKey(feedId);
  const oldIconUrl = queryClient.getQueryData<string>(key);
  if (oldIconUrl) URL.revokeObjectURL(oldIconUrl);
  queryClient.removeQueries({ queryKey: key });
}
