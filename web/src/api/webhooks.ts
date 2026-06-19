import type { Webhook } from "../types/Webhook";
import type { CreateWebhook } from "../types/CreateWebhook";
import type { UpdateWebhook } from "../types/UpdateWebhook";
import { ENDPOINTS, json, noContent } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";

export const getWebhooks = (): Promise<Webhook[]> =>
  fetchWithAuth(ENDPOINTS.webhooks.list()).then<Webhook[]>(json);

export const getWebhook = (id: number): Promise<Webhook> =>
  fetchWithAuth(ENDPOINTS.webhooks.get(id)).then<Webhook>(json);

export const getFeedWebhooks = (feedId: number): Promise<Webhook[]> =>
  fetchWithAuth(ENDPOINTS.webhooks.listForFeed(feedId)).then<Webhook[]>(json);

export const createWebhook = (body: CreateWebhook): Promise<Webhook> =>
  fetchWithAuth(ENDPOINTS.webhooks.create(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Webhook>(json);

export const updateWebhook = (id: number, body: UpdateWebhook): Promise<Webhook> =>
  fetchWithAuth(ENDPOINTS.webhooks.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Webhook>(json);

export const deleteWebhook = (id: number): Promise<Webhook> =>
  fetchWithAuth(ENDPOINTS.webhooks.delete(id), { method: "DELETE" }).then<Webhook>(json);

export const attachWebhook = (feedId: number, webhookId: number): Promise<void> =>
  fetchWithAuth(ENDPOINTS.webhooks.attach(feedId, webhookId), { method: "PUT" }).then(noContent);

export const detachWebhook = (feedId: number, webhookId: number): Promise<void> =>
  fetchWithAuth(ENDPOINTS.webhooks.detach(feedId, webhookId), { method: "DELETE" }).then(
    noContent,
  );
