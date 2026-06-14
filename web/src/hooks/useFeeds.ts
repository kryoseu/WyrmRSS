import { useMemo } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFeed, deleteFeed, getFeeds, pollFeeds, updateFeed } from "../api/feeds";
import { postKeys } from "../cache/posts";
import type { CreateFeed } from "../types/CreateFeed";
import type { UpdateFeed } from "../types/UpdateFeed";
import type { Feed } from "../types/Feed";
import type { Tag } from "../components/PostsTagChips";

export const feedKeys = {
  all: ["feeds"] as const,
};

export function useFeeds() {
  return useQuery({ queryKey: feedKeys.all, queryFn: getFeeds });
}

/** Extracts tags from feeds into a Tag array */
export function useFeedTags(feeds: Feed[] | undefined): Tag[] {
  return useMemo(
    () =>
      (feeds ?? [])
        .filter((f) => f.tag !== undefined)
        .map((f) => ({ name: f.tag!, color: f.tag_color })),
    [feeds]
  );
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
    mutationFn: ({ id, body }: { id: number; body: UpdateFeed }) =>
      updateFeed(id, body),
    onSuccess: () => qc.invalidateQueries({ queryKey: feedKeys.all }),
  });
}

export function useDeleteFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => deleteFeed(id),
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
