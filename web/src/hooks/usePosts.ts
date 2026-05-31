import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  type InfiniteData,
} from "@tanstack/react-query";
import { getFavoritePosts, getPost, getPosts, getPostsByFeed, updatePost } from "../api/posts";
import type { Post } from "../types/Post";
import type { PostsPage } from "../types/PostsPage";
import type { UpdatePost } from "../types/UpdatePost";
import type { Cursor } from "../utils/api";

export const postKeys = {
  all: ["posts"] as const,
  byTag: (tag: string) => ["posts", "tag", tag] as const,
  byFeed: (feedId: number) => ["posts", "feed", feedId] as const,
  favorites: ["posts", "favorites"] as const,
  detail: (id: number) => ["posts", id] as const,
};

export function usePosts(feedId?: number, tag?: string) {
  return useInfiniteQuery({
    queryKey: feedId ? postKeys.byFeed(feedId) : tag ? postKeys.byTag(tag) : postKeys.all,
    queryFn: ({ pageParam: cursor }) =>
      feedId ? getPostsByFeed(feedId, cursor) : getPosts(cursor, tag),
    initialPageParam: undefined as Cursor | undefined,
    getNextPageParam: (lastPage): Cursor | undefined => {
      if (!lastPage.has_more) return undefined;
      const last = lastPage.items.at(-1)!;
      return { timestamp: new Date(last.published_at).getTime(), post_id: last.id };
    },
  });
}

export function useFavoritePosts() {
  return useInfiniteQuery({
    queryKey: postKeys.favorites,
    queryFn: ({ pageParam: cursor }) => getFavoritePosts(cursor),
    initialPageParam: undefined as Cursor | undefined,
    getNextPageParam: (lastPage): Cursor | undefined => {
      if (!lastPage.has_more) return undefined;
      const last = lastPage.items.at(-1)!;
      return { timestamp: new Date(last.published_at).getTime(), post_id: last.id };
    },
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

      const updatePages = (old: InfiniteData<PostsPage> | undefined) => {
        if (!old) return old;
        return {
          ...old,
          pages: old.pages.map((page) => ({
            ...page,
            items: page.items.map((p) => (p.id === post.id ? post : p)),
          })),
        };
      };

      queryClient.setQueryData(postKeys.all, updatePages);
      queryClient.setQueryData(postKeys.byFeed(post.feed_id), updatePages);
      queryClient.setQueriesData({ queryKey: ["posts", "tag"] }, updatePages);
      queryClient.invalidateQueries({ queryKey: postKeys.favorites });
    },
  });
}
