import type { Feed } from "../types/Feed";
import type { CreateFeed } from "../types/CreateFeed";
import type { UpdateFeed } from "../types/UpdateFeed";
import { ENDPOINTS, json, noContent } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";

export const getFeeds = (): Promise<Feed[]> =>
  fetchWithAuth(ENDPOINTS.feeds.list()).then<Feed[]>(json);

export const getFeed = (id: number): Promise<Feed> =>
  fetchWithAuth(ENDPOINTS.feeds.get(id)).then<Feed>(json);

export const createFeed = (body: CreateFeed): Promise<Feed> =>
  fetchWithAuth(ENDPOINTS.feeds.create(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Feed>(json);

export const updateFeed = (id: number, body: UpdateFeed): Promise<Feed> =>
  fetchWithAuth(ENDPOINTS.feeds.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Feed>(json);

export const deleteFeed = (id: number): Promise<Feed> =>
  fetchWithAuth(ENDPOINTS.feeds.delete(id), { method: "DELETE" }).then<Feed>(json);

export const pollFeeds = (): Promise<void> =>
  fetchWithAuth(ENDPOINTS.feeds.poll(), { method: "POST" }).then(noContent);
