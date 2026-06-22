import type { ListPostArchive } from "../types/ListPostArchive";
import type { ListPosts } from "../types/ListPosts";
import type { FeedId } from "../types/FeedId";
import type { PostId } from "../types/PostId";
import type { WebhookId } from "../types/WebhookId";
import { handleUnauthorized } from "./auth";

const BASE = "/api/v1";

// Drops undefined params before building the query string,
// e.g. { page: "1", tag: undefined, search: "foo" } → "page=1&search=foo".
function buildUrl(base: string, params: Record<string, string | undefined>): string {
  const qs = new URLSearchParams(
    Object.entries(params).filter(([, v]) => v !== undefined) as [string, string][]);
  return qs ? `${base}?${qs}` : base;
}

export const ENDPOINTS = {
  feeds: {
    list: () => `${BASE}/feeds`,
    get: (id: FeedId) => `${BASE}/feeds/${id}`,
    create: () => `${BASE}/feeds`,
    update: (id: FeedId) => `${BASE}/feeds/${id}`,
    delete: (id: FeedId) => `${BASE}/feeds/${id}`,
    poll: () => `${BASE}/feeds/poll`,
  },
  posts: {
    list: ({ page, tag, search }: ListPosts) =>
      buildUrl(`${BASE}/posts`, { page, tag, search }),

    get: (id: PostId) => `${BASE}/posts/${id}`,

    listByFeed: (id: FeedId, { page, search }: ListPosts) =>
      buildUrl(`${BASE}/feeds/${id}/posts`, { page, search }),

    listFavorites: ({ page, search }: ListPosts) =>
      buildUrl(`${BASE}/posts/favorites`, { page, search }),

    listArchived: ({ page, tag, search }: ListPostArchive) =>
      buildUrl(`${BASE}/posts/archive`, { page, tag, search }),

    getArchivedPost: (id: PostId) => `${BASE}/posts/archive/${id}`,
    update: (id: PostId) => `${BASE}/posts/${id}`,
    archive: (id: PostId) => `${BASE}/posts/archive/${id}`,
    unarchive: (id: PostId) => `${BASE}/posts/archive/${id}`,
  },
  settings: {
    get: () => `${BASE}/settings`,
    update: () => `${BASE}/settings`,
    import: () => `${BASE}/settings/opml/import`,
    export: () => `${BASE}/settings/opml/export`,
  },
  webhooks: {
    list: () => `${BASE}/webhooks`,
    get: (id: WebhookId) => `${BASE}/webhooks/${id}`,
    create: () => `${BASE}/webhooks`,
    update: (id: WebhookId) => `${BASE}/webhooks/${id}`,
    delete: (id: WebhookId) => `${BASE}/webhooks/${id}`,
    listForFeed: (id: FeedId) => `${BASE}/feeds/${id}/webhooks`,
    attach: (feedId: FeedId, webhookId: WebhookId) =>
      `${BASE}/feeds/${feedId}/webhooks/${webhookId}`,
    detach: (feedId: FeedId, webhookId: WebhookId) =>
      `${BASE}/feeds/${feedId}/webhooks/${webhookId}`,
  },
} as const;


export async function json<T>(res: Response): Promise<T> {
  if (res.status === 401) {
    handleUnauthorized();
    throw new Error("Unauthorized");
  }
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}

export async function noContent(res: Response): Promise<void> {
  if (res.status === 401) {
    handleUnauthorized();
    throw new Error("Unauthorized");
  }
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
}
