/**
 * Mutations for changing a post (archive, unarchive, edit read/favorite).
 *
 * Each mutation does two things: `mutationFn` sends the change to the server,
 * and `onSuccess` reconciles the local cache so the UI reflects it without
 * waiting for a fresh load. The list hooks (`useFavoritePosts`, etc.) only
 * fetch when they mount or when their cache entry is invalidated — they don't
 * know a post changed — so this `onSuccess` step is the only thing that keeps
 * the already-rendered lists in sync after an action.
 *
 * The actual cache reconciliation lives in `cache/posts.ts` (`setPostArchived`,
 * `setPostRead`, `setPostFavorite`); see that file for the patch-vs-invalidate logic. In
 * short: a field change (e.g. mark-as-read) patches the cached rows in place,
 * while a change to which list a post belongs to (e.g. (un)favorite) invalidates
 * that list so the subscribed hook refetches and the row appears/disappears.
 */
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { archivePost, unarchivePost, updatePost } from "../api/posts";
import { setFeedUnreadCount } from "../cache/feeds";
import { setPostArchived, setPostRead, setPostFavorite } from "../cache/posts";
import type { PostId } from "../types/PostId";

export function useArchivePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: archivePost,
    onSuccess: (archive) => setPostArchived(queryClient, archive.id, true),
  });
}

export function useUnarchivePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: unarchivePost,
    onSuccess: (_, postId) => setPostArchived(queryClient, postId, false),
  });
}

export function useSetPostRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, isRead }: { id: PostId; isRead: boolean }) =>
      updatePost(id, { is_read: isRead, is_favorite: null }),
    onSuccess: ({ post, feed_unread_count }) => {
      setPostRead(queryClient, post.id, post.is_read);
      setFeedUnreadCount(queryClient, post.feed_id, feed_unread_count);
    },
  });
}

export function useSetPostFavorite() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, isFavorite }: { id: PostId; isFavorite: boolean }) =>
      updatePost(id, { is_favorite: isFavorite, is_read: null }),
    onSuccess: ({ post, feed_unread_count }) => {
      setPostFavorite(queryClient, post.id, post.is_favorite);
      // The count can't change on a favorite toggle, but the response carries
      // the current value for free — applying it reconciles any drift from
      // background polls.
      setFeedUnreadCount(queryClient, post.feed_id, feed_unread_count);
    },
  });
}
