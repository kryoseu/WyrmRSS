import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  type InfiniteData,
} from "@tanstack/react-query";
import { archivePost, getArchivedPost, listArchivedPosts, listFavoritePosts, getPost, listPosts, listPostsByFeed, unarchivePost, updatePost } from "../api/posts";
import type { Post } from "../types/Post";
import type { PostArchive } from "../types/PostArchive";
import type { PagedResponse } from "../types/PagedResponse";
import type { UpdatePost } from "../types/UpdatePost";

type PostPages = InfiniteData<PagedResponse<Post[]>> | undefined;

function patchPostInPages(cached: PostPages, patch: (post: Post) => Post): PostPages {
  if (!cached) return cached;
  return {
    ...cached,
    pages: cached.pages.map((page) => ({
      ...page,
      items: page.items.map(patch),
    })),
  };
}

export const postKeys = {
  all: ["posts"] as const,
  listed: (tag?: string, search?: string) => ["posts", "list", { tag, search }] as const,
  byFeed: (feedId: number) => ["posts", "feed", feedId] as const,
  favorites: (search?: string) => ["posts", "favorites", { search }] as const,
  archived: (search?: string, tag?: string) => ["posts", "archived", { search, tag }] as const,
  detail: (id: number) => ["posts", id] as const,
};

export function usePosts(feedId?: number, tag?: string, search?: string) {
  return useInfiniteQuery({
    queryKey: feedId ? postKeys.byFeed(feedId) : postKeys.listed(tag, search),
    queryFn: ({ pageParam }) =>
      feedId ? listPostsByFeed(feedId, { page: pageParam, search }) : listPosts({ page: pageParam, tag, search }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage): string | undefined =>
      lastPage.next_page ?? undefined,
  });
}

export function useFavoritePosts(search?: string) {
  return useInfiniteQuery({
    queryKey: postKeys.favorites(search),
    queryFn: ({ pageParam }) => listFavoritePosts({ page: pageParam, search }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage): string | undefined =>
      lastPage.next_page ?? undefined,
  });
}

export function useArchivedPosts(search?: string, tag?: string) {
  return useInfiniteQuery({
    queryKey: postKeys.archived(search, tag),
    queryFn: ({ pageParam }) => listArchivedPosts({ page: pageParam, search, tag }),
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

export function useArchivedPost(id?: number) {
  return useQuery({
    queryKey: ["posts", "archive", id!],
    queryFn: () => getArchivedPost(id!),
    enabled: id !== undefined,
  });
}

export function useArchivePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (postId: number) => archivePost(postId),
    onSuccess: (archive: PostArchive) => {
      const patch = (p: Post) => p.id === archive.id ? { ...p, is_archived: true } : p;
      queryClient.setQueryData(postKeys.detail(archive.id), (cached: Post | undefined) =>
        cached ? { ...cached, is_archived: true } : cached,
      );
      queryClient.setQueriesData({ queryKey: ["posts", "list"] }, (cached: PostPages) => patchPostInPages(cached, patch));
      queryClient.setQueriesData({ queryKey: ["posts", "feed"] }, (cached: PostPages) => patchPostInPages(cached, patch));
      queryClient.setQueriesData({ queryKey: ["posts", "favorites"] }, (cached: PostPages) => patchPostInPages(cached, patch));
      queryClient.invalidateQueries({ queryKey: ["posts", "archived"] });
    },
  });
}

export function useUnarchivePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (postId: number) => unarchivePost(postId),
    onSuccess: (_, postId: number) => {
      const patch = (p: Post) => p.id === postId ? { ...p, is_archived: false } : p;
      queryClient.setQueryData(postKeys.detail(postId), (cached: Post | undefined) =>
        cached ? { ...cached, is_archived: false } : cached,
      );
      queryClient.setQueriesData({ queryKey: ["posts", "list"] }, (cached: PostPages) => patchPostInPages(cached, patch));
      queryClient.setQueriesData({ queryKey: ["posts", "feed"] }, (cached: PostPages) => patchPostInPages(cached, patch));
      queryClient.setQueriesData({ queryKey: ["posts", "favorites"] }, (cached: PostPages) => patchPostInPages(cached, patch));
      queryClient.invalidateQueries({ queryKey: ["posts", "archived"] });
      queryClient.removeQueries({ queryKey: ["posts", "archive", postId] });
    },
  });
}

export function useUpdatePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...data }: { id: number } & UpdatePost) => updatePost(id, data),
    onSuccess: (post: Post, variables) => {
      const patch = (p: Post) => p.id === post.id ? post : p;
      queryClient.setQueryData(postKeys.detail(post.id), post);
      queryClient.setQueriesData({ queryKey: ["posts", "list"] }, (cached: PostPages) => patchPostInPages(cached, patch));
      queryClient.setQueryData(postKeys.byFeed(post.feed_id), (cached: PostPages) => patchPostInPages(cached, patch));
      if (variables.is_favorite !== null) {
        queryClient.invalidateQueries({ queryKey: ["posts", "favorites"] });
      }
    },
  });
}
