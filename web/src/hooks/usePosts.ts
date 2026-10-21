import {
  infiniteQueryOptions,
  useInfiniteQuery,
  useQuery,
  type QueryKey,
} from "@tanstack/react-query";
import { getArchivedPost, getPost, listArchivedPosts, listPosts } from "../api/posts";
import type { PagedResponse } from "../types/PagedResponse";
import type { PostId } from "../types/PostId";
import type { ListPosts } from "../types/ListPosts";
import { postKeys } from "../cache/posts";

type ArchivedQuery = { search?: string };

// Shared config for the cursor-paginated post lists: same paging behaviour,
// differing only by cache key and which endpoint fetches a page.
function infinitePostsQuery<T>(
  queryKey: QueryKey,
  fetchPage: (page?: string) => Promise<PagedResponse<T[]>>,
) {
  return infiniteQueryOptions({
    queryKey,
    queryFn: ({ pageParam }) => fetchPage(pageParam),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.next_page ?? undefined,
  });
}

export function usePost(id?: PostId) {
  return useQuery({
    queryKey: postKeys.detail(id!),
    queryFn: () => getPost(id!),
    enabled: id !== undefined,
  });
}

// `params.page` is ignored: the infinite query owns the cursor and sets it per page.
// `feed_id` / `folder_id` scope the list to one feed or folder, each with its own cache namespace.
export function usePosts(params: ListPosts = {}) {
  const { feed_id, folder_id, search, unread_only } = params;
  // A feed is narrower than its folder, so feed_id wins and folder_id isn't sent.
  const scoped = feed_id ? { ...params, folder_id: undefined } : params;
  return useInfiniteQuery(
    infinitePostsQuery(
      feed_id
        ? postKeys.byFeed(feed_id, search, unread_only)
        : folder_id
          ? postKeys.byFolder(folder_id, search, unread_only)
          : postKeys.listed(search, unread_only),
      (page) => listPosts({ ...scoped, page }),
    ),
  );
}

export function useBookmarkedPosts(search?: string) {
  return useInfiniteQuery(
    infinitePostsQuery(postKeys.bookmarked(search), (page) =>
      // explicit false: bookmarked posts are usually read, the unread-only API default would hide them
      listPosts({ bookmarked: true, unread_only: false, page, search })),
  );
}

export function useArchivedPost(id?: PostId) {
  return useQuery({
    queryKey: postKeys.archiveDetail(id!),
    queryFn: () => getArchivedPost(id!),
    enabled: id !== undefined,
  });
}

export function useArchivedPosts({ search }: ArchivedQuery = {}) {
  return useInfiniteQuery(
    infinitePostsQuery(postKeys.archived(search), (page) => listArchivedPosts({ page, search })),
  );
}

