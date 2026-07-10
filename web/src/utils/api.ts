import type { ListPostArchive } from "../types/ListPostArchive";
import type { ListPosts } from "../types/ListPosts";
import type { FeedId } from "../types/FeedId";
import type { PostId } from "../types/PostId";
import type { WebhookId } from "../types/WebhookId";
import { handleUnauthorized } from "./auth";
import type { FolderId } from "../types/FolderId";

const BASE = "/api/v1";

// Drops undefined params before building the query string,
// e.g. { page: "1", feed_id: undefined, search: "foo" } → "page=1&search=foo".
function buildUrl(base: string, params: Record<string, string | undefined>): string {
  const qs = new URLSearchParams(
    Object.entries(params).filter(([, v]) => v !== undefined) as [string, string][]);
  return qs ? `${base}?${qs}` : base;
}

export const ENDPOINTS = {
  feeds: {
    list: () => `${BASE}/feeds`,
    create: () => `${BASE}/feeds`,
    get: (id: FeedId) => `${BASE}/feeds/${id}`,
    update: (id: FeedId) => `${BASE}/feeds/${id}`,
    delete: (id: FeedId) => `${BASE}/feeds/${id}`,
    poll: () => `${BASE}/feeds/poll`,
  },
  posts: {
    list: ({ page, feed_id, bookmarked, unread_only, search, exclude }: ListPosts) =>
      buildUrl(`${BASE}/posts`, {
        page,
        feed_id: feed_id !== undefined ? String(feed_id) : undefined,
        bookmarked: bookmarked ? "true" : undefined,
        unread_only: unread_only !== undefined ? String(unread_only) : undefined,
        search,
        exclude: exclude?.length ? exclude.join(",") : undefined,
      }),

    get: (id: PostId) => `${BASE}/posts/${id}`,

    listArchived: ({ page, search }: ListPostArchive) =>
      buildUrl(`${BASE}/posts/archive`, { page, search }),

    getArchivedPost: (id: PostId) => `${BASE}/posts/archive/${id}`,
    update: (id: PostId) => `${BASE}/posts/${id}`,
    archive: (id: PostId) => `${BASE}/posts/archive/${id}`,
    unarchive: (id: PostId) => `${BASE}/posts/archive/${id}`,
  },
  folders: {
    list: () => `${BASE}/folders`,
    create: () => `${BASE}/folders`,
    update: (id: FolderId) => `${BASE}/folders/${id}`,
    delete: (id: FolderId) => `${BASE}/folders/${id}`,
  },
  settings: {
    get: () => `${BASE}/settings`,
    update: () => `${BASE}/settings`,
    import: () => `${BASE}/settings/opml/import`,
    export: () => `${BASE}/settings/opml/export`,
  },
  webhooks: {
    list: () => `${BASE}/webhooks`,
    create: () => `${BASE}/webhooks`,
    get: (id: WebhookId) => `${BASE}/webhooks/${id}`,
    update: (id: WebhookId) => `${BASE}/webhooks/${id}`,
    delete: (id: WebhookId) => `${BASE}/webhooks/${id}`,
    listForFeed: (id: FeedId) => `${BASE}/feeds/${id}/webhooks`,
    attach: (feedId: FeedId, webhookId: WebhookId) =>
      `${BASE}/feeds/${feedId}/webhooks/${webhookId}`,
    detach: (feedId: FeedId, webhookId: WebhookId) =>
      `${BASE}/feeds/${feedId}/webhooks/${webhookId}`,
  },
} as const;


/** API failure carrying the status and the server's curated `error` message. */
export class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function throwApiError(res: Response): Promise<never> {
  const body = (await res.json().catch(() => undefined)) as { error?: string } | undefined;
  throw new ApiError(res.status, body?.error ?? `${res.status} ${res.statusText}`);
}

export async function json<T>(res: Response): Promise<T> {
  if (res.status === 401) {
    handleUnauthorized();
    throw new Error("Unauthorized");
  }
  if (!res.ok) await throwApiError(res);
  return res.json();
}

export async function noContent(res: Response): Promise<void> {
  if (res.status === 401) {
    handleUnauthorized();
    throw new Error("Unauthorized");
  }
  if (!res.ok) await throwApiError(res);
}
