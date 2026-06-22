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
import type { FeedId } from "../types/FeedId";
import type { WebhookId } from "../types/WebhookId";

export const webhookKeys = {
  all: ["webhooks"] as const,
  forFeed: (id: FeedId) => ["webhooks", "feed", id] as const,
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
    mutationFn: ({ id, body }: { id: WebhookId; body: UpdateWebhook }) => updateWebhook(id, body),
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.all }),
  });
}

export function useDeleteWebhook() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: WebhookId) => deleteWebhook(id),
    // Deleting cascades to feed_webhooks, so the "all" list and every per-feed
    // list may change. The "webhooks" prefix invalidates both.
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.all }),
  });
}

export function useFeedWebhooks(id: FeedId) {
  return useQuery({
    queryKey: webhookKeys.forFeed(id),
    queryFn: () => getFeedWebhooks(id),
    // Per-feed assignments are invalidated on save; no need to refetch when
    // re-opening the same feed's edit form.
    staleTime: Infinity,
  });
}

export function useAttachWebhook(feedId: FeedId) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (webhookId: WebhookId) => attachWebhook(feedId, webhookId),
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.forFeed(feedId) }),
  });
}

export function useDetachWebhook(feedId: FeedId) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (webhookId: WebhookId) => detachWebhook(feedId, webhookId),
    onSuccess: () => qc.invalidateQueries({ queryKey: webhookKeys.forFeed(feedId) }),
  });
}
