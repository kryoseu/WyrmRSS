import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFeed, deleteFeed, getFeeds, pollFeeds, updateFeed } from "../api/feeds";
import { postKeys } from "./usePosts";
import type { CreateFeed } from "../types/CreateFeed";
import type { UpdateFeed } from "../types/UpdateFeed";

export const feedKeys = {
  all: ["feeds"] as const,
};

export function useFeeds() {
  return useQuery({ queryKey: feedKeys.all, queryFn: getFeeds });
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
