import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFeed, deleteFeed, getFeeds, pollFeeds, updateFeed } from "../api/feeds";
import { feedKeys } from "../cache/feeds";
import { postKeys } from "../cache/posts";
import { folderKeys } from "./useFolders";
import type { CreateFeed } from "../types/CreateFeed";
import type { UpdateFeed } from "../types/UpdateFeed";
import type { FeedId } from "../types/FeedId";

export function useFeeds() {
  return useQuery({
    queryKey: feedKeys.all,
    queryFn: getFeeds,
    // Don't refetch on every mount (settings/sidebar remount often). Mutations
    // — create/update/delete/poll — invalidate this key, which refetches even
    // under `Infinity`. Only an autonomous background poll won't be reflected
    // until the next user action or reload.
    staleTime: Infinity,
  });
}

export function useCreateFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateFeed) => createFeed(body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: feedKeys.all });
      // Submitting a folder name can resolve-or-create a folder server-side.
      qc.invalidateQueries({ queryKey: folderKeys.all });
      const refresh = () => {
        qc.invalidateQueries({ queryKey: postKeys.all });
        // Unread counts live on the feeds query; refetch once the poll lands.
        qc.invalidateQueries({ queryKey: feedKeys.all });
      };
      pollFeeds().then(refresh).catch(refresh);
    },
  });
}

export function useUpdateFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, body }: { id: FeedId; body: UpdateFeed }) =>
      updateFeed(id, body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: feedKeys.all });
      qc.invalidateQueries({ queryKey: folderKeys.all });
    },
  });
}

export function useDeleteFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: FeedId) => deleteFeed(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: feedKeys.all }),
  });
}

export function usePollFeeds() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: pollFeeds,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: feedKeys.all });
      qc.invalidateQueries({ queryKey: postKeys.all });
    },
  });
}
