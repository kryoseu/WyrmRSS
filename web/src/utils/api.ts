import type { ListPosts } from "../types/ListPosts";

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
    list: ({ page, tag, search }: ListPosts) => {
      const params = new URLSearchParams();
      if (page) params.set("page", page);
      if (tag) params.set("tag", tag);
      if (search) params.set("search", search);
      const qs = params.toString();
      return `${BASE}/posts${qs ? `?${qs}` : ""}`;
    },
    get: (id: number) => `${BASE}/posts/${id}`,
    getByFeed: (feedId: number, { page, search }: ListPosts) => {
      const params = new URLSearchParams();
      if (page) params.set("page", page);
      if (search) params.set("search", search);
      const qs = params.toString();
      return `${BASE}/feeds/${feedId}/posts${qs ? `?${qs}` : ""}`;
    },
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
