import { useMemo, useState } from "react";
import { useOutletContext, useParams } from "react-router-dom";
import { TbRefresh } from "react-icons/tb";
import { usePosts } from "../hooks/usePosts";
import { useFeeds, usePollFeeds, useFeedTags } from "../hooks/useFeeds";
import { useDebouncedSearch } from "../hooks/useDebouncedSearch";
import { useOpenPostFromRoute } from "../hooks/useOpenPostFromRoute";
import { useFeedMap } from "../hooks/useFeedMap";
import { useFlattenedPages } from "../hooks/useFlattenedPages";
import { PostItem } from "./PostItem";
import { PostsToolbar } from "./PostsToolbar";
import { PostsTagChips } from "./PostsTagChips";
import { VirtualGroupedList } from "./VirtualGroupedList";
import type { ReaderOutletContext } from "../pages/ReaderPage";
import type { FeedId } from "../types/FeedId";
import { postPath } from "../utils/posts";

export function PostList() {
  const { excludedFeeds, activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();

  const { feedId, postId } = useParams();

  const feedIdNum: FeedId | undefined = feedId ? Number(feedId) : undefined;

  const { data: feeds } = useFeeds();

  const pollFeeds = usePollFeeds();

  const { search, setSearch, debouncedSearch } = useDebouncedSearch();

  const [activeTag, setActiveTag] = useState<string | undefined>(undefined);

  // Reset activeTag on navigation. setState during render (vs. useEffect) avoids rendering once
  // with a stale tag before the reset kicks in.
  const [prevFeedId, setPrevFeedId] = useState(feedId);
  if (prevFeedId !== feedId) {
    setPrevFeedId(feedId);
    setActiveTag(undefined);
  }

  const showTagChips = !feedIdNum;

  const {
    data,
    isLoading,
    hasNextPage,
    fetchNextPage,
    isFetchingNextPage,
    isRefetching }
    = usePosts({
      feedId: feedIdNum,
      tag: showTagChips ? activeTag : undefined,
      search: debouncedSearch || undefined
    });

  useOpenPostFromRoute(postId, onOpenPost);

  const feedMap = useFeedMap(feeds);
  const tags = useFeedTags(feeds);

  const allPosts = useFlattenedPages(data);
  const posts = useMemo(
    () =>
      allPosts?.filter(
        (p) => feedIdNum !== undefined || !excludedFeeds.has(p.feed_id)
      ),
    [allPosts, excludedFeeds, feedIdNum]
  );

  return (
    <div className="pane pane-posts">
      <PostsToolbar value={search} onChange={setSearch}>
        <button
          className="posts-refresh-btn"
          onClick={() => pollFeeds.mutate()}
          disabled={pollFeeds.isPending || isRefetching || isLoading}
          title="Refresh"
        >
          <TbRefresh className={pollFeeds.isPending || isRefetching ? "spinning" : ""} />
        </button>
      </PostsToolbar>
      {showTagChips && (
        <PostsTagChips
          tags={tags}
          activeTag={activeTag}
          onToggle={(tag) => setActiveTag((prev) => (prev === tag ? undefined : tag))}
        />
      )}
      <VirtualGroupedList
        items={posts}
        isLoading={isLoading}
        hasNextPage={hasNextPage}
        fetchNextPage={fetchNextPage}
        isFetchingNextPage={isFetchingNextPage}
        emptyMessage="No posts"
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
