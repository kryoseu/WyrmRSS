import type { Post } from "../types/Post";
import type { PostId } from "../types/PostId";
import type { PagedResponse } from "../types/PagedResponse";
import type { UpdatePost } from "../types/UpdatePost";
import type { UpdatePostResponse } from "../types/UpdatePostResponse";
import type { ListPosts } from "../types/ListPosts";
import type { MarkRead } from "../types/MarkRead";
import { ENDPOINTS } from "../utils/api";
import type { ListPostArchive } from "../types/ListPostArchive";
import type { PostArchive } from "../types/PostArchive";
import { fetchWithAuth } from "../utils/auth";
import { json, noContent } from "../utils/response";

export const listPosts = (params: ListPosts): Promise<PagedResponse<Array<Post>>> =>
  fetchWithAuth(ENDPOINTS.posts.list(params)).then<PagedResponse<Array<Post>>>(json);

export const getPost = (id: PostId): Promise<Post> =>
  fetchWithAuth(ENDPOINTS.posts.get(id)).then<Post>(json);

export const listArchivedPosts = (params: ListPostArchive): Promise<PagedResponse<Array<PostArchive>>> =>
  fetchWithAuth(ENDPOINTS.posts.listArchived(params)).then<PagedResponse<Array<PostArchive>>>(json);

export const getArchivedPost = (id: PostId): Promise<PostArchive> =>
  fetchWithAuth(ENDPOINTS.posts.getArchivedPost(id)).then<PostArchive>(json);

export const archivePost = (id: PostId): Promise<PostArchive> =>
  fetchWithAuth(ENDPOINTS.posts.archive(id), { method: "POST" }).then<PostArchive>(json);

export const unarchivePost = (id: PostId): Promise<void> =>
  fetchWithAuth(ENDPOINTS.posts.unarchive(id), { method: "DELETE" }).then(noContent);

export const markPostsRead = (body: MarkRead): Promise<void> =>
  fetchWithAuth(ENDPOINTS.posts.markRead(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(noContent);

export const updatePost = (id: PostId, data: UpdatePost): Promise<UpdatePostResponse> =>
  fetchWithAuth(ENDPOINTS.posts.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  }).then<UpdatePostResponse>(json);
