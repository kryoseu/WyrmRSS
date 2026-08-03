import { useOutletContext, useParams } from "react-router";
import { TbChecks, TbEye, TbEyeOff, TbRefresh } from "react-icons/tb";
import { usePosts } from "../hooks/usePosts";
import { useUnreadOnly } from "../hooks/useUnreadOnly";
import { useFeeds, usePollFeeds } from "../hooks/useFeeds";
import { useMarkRead } from "../hooks/usePostMutations";
import { useDebouncedSearch } from "../hooks/useDebouncedSearch";
import { useOpenPostFromRoute } from "../hooks/useOpenPostFromRoute";
import { useFeedMap } from "../hooks/useFeedMap";
import { useFlattenedPages } from "../hooks/useFlattenedPages";
import { PostItem } from "./PostItem";
import { PostsToolbar } from "./PostsToolbar";
import { VirtualGroupedList } from "./VirtualGroupedList";
import type { ReaderOutletContext } from "../pages/ReaderPage";
import type { FeedId } from "../types/FeedId";
import type { Post } from "../types/Post";
import { postPath } from "../utils/posts";

const groupDate = (post: Post) => post.created_at;

export function PostList() {
  const { activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();

  const { feedId, postId } = useParams();

  const feedIdNum: FeedId | undefined = feedId ? Number(feedId) : undefined;

  const { data: feeds } = useFeeds();

  const pollFeeds = usePollFeeds();

  const markRead = useMarkRead();

  const { search, setSearch, debouncedSearch } = useDebouncedSearch();

  const { unreadOnly, setUnreadOnly } = useUnreadOnly();

  const {
    data,
    isLoading,
    hasNextPage,
    fetchNextPage,
    isFetchingNextPage,
    isRefetching }
    = usePosts({
      feed_id: feedIdNum,
      search: debouncedSearch || undefined,
      // unread-only is the API default; only "all" mode needs the param
      unread_only: unreadOnly ? undefined : false,
    });

  useOpenPostFromRoute(postId, onOpenPost);

  const feedMap = useFeedMap(feeds);

  const posts = useFlattenedPages(data);

  return (
    <div className="pane pane-posts">
      <PostsToolbar value={search} onChange={setSearch}>
        <button
          className="posts-refresh-btn"
          aria-pressed={!unreadOnly}
          onClick={() => setUnreadOnly(!unreadOnly)}
          title={unreadOnly ? "Unread only — click to show all posts" : "All posts — click to show unread only"}
        >
          {unreadOnly ? <TbEyeOff /> : <TbEye />}
        </button>
        <button
          className="posts-refresh-btn"
          onClick={() => markRead.mutate({ feed_id: feedIdNum ?? null, folder_id: null })}
          disabled={markRead.isPending}
          title={feedIdNum !== undefined ? "Mark feed as read" : "Mark all as read"}
        >
          <TbChecks />
        </button>
        <button
          className="posts-refresh-btn"
          onClick={() => pollFeeds.mutate()}
          disabled={pollFeeds.isPending || isRefetching || isLoading}
          title="Refresh"
        >
          <TbRefresh className={pollFeeds.isPending || isRefetching ? "spinning" : ""} />
        </button>
      </PostsToolbar>
      <VirtualGroupedList
        items={posts}
        isLoading={isLoading}
        hasNextPage={hasNextPage}
        fetchNextPage={fetchNextPage}
        isFetchingNextPage={isFetchingNextPage}
        groupDate={groupDate}
        emptyMessage={unreadOnly && !debouncedSearch ? "All caught up" : "No posts"}
        renderItem={(post) => (
          <PostItem
            post={post}
            to={postPath(post, feedIdNum, false)}
            active={activePostId === post.id}
            feed={feedIdNum === undefined ? feedMap.get(post.feed_id) : undefined}
          />
        )}
      />
    </div>
  );
}
