const BASE = "/api/v1";

export type Cursor = { timestamp: number; post_id: number };

function cursorParams(cursor?: Cursor): string {
  if (!cursor) return "";
  return `?timestamp=${cursor.timestamp}&post_id=${cursor.post_id}`;
}

export const ENDPOINTS = {
  feeds: {
    list: () => `${BASE}/feeds`,
    get: (id: number) => `${BASE}/feeds/${id}`,
    create: () => `${BASE}/feeds`,
    update: (id: number) => `${BASE}/feeds/${id}`,
    delete: (id: number) => `${BASE}/feeds/${id}`,
    poll: () => `${BASE}/feeds/poll`,
  },
  posts: {
    list: (cursor?: Cursor, tag?: string) => {
      const base = `${BASE}/posts${cursorParams(cursor)}`;
      if (!tag) return base;
      return `${base}${cursor ? "&" : "?"}tag=${encodeURIComponent(tag)}`;
    },
    get: (id: number) => `${BASE}/posts/${id}`,
    getByFeed: (feedId: number, cursor?: Cursor) => `${BASE}/feeds/${feedId}/posts${cursorParams(cursor)}`,
    getFavorites: (cursor?: Cursor) => `${BASE}/posts/favorites${cursorParams(cursor)}`,
    update: (id: number) => `${BASE}/posts/${id}`,
  },
} as const;

export async function json<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}

export async function noContent(res: Response): Promise<void> {
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
}
