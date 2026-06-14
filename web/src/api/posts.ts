import type { Post } from "../types/Post";
import type { PagedResponse } from "../types/PagedResponse";
import type { UpdatePost } from "../types/UpdatePost";
import type { ListPosts } from "../types/ListPosts";
import { ENDPOINTS, json, noContent } from "../utils/api";
import type { ListPostArchive } from "../types/ListPostArchive";
import type { PostArchive } from "../types/PostArchive";
import { fetchWithAuth } from "../utils/auth";

export const listPosts = (params: ListPosts): Promise<PagedResponse<Array<Post>>> =>
  fetchWithAuth(ENDPOINTS.posts.list(params)).then<PagedResponse<Array<Post>>>(json);

export const getPost = (id: number): Promise<Post> =>
  fetchWithAuth(ENDPOINTS.posts.get(id)).then<Post>(json);

export const listPostsByFeed = (feedId: number, params: ListPosts): Promise<PagedResponse<Array<Post>>> =>
  fetchWithAuth(ENDPOINTS.posts.listByFeed(feedId, params)).then<PagedResponse<Array<Post>>>(json);

export const listFavoritePosts = (params: ListPosts): Promise<PagedResponse<Array<Post>>> =>
  fetchWithAuth(ENDPOINTS.posts.listFavorites(params)).then<PagedResponse<Array<Post>>>(json);

export const listArchivedPosts = (params: ListPostArchive): Promise<PagedResponse<Array<PostArchive>>> =>
  fetchWithAuth(ENDPOINTS.posts.listArchived(params)).then<PagedResponse<Array<PostArchive>>>(json);

export const getArchivedPost = (id: number): Promise<PostArchive> =>
  fetchWithAuth(ENDPOINTS.posts.getArchivedPost(id)).then<PostArchive>(json);

export const archivePost = (id: number): Promise<PostArchive> =>
  fetchWithAuth(ENDPOINTS.posts.archive(id), { method: "POST" }).then<PostArchive>(json);

export const unarchivePost = (id: number): Promise<void> =>
  fetchWithAuth(ENDPOINTS.posts.unarchive(id), { method: "DELETE" }).then(noContent);

export const updatePost = (id: number, data: UpdatePost): Promise<Post> =>
  fetchWithAuth(ENDPOINTS.posts.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  }).then<Post>(json);
