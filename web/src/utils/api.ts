const BASE = "/api/v1";

function pageParam(page?: string): string {
  return page ? `?page=${encodeURIComponent(page)}` : "";
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
    list: (page?: string, tag?: string) => {
      const base = `${BASE}/posts${pageParam(page)}`;
      if (!tag) return base;
      return `${base}${page ? "&" : "?"}tag=${encodeURIComponent(tag)}`;
    },
    get: (id: number) => `${BASE}/posts/${id}`,
    getByFeed: (feedId: number, page?: string) => `${BASE}/feeds/${feedId}/posts${pageParam(page)}`,
    getFavorites: (page?: string) => `${BASE}/posts/favorites${pageParam(page)}`,
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
