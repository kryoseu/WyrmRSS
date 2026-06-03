import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  type InfiniteData,
} from "@tanstack/react-query";
import { getFavoritePosts, getPost, getPosts, getPostsByFeed, updatePost } from "../api/posts";
import type { Post } from "../types/Post";
import type { PagedResponse } from "../types/PagedResponse";
import type { UpdatePost } from "../types/UpdatePost";

export const postKeys = {
  all: ["posts"] as const,
  listed: (tag?: string, search?: string) => ["posts", "list", { tag, search }] as const,
  byFeed: (feedId: number) => ["posts", "feed", feedId] as const,
  favorites: ["posts", "favorites"] as const,
  detail: (id: number) => ["posts", id] as const,
};

export function usePosts(feedId?: number, tag?: string, search?: string) {
  return useInfiniteQuery({
    queryKey: feedId ? postKeys.byFeed(feedId) : postKeys.listed(tag, search),
    queryFn: ({ pageParam }) =>
      feedId ? getPostsByFeed(feedId, { page: pageParam, search }) : getPosts({ page: pageParam, tag, search }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage): string | undefined =>
      lastPage.next_page ?? undefined,
  });
}

export function useFavoritePosts() {
  return useInfiniteQuery({
    queryKey: postKeys.favorites,
    queryFn: ({ pageParam }) => getFavoritePosts(pageParam),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage): string | undefined =>
      lastPage.next_page ?? undefined,
  });
}

export function usePost(id?: number) {
  return useQuery({
    queryKey: postKeys.detail(id!),
    queryFn: () => getPost(id!),
    enabled: id !== undefined,
  });
}

export function useUpdatePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...data }: { id: number } & UpdatePost) => updatePost(id, data),
    onSuccess: (post: Post) => {
      queryClient.setQueryData(postKeys.detail(post.id), post);

      const updatePages = (old: InfiniteData<PagedResponse<Array<Post>>> | undefined) => {
        if (!old) return old;
        return {
          ...old,
          pages: old.pages.map((page) => ({
            ...page,
            items: page.items.map((p) => (p.id === post.id ? post : p)),
          })),
        };
      };

      queryClient.setQueriesData({ queryKey: ["posts", "list"] }, updatePages);
      queryClient.setQueryData(postKeys.byFeed(post.feed_id), updatePages);
      queryClient.invalidateQueries({ queryKey: postKeys.favorites });
    },
  });
}
