import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFeed, deleteFeed, getFeedIcon, getFeeds, pollFeeds, updateFeed } from "../api/feeds";
import { evictFeedIconIfUrlChanged, feedIconKey, feedKeys, setFeedPauseState } from "../cache/feeds";
import { postKeys } from "../cache/posts";
import { folderKeys } from "./useFolders";
import type { CreateFeed } from "../types/CreateFeed";
import type { UpdateFeed } from "../types/UpdateFeed";
import type { FeedId } from "../types/FeedId";
import type { FeedMeta } from "../components/PostItem";

export function useFeeds() {
  return useQuery({
    queryKey: feedKeys.all,
    queryFn: getFeeds,
    // Don't refetch on every mount (settings/sidebar remount often). Mutations
    // — create/update/delete/poll — invalidate this key, which refetches even
    // under `Infinity`. Only an autonomous background poll won't be reflected
    // until the next user action or reload.
    staleTime: Infinity,
  });
}

/**
 * Object URL for a feed's icon, or `undefined` while it loads / when the
 * feed has none (render the initials fallback then). The blob is fetched
 * with auth once per feed and shared by every post row via the query cache;
 * the key sits outside `feedKeys.all` so feed mutations don't refetch icons,
 * except a URL edit, which evicts it explicitly — see
 * `evictFeedIconIfUrlChanged`.
 */
export function useFeedIcon(feed?: FeedMeta): string | undefined {
  const { data } = useQuery({
    queryKey: feedIconKey(feed?.id),
    queryFn: () => getFeedIcon(feed!.id).then(URL.createObjectURL),
    enabled: !!feed?.hasIcon,
    // Object URLs stay valid for the whole session; never refetch or evict
    // (eviction would leak the URL without revoking it).
    staleTime: Infinity,
    gcTime: Infinity,
  });
  return data;
}

export function useCreateFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateFeed) => createFeed(body),
    onSuccess: (_feed, body) => {
      qc.invalidateQueries({ queryKey: feedKeys.all });
      // Submitting a folder name can resolve-or-create a folder server-side;
      // without one the folder list can't have changed.
      if (body.folder) qc.invalidateQueries({ queryKey: folderKeys.all });
      const refresh = () => {
        qc.invalidateQueries({ queryKey: postKeys.all });
        // Unread counts live on the feeds query; refetch once the poll lands.
        qc.invalidateQueries({ queryKey: feedKeys.all });
      };
      pollFeeds().then(refresh).catch(refresh);
    },
  });
}

export function useUpdateFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, body }: { id: FeedId; body: UpdateFeed }) =>
      updateFeed(id, body),
    onSuccess: (_feed, { id, body }) => {
      evictFeedIconIfUrlChanged(qc, id, body.url);
      qc.invalidateQueries({ queryKey: feedKeys.all });
      // Only assigning a folder name can create one; clearing (null/blank)
      // or leaving it untouched never changes the folder list.
      if (body.folder) qc.invalidateQueries({ queryKey: folderKeys.all });
    },
  });
}

// Pause/resume is a one-field toggle: patch the cached list with the value the
// response carried back instead of refetching every feed.
export function useSetFeedPaused() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, paused }: { id: FeedId; paused: boolean }) =>
      updateFeed(id, { title: null, url: null, ttl: null, filters: null, display_mode: null, is_paused: paused }),
    onSuccess: (feed) => setFeedPauseState(qc, feed.id, feed.is_paused),
  });
}

export function useDeleteFeed() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: FeedId) => deleteFeed(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: feedKeys.all }),
  });
}

export function usePollFeeds() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: pollFeeds,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: feedKeys.all });
      qc.invalidateQueries({ queryKey: postKeys.all });
    },
  });
}
