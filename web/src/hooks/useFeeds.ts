import { useMemo } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFeed, deleteFeed, getFeeds, pollFeeds, updateFeed } from "../api/feeds";
import { feedKeys } from "../cache/feeds";
import { postKeys } from "../cache/posts";
import type { CreateFeed } from "../types/CreateFeed";
import type { UpdateFeed } from "../types/UpdateFeed";
import type { FeedId } from "../types/FeedId";
import type { Feed } from "../types/Feed";
import type { Tag } from "../components/PostsTagChips";

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

/** Extracts tags from feeds into a Tag array */
export function useFeedTags(feeds: Feed[] | undefined): Tag[] {
  return useMemo(() => {
    const byName = new Map<string, Tag>();
    for (const f of feeds ?? []) {
      if (f.tag !== undefined && !byName.has(f.tag)) {
        byName.set(f.tag, { name: f.tag, color: f.tag_color });
      }
    }
    return [...byName.values()];
  }, [feeds]);
}

export function useCreateFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateFeed) => createFeed(body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: feedKeys.all });
      const refresh = () => qc.invalidateQueries({ queryKey: postKeys.all });
      pollFeeds().then(refresh).catch(refresh);
    },
  });
}

export function useUpdateFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, body }: { id: FeedId; body: UpdateFeed }) =>
      updateFeed(id, body),
    onSuccess: () => qc.invalidateQueries({ queryKey: feedKeys.all }),
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
