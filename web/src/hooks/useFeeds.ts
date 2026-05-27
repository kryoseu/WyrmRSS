import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFeed, deleteFeed, getFeeds, updateFeed } from "../api/feeds";
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
    onSuccess: () => qc.invalidateQueries({ queryKey: feedKeys.all }),
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
