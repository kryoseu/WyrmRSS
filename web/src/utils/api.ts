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
  },
  posts: {
    list: (cursor?: Cursor) => `${BASE}/posts${cursorParams(cursor)}`,
    get: (id: number) => `${BASE}/posts/${id}`,
    getByFeed: (feedId: number, cursor?: Cursor) => `${BASE}/feeds/${feedId}/posts${cursorParams(cursor)}`,
    getFavorites: (cursor?: Cursor) => `${BASE}/posts/favorites${cursorParams(cursor)}`,
    toggleFavorite: (id: number) => `${BASE}/posts/${id}/favorite`,
  },
} as const;

export async function json<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}
