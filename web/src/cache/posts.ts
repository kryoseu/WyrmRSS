/**
 * Post cache: we use TanStack's QueryClient posts fetched under a query key.
 *
 * We store every fetched result under a "query key" (an array like
 * `["posts", "list", { feed_id: 3 }]`). Components read from those
 * cached entries, so when the user changes a post (reads it, bookmarks it,
 * archives it) we can either edit the cached copy directly so the change shows
 * instantly, or mark the entry stale so React Query refetches it. The refetch
 * isn't manual: the component reading that query (via `useQuery` /
 * `useInfiniteQuery`) is subscribed to it, so when we invalidate the entry that
 * hook re-runs its own fetch. 
 * This file centralizes all of that, so the mutation hooks stay thin and no
 * component has to know how the cache works.
 *
 * The same post can live in several cached entries at once — any number of
 * lists (all posts, a feed, a folder, bookmarked), the archived list, plus its
 * own by-id entry. A change has to be reflected in all of them.
 *
 * Structure:
 *  - `postKeys`  — the single source of truth for every query key. Each entry
 *                  is either a prefix (matches a whole group, e.g. every list —
 *                  used for bulk updates) or a function that builds the exact
 *                  key one query is stored under.
 *  - patch helpers (`patchPostInPages`, `patchPostInLists`, `patchPost`) — edit
 *                  the cached copies in place so the UI updates without a refetch.
 *  - actions (`setPostArchived`, `setPostRead`, `setPostBookmarked`) — what the
 *                  mutation hooks call; each is one user action and does the right
 *                  mix of patching (changed fields) and invalidating (when a list
 *                  gains or loses the post).
 *
 * Rule of thumb: patch when a post's *fields* change but it stays in the same
 * lists; invalidate (refetch) when a list's *membership* changes, because a
 * patch can edit existing rows but can't add or remove them.
 */
import type { InfiniteData, QueryClient } from "@tanstack/react-query";
import type { ListPosts } from "../types/ListPosts";
import type { Post } from "../types/Post";
import type { PagedResponse } from "../types/PagedResponse";

// ─── Cache keys ───
// Every query key in one place. A prefix is a partial key matching a whole
// group at once — e.g. listPrefix = ["posts","list"] hits every post list.
// A builder makes the one exact key a query uses — a list's key holds the
// request params themselves, so the key always matches what was fetched:
// list({ feed_id: 3 }) = ["posts","list",{ feed_id: 3 }].
export const postKeys = {
  all: ["posts"] as const,

  listPrefix: ["posts", "list"] as const,
  list: (params: ListPosts) => [...postKeys.listPrefix, params] as const,

  bookmarkedPrefix: ["posts", "list", "bookmarked"] as const,
  bookmarked: (search?: string) => [...postKeys.bookmarkedPrefix, { search }] as const,

  archivedPrefix: ["posts", "archived"] as const,
  archived: (search?: string) => [...postKeys.archivedPrefix, { search }] as const,

  byId: (id: number) => [...postKeys.all, "byId", id] as const,

  archiveById: (id: number) => [...postKeys.all, "archive", id] as const,
};

// ─── Cache writers (internal functions) ───
type PostPages = InfiniteData<PagedResponse<Post[]>> | undefined;

function patchPostInPages(cached: PostPages, patch: (post: Post) => Post): PostPages {
  if (!cached) return cached;
  return {
    ...cached,
    pages: cached.pages.map((page) => ({
      ...page,
      items: page.items.map(patch),
    })),
  };
}

// Update the post with `postId` wherever it appears in the cached lists.
// Pass a full Post to replace it
function patchPostInLists(queryClient: QueryClient, postId: number, changes: Partial<Post>) {
  const apply = (post: Post) => (post.id === postId ? { ...post, ...changes } : post);
  queryClient.setQueriesData({ queryKey: postKeys.listPrefix }, (cached: PostPages) =>
    patchPostInPages(cached, apply),
  );
}

// Apply field changes to a post wherever it's cached — its by-id entry and every list.
function patchPost(queryClient: QueryClient, postId: number, changes: Partial<Post>) {
  queryClient.setQueryData(postKeys.byId(postId), (cached: Post | undefined) =>
    cached ? { ...cached, ...changes } : cached,
  );
  patchPostInLists(queryClient, postId, changes);
}

// ─── Actions (called by the mutation hooks) ───

// Archiving/unarchiving also changes the archived list's membership.
export function setPostArchived(queryClient: QueryClient, postId: number, isArchived: boolean) {
  patchPost(queryClient, postId, { is_archived: isArchived });
  queryClient.invalidateQueries({ queryKey: postKeys.archivedPrefix });

  // a cached single archived-post is only valid while it's archived
  if (!isArchived) {
    queryClient.removeQueries({ queryKey: postKeys.archiveById(postId) });
  }
}

// Read is a field-only change — patch in place; nothing joins or leaves a list.
export function setPostRead(queryClient: QueryClient, postId: number, isRead: boolean) {
  patchPost(queryClient, postId, { is_read: isRead });
}

// Bookmarking also changes the bookmarked list's membership, so refetch it.
export function setPostBookmarked(queryClient: QueryClient, postId: number, bookmarked: boolean) {
  patchPost(queryClient, postId, { bookmarked });
  queryClient.invalidateQueries({ queryKey: postKeys.bookmarkedPrefix });
}
