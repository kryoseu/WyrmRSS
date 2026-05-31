import type { Post } from "../types/Post";
import type { PagedResponse } from "../types/PagedResponse";
import type { UpdatePost } from "../types/UpdatePost";
import { ENDPOINTS, json } from "../utils/api";

export const getPosts = (page?: string, tag?: string): Promise<PagedResponse<Array<Post>>> =>
  fetch(ENDPOINTS.posts.list(page, tag)).then<PagedResponse<Array<Post>>>(json);

export const getPost = (id: number): Promise<Post> =>
  fetch(ENDPOINTS.posts.get(id)).then<Post>(json);

export const getPostsByFeed = (feedId: number, page?: string): Promise<PagedResponse<Array<Post>>> =>
  fetch(ENDPOINTS.posts.getByFeed(feedId, page)).then<PagedResponse<Array<Post>>>(json);

export const getFavoritePosts = (page?: string): Promise<PagedResponse<Array<Post>>> =>
  fetch(ENDPOINTS.posts.getFavorites(page)).then<PagedResponse<Array<Post>>>(json);

export const updatePost = (id: number, data: UpdatePost): Promise<Post> =>
  fetch(ENDPOINTS.posts.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  }).then<Post>(json);
