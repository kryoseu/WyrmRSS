import { useMemo } from "react";
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
import type { Tag } from "../components/PostsTagChips";
import { postKeys } from "../cache/posts";

type ArchivedQuery = { search?: string; tag?: string };

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
  const { feed_id, tag, search, exclude, unread_only } = params;
  return useInfiniteQuery(
    infinitePostsQuery(
      feed_id
        ? postKeys.byFeed(feed_id, search, unread_only)
        : postKeys.listed(tag, search, exclude, unread_only),
      (page) => listPosts({ ...params, page }),
    ),
  );
}

export function useFavoritePosts(search?: string) {
  return useInfiniteQuery(
    infinitePostsQuery(postKeys.favorites(search), (page) => listPosts({ fav_only: true, page, search })),
  );
}

export function useArchivedPost(id?: PostId) {
  return useQuery({
    queryKey: postKeys.archiveDetail(id!),
    queryFn: () => getArchivedPost(id!),
    enabled: id !== undefined,
  });
}

export function useArchivedPosts({ search, tag }: ArchivedQuery = {}) {
  return useInfiniteQuery(
    infinitePostsQuery(postKeys.archived(search, tag), (page) => listArchivedPosts({ page, search, tag })),
  );
}

/** Extracts tags from archive posts into a Tag array */
export function useArchiveTags(): Tag[] {
  const { data } = useArchivedPosts();
  return useMemo(() => {
    const map = new Map<string, string | undefined>();
    for (const page of data?.pages ?? []) {
      for (const item of page.items) {
        if (item.tag) map.set(item.tag, item.tag_color ?? undefined);
      }
    }
    return [...map].map(([name, color]) => ({ name, color }));
  }, [data]);
}


