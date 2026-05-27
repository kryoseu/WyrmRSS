import type { Feed } from "../types/Feed";
import type { CreateFeed } from "../types/CreateFeed";
import type { UpdateFeed } from "../types/UpdateFeed";
import { ENDPOINTS, json } from "../utils/api";

export const getFeeds = (): Promise<Feed[]> =>
  fetch(ENDPOINTS.feeds.list()).then<Feed[]>(json);

export const getFeed = (id: number): Promise<Feed> =>
  fetch(ENDPOINTS.feeds.get(id)).then<Feed>(json);

export const createFeed = (body: CreateFeed): Promise<Feed> =>
  fetch(ENDPOINTS.feeds.create(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Feed>(json);

export const updateFeed = (id: number, body: UpdateFeed): Promise<Feed> =>
  fetch(ENDPOINTS.feeds.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Feed>(json);

export const deleteFeed = (id: number): Promise<Feed> =>
  fetch(ENDPOINTS.feeds.delete(id), { method: "DELETE" }).then<Feed>(json);
