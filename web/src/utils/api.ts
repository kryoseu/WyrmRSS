import type { ListPostArchive } from "../types/ListPostArchive";
import type { ListPosts } from "../types/ListPosts";
import type { FeedId } from "../types/FeedId";
import type { PostId } from "../types/PostId";
import type { WebhookId } from "../types/WebhookId";
import type { FolderId } from "../types/FolderId";

// Same-origin by default (self-hosted, reverse-proxied). The desktop build
// points this at the embedded backend's dynamic local port at startup.
let apiOrigin = "";

export function setApiOrigin(origin: string): void {
  apiOrigin = origin;
}

// Evaluated per-call (not captured at module load) so ENDPOINTS reflects an
// origin set after startup, e.g. once the desktop runtime resolves it.
function apiBase(): string {
  return `${apiOrigin}/api/v1`;
}

// Drops undefined params before building the query string,
// e.g. { page: "1", feed_id: undefined, search: "foo" } → "page=1&search=foo".
function buildUrl(base: string, params: Record<string, string | undefined>): string {
  const qs = new URLSearchParams(
    Object.entries(params).filter(([, v]) => v !== undefined) as [string, string][]);
  return qs ? `${base}?${qs}` : base;
}

export const ENDPOINTS = {
  auth: {
    verify: () => `${apiBase()}/auth/verify`,
  },
  feeds: {
    list: () => `${apiBase()}/feeds`,
    create: () => `${apiBase()}/feeds`,
    get: (id: FeedId) => `${apiBase()}/feeds/${id}`,
    icon: (id: FeedId) => `${apiBase()}/feeds/${id}/icon`,
    update: (id: FeedId) => `${apiBase()}/feeds/${id}`,
    delete: (id: FeedId) => `${apiBase()}/feeds/${id}`,
    poll: () => `${apiBase()}/feeds/poll`,
  },
  posts: {
    list: ({ page, feed_id, bookmarked, unread_only, search }: ListPosts) =>
      buildUrl(`${apiBase()}/posts`, {
        page,
        feed_id: feed_id !== undefined ? String(feed_id) : undefined,
        bookmarked: bookmarked ? "true" : undefined,
        unread_only: unread_only !== undefined ? String(unread_only) : undefined,
        search,
      }),

    get: (id: PostId) => `${apiBase()}/posts/${id}`,
    markRead: () => `${apiBase()}/posts/mark-read`,

    listArchived: ({ page, search }: ListPostArchive) =>
      buildUrl(`${apiBase()}/posts/archive`, { page, search }),

    getArchivedPost: (id: PostId) => `${apiBase()}/posts/archive/${id}`,
    update: (id: PostId) => `${apiBase()}/posts/${id}`,
    archive: (id: PostId) => `${apiBase()}/posts/archive/${id}`,
    unarchive: (id: PostId) => `${apiBase()}/posts/archive/${id}`,
  },
  folders: {
    list: () => `${apiBase()}/folders`,
    create: () => `${apiBase()}/folders`,
    update: (id: FolderId) => `${apiBase()}/folders/${id}`,
    delete: (id: FolderId) => `${apiBase()}/folders/${id}`,
  },
  settings: {
    get: () => `${apiBase()}/settings`,
    update: () => `${apiBase()}/settings`,
    import: () => `${apiBase()}/settings/opml/import`,
    export: () => `${apiBase()}/settings/opml/export`,
  },
  webhooks: {
    list: () => `${apiBase()}/webhooks`,
    create: () => `${apiBase()}/webhooks`,
    get: (id: WebhookId) => `${apiBase()}/webhooks/${id}`,
    update: (id: WebhookId) => `${apiBase()}/webhooks/${id}`,
    delete: (id: WebhookId) => `${apiBase()}/webhooks/${id}`,
    listForFeed: (id: FeedId) => `${apiBase()}/feeds/${id}/webhooks`,
    attach: (feedId: FeedId, webhookId: WebhookId) =>
      `${apiBase()}/feeds/${feedId}/webhooks/${webhookId}`,
    detach: (feedId: FeedId, webhookId: WebhookId) =>
      `${apiBase()}/feeds/${feedId}/webhooks/${webhookId}`,
  },
  // Desktop only, unauthenticated, and deliberately not under /api/v1 -- see
  // the matching route in desktop/src-tauri/src/backend.rs.
  desktop: {
    youtubeEmbed: (videoId: string) => `${apiOrigin}/youtube-embed?v=${encodeURIComponent(videoId)}`,
  },
} as const;
