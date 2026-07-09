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
// `feed_id` scopes the list to one feed and keeps its own cache namespace.
export function usePosts(params: ListPosts = {}) {
  const { feed_id, search, exclude, unread_only } = params;
  return useInfiniteQuery(
    infinitePostsQuery(
      feed_id
        ? postKeys.byFeed(feed_id, search, unread_only)
        : postKeys.listed(search, exclude, unread_only),
      (page) => listPosts({ ...params, page }),
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

