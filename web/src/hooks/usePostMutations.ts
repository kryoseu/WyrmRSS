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
 * `applyPostEdit`); see that file for the patch-vs-invalidate logic. In
 * short: a field change (e.g. mark-as-read) patches the cached rows in place,
 * while a change to which list a post belongs to (e.g. (un)favorite) invalidates
 * that list so the subscribed hook refetches and the row appears/disappears.
 */
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { archivePost, unarchivePost, updatePost } from "../api/posts";
import type { UpdatePost } from "../types/UpdatePost";
import { setPostArchived, applyPostEdit } from "../cache/posts";

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

export function useUpdatePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...data }: { id: number } & UpdatePost) => updatePost(id, data),
    // is_favorite is null when this edit didn't touch the favorite flag (e.g. a
    // read toggle); a non-null value means the user (un)favorited the post.
    onSuccess: (post, variables) => applyPostEdit(queryClient, post, variables.is_favorite !== null),
  });
}
