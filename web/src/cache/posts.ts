/**
 * Post cache: we use TanStack's QueryClient posts fetched under a query key.
 *
 * We store every fetched result under a "query key" (an array like 
 * `["posts", "list", { tag, search }]`). Components read from those
 * cached entries, so when the user changes a post (reads it, favorites it,
 * archives it) we can either edit the cached copy directly so the change shows
 * instantly, or mark the entry stale so React Query refetches it. The refetch
 * isn't manual: the component reading that query (via `useQuery` /
 * `useInfiniteQuery`) is subscribed to it, so when we invalidate the entry that
 * hook re-runs its own fetch. 
 * This file centralizes all of that, so the mutation hooks stay thin and no
 * component has to know how the cache works.
 *
 * The same post can live in several cached entries at once — the all-posts list,
 * a single feed's list, the favorites list, the archived list, plus its own
 * detail entry. A change has to be reflected in all of them.
 *
 * Structure:
 *  - `postKeys`  — the single source of truth for every query key. Each entry
 *                  is either a `*Prefix` (matches *every* variant of a list,
 *                  e.g. all tag/search combos — used for bulk updates) or a
 *                  function that builds the exact key one query is stored under.
 *  - patch helpers (`patchPostInPages`, `patchPostInLists`) — edit the cached
 *                  copies in place so the UI updates without a request fired.
 *  - intent helpers (`setPostArchived`, `applyPostEdit`) — what the mutation
 *                  hooks actually call; each describes one user action and does
 *                  the right mix of patching (for changed fields) and
 *                  invalidating (when a list gains or loses the post).
 *
 * Rule of thumb: patch when a post's *fields* change but it stays in the same
 * lists; invalidate (refetch) when a list's *membership* changes, because a
 * patch can edit existing rows but can't add or remove them.
 */
import type { InfiniteData, QueryClient } from "@tanstack/react-query";
import type { Post } from "../types/Post";
import type { PagedResponse } from "../types/PagedResponse";

export const postKeys = {
  all: ["posts"] as const,

  // Prefixes match every variant of a list (any tag/search) for bulk cache updates;
  // the functions below build the exact key a given query is cached under.
  listPrefix: ["posts", "list"] as const,
  listed: (tag?: string, search?: string) => ["posts", "list", { tag, search }] as const,

  feedPrefix: ["posts", "feed"] as const,
  byFeed: (feedId: number) => ["posts", "feed", feedId] as const,

  favoritesPrefix: ["posts", "favorites"] as const,
  favorites: (search?: string) => ["posts", "favorites", { search }] as const,

  archivedPrefix: ["posts", "archived"] as const,
  archived: (search?: string, tag?: string) => ["posts", "archived", { search, tag }] as const,

  detail: (id: number) => ["posts", id] as const,
  archiveDetail: (id: number) => ["posts", "archive", id] as const,
};

export type PostPages = InfiniteData<PagedResponse<Post[]>> | undefined;

export function patchPostInPages(cached: PostPages, patch: (post: Post) => Post): PostPages {
  if (!cached) return cached;
  return {
    ...cached,
    pages: cached.pages.map((page) => ({
      ...page,
      items: page.items.map(patch),
    })),
  };
}

// Update the post with `postId` wherever it appears in the cached lists
// (all-posts, by-feed, favorites). Pass a full Post to replace it
function patchPostInLists(queryClient: QueryClient, postId: number, changes: Partial<Post>) {
  const apply = (post: Post) => (post.id === postId ? { ...post, ...changes } : post);
  for (const queryKey of [postKeys.listPrefix, postKeys.feedPrefix, postKeys.favoritesPrefix]) {
    queryClient.setQueriesData({ queryKey }, (cached: PostPages) => patchPostInPages(cached, apply));
  }
}

// Reflects an archive/unarchive in the cache: flips is_archived everywhere the
// post is cached, and refreshes the archived list.
export function setPostArchived(queryClient: QueryClient, postId: number, isArchived: boolean) {
  queryClient.setQueryData(postKeys.detail(postId), (cached: Post | undefined) =>
    cached ? { ...cached, is_archived: isArchived } : cached,
  );
  patchPostInLists(queryClient, postId, { is_archived: isArchived });
  queryClient.invalidateQueries({ queryKey: postKeys.archivedPrefix });

  // a cached single archived-post is only valid while it's archived
  if (!isArchived) {
    queryClient.removeQueries({ queryKey: postKeys.archiveDetail(postId) });
  }
}

// Reflects an edited post (title/read/favorite) across every list it appears in
// — all-posts, by-feed, and favorites.
export function applyPostEdit(queryClient: QueryClient, post: Post, favoriteChanged: boolean) {
  queryClient.setQueryData(postKeys.detail(post.id), post);
  patchPostInLists(queryClient, post.id, post);

  // Favoriting adds the post to the favorites list; unfavoriting removes it.
  // Patching only updates rows already there, so refetch to add/drop the post.
  if (favoriteChanged) {
    queryClient.invalidateQueries({ queryKey: postKeys.favoritesPrefix });
  }
}
