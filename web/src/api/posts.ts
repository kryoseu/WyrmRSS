import type { Post } from "../types/Post";
import type { PostsPage } from "../types/PostsPage";
import type { UpdatePost } from "../types/UpdatePost";
import { ENDPOINTS, json, type Cursor } from "../utils/api";

export const getPosts = (cursor?: Cursor): Promise<PostsPage> =>
  fetch(ENDPOINTS.posts.list(cursor)).then<PostsPage>(json);

export const getPost = (id: number): Promise<Post> =>
  fetch(ENDPOINTS.posts.get(id)).then<Post>(json);

export const getPostsByFeed = (feedId: number, cursor?: Cursor): Promise<PostsPage> =>
  fetch(ENDPOINTS.posts.getByFeed(feedId, cursor)).then<PostsPage>(json);

export const getFavoritePosts = (cursor?: Cursor): Promise<PostsPage> =>
  fetch(ENDPOINTS.posts.getFavorites(cursor)).then<PostsPage>(json);

export const updatePost = (id: number, data: UpdatePost): Promise<Post> =>
  fetch(ENDPOINTS.posts.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  }).then<Post>(json);
