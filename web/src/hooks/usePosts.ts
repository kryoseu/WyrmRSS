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

// The params are both the request and the cache key, so each filter combination
// gets its own cached list. `params.page` is ignored: the infinite query owns the cursor.
export function usePosts(params: ListPosts = {}) {
  const request: ListPosts = {
    ...params,
    page: undefined,
    // A feed is narrower than its folder, so feed_id wins and folder_id isn't sent.
    folder_id: params.feed_id ? undefined : params.folder_id,
  };
  return useInfiniteQuery(
    infinitePostsQuery(postKeys.list(request), (page) => listPosts({ ...request, page })),
  );
}

export function useBookmarkedPosts(search?: string) {
  return useInfiniteQuery(
    infinitePostsQuery(postKeys.bookmarked(search), (page) =>
      // explicit false: bookmarked posts are usually read, the unread-only API default would hide them
      listPosts({ bookmarked: true, unread_only: false, page, search })),
  );
}

export function useArchivedPosts({ search }: ArchivedQuery = {}) {
  return useInfiniteQuery(
    infinitePostsQuery(postKeys.archived(search), (page) => listArchivedPosts({ page, search })),
  );
}

export function usePost(id?: PostId) {
  return useQuery({
    queryKey: postKeys.byId(id!),
    queryFn: () => getPost(id!),
    enabled: id !== undefined,
  });
}

export function useArchivedPost(id?: PostId) {
  return useQuery({
    queryKey: postKeys.archiveById(id!),
    queryFn: () => getArchivedPost(id!),
    enabled: id !== undefined,
  });
}


