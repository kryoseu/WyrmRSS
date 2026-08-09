import type { Webhook } from "../types/Webhook";
import type { FeedWebhookView } from "../types/FeedWebhookView";
import type { FeedId } from "../types/FeedId";
import type { WebhookId } from "../types/WebhookId";
import type { CreateWebhook } from "../types/CreateWebhook";
import type { UpdateWebhook } from "../types/UpdateWebhook";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import { json, noContent } from "../utils/response";

export const getWebhooks = (): Promise<Webhook[]> =>
  fetchWithAuth(ENDPOINTS.webhooks.list()).then<Webhook[]>(json);

export const getFeedWebhooks = (id: FeedId): Promise<FeedWebhookView[]> =>
  fetchWithAuth(ENDPOINTS.webhooks.listForFeed(id)).then<FeedWebhookView[]>(json);

export const createWebhook = (body: CreateWebhook): Promise<Webhook> =>
  fetchWithAuth(ENDPOINTS.webhooks.create(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Webhook>(json);

export const updateWebhook = (id: WebhookId, body: UpdateWebhook): Promise<Webhook> =>
  fetchWithAuth(ENDPOINTS.webhooks.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Webhook>(json);

export const deleteWebhook = (id: WebhookId): Promise<Webhook> =>
  fetchWithAuth(ENDPOINTS.webhooks.delete(id), { method: "DELETE" }).then<Webhook>(json);

export const attachWebhook = (feedId: FeedId, webhookId: WebhookId): Promise<void> =>
  fetchWithAuth(ENDPOINTS.webhooks.attach(feedId, webhookId), { method: "PUT" }).then(noContent);

export const detachWebhook = (feedId: FeedId, webhookId: WebhookId): Promise<void> =>
  fetchWithAuth(ENDPOINTS.webhooks.detach(feedId, webhookId), { method: "DELETE" }).then(
    noContent,
  );
