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
