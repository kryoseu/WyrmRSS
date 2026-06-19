import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  attachWebhook,
  createWebhook,
  deleteWebhook,
  detachWebhook,
  getFeedWebhooks,
  getWebhooks,
  updateWebhook,
} from "../api/webhooks";
import type { CreateWebhook } from "../types/CreateWebhook";
import type { UpdateWebhook } from "../types/UpdateWebhook";

export const webhookKeys = {
  all: ["webhooks"] as const,
  forFeed: (feedId: number) => ["webhooks", "feed", feedId] as const,
};

export function useWebhooks() {
  return useQuery({
    queryKey: webhookKeys.all,
    queryFn: getWebhooks,
    // Webhooks change rarely and every mutation invalidates this key, so don't
    // refetch on each mount (the edit form remounts on every "Edit" click).
    staleTime: Infinity,
  });
}

export function useCreateWebhook() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateWebhook) => createWebhook(body),
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.all }),
  });
}

export function useUpdateWebhook() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, body }: { id: number; body: UpdateWebhook }) => updateWebhook(id, body),
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.all }),
  });
}

export function useDeleteWebhook() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => deleteWebhook(id),
    // Deleting cascades to feed_webhooks, so the "all" list and every per-feed
    // list may change. The "webhooks" prefix invalidates both.
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.all }),
  });
}

export function useFeedWebhooks(feedId: number) {
  return useQuery({
    queryKey: webhookKeys.forFeed(feedId),
    queryFn: () => getFeedWebhooks(feedId),
    // Per-feed assignments are invalidated on save; no need to refetch when
    // re-opening the same feed's edit form.
    staleTime: Infinity,
  });
}

export function useAttachWebhook(feedId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (webhookId: number) => attachWebhook(feedId, webhookId),
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.forFeed(feedId) }),
  });
}

export function useDetachWebhook(feedId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (webhookId: number) => detachWebhook(feedId, webhookId),
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.forFeed(feedId) }),
  });
}
