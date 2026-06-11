import { useEffect, useMemo, useState } from "react";
import { useLocation, useOutletContext, useParams } from "react-router-dom";
import { TbRefresh } from "react-icons/tb";
import { useFavoritePosts, usePosts } from "../hooks/usePosts";
import { useFeeds, usePollFeeds } from "../hooks/useFeeds";
import { PostItem, type FeedMeta } from "./PostItem";
import { VirtualGroupedList } from "./VirtualGroupedList";
import type { ReaderOutletContext } from "../pages/ReaderPage";
import { postPath } from "../utils/posts";

export function PostList() {
  const { excludedFeeds, activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();

  const { pathname } = useLocation();
  const isFavorites = pathname.startsWith("/favorites");

  const { feedId, postId } = useParams();
  const feedIdNum = feedId ? Number(feedId) : undefined;

  const { data: feeds } = useFeeds();
  const pollFeeds = usePollFeeds();
  const favoritePosts = useFavoritePosts();

  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");

  const [activeTag, setActiveTag] = useState<string | undefined>(undefined);

  // Reset activeTag on navigation. setState during render (vs. useEffect) avoids rendering once
  // with a stale tag before the reset kicks in.
  const [prevFeedId, setPrevFeedId] = useState(feedId);
  const [prevIsFavorites, setPrevIsFavorites] = useState(isFavorites);
  if (prevFeedId !== feedId || prevIsFavorites !== isFavorites) {
    setPrevFeedId(feedId);
    setPrevIsFavorites(isFavorites);
    setActiveTag(undefined);
  }

  // Delay search query until the user stops typing
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search), 300);
    return () => clearTimeout(timer);
  }, [search]);

  const showTagChips = !feedIdNum && !isFavorites;
  const feedPosts = usePosts(feedIdNum, showTagChips ? activeTag : undefined, debouncedSearch || undefined);
  const {
    data,
    isLoading,
    hasNextPage,
    fetchNextPage,
    isFetchingNextPage,
    isRefetching,
  } = isFavorites ? favoritePosts : feedPosts;

  // Keep the same FeedMeta object references between renders so React.memo on PostItem can bail out
  const feedMap = useMemo(
    () =>
      new Map(
        (feeds ?? []).map((f): [number, FeedMeta] => [
          f.id,
          { name: f.title, tag: f.tag, tagColor: f.tag_color },
        ])
      ),
    [feeds]
  );

  // Same — avoid recreating tag arrays on every render
  const { tagMap, tags } = useMemo(() => {
    const tagMap = new Map(
      (feeds ?? [])
        .filter((f) => f.tag !== undefined)
        .map((f) => [f.tag!, f.tag_color])
    );
    return { tagMap, tags: [...tagMap.keys()] };
  }, [feeds]);

  useEffect(() => {
    if (postId) onOpenPost(Number(postId));
  }, [postId, onOpenPost]);

  const items = useMemo(() => {
    const posts = data?.pages.flatMap((p) => p.items);
    return posts?.filter(
      (p) => isFavorites || feedIdNum !== undefined || !excludedFeeds.has(p.feed_id)
    );
  }, [data, excludedFeeds, isFavorites, feedIdNum]);

  return (
    <div className="pane pane-posts">
      <div className="posts-toolbar">
        <input
          className="posts-search"
          placeholder="Search posts…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <button
          className="posts-refresh-btn"
          onClick={() => pollFeeds.mutate()}
          disabled={pollFeeds.isPending || isRefetching || isLoading}
          title="Refresh"
        >
          <TbRefresh className={pollFeeds.isPending || isRefetching ? "spinning" : ""} />
        </button>
      </div>
      {showTagChips && tags.length > 0 && (
        <div className="posts-tags">
          {tags.map((tag) => (
            <button
              key={tag}
              className={`posts-tag-chip${activeTag === tag ? " active" : ""}`}
              style={{ "--tag-color": tagMap.get(tag) ?? "var(--text)" } as React.CSSProperties}
              onClick={() => setActiveTag((prev) => (prev === tag ? undefined : tag))}
            >
              {tag}
            </button>
          ))}
        </div>
      )}
      <VirtualGroupedList
        items={items}
        isLoading={isLoading}
        hasNextPage={hasNextPage}
        fetchNextPage={fetchNextPage}
        isFetchingNextPage={isFetchingNextPage}
        emptyMessage={isFavorites ? "No favorites yet" : "No posts"}
        renderItem={(post) => (
          <PostItem
            post={post}
            to={postPath(post, feedIdNum, isFavorites)}
            active={activePostId === post.id}
            feed={feedIdNum === undefined ? feedMap.get(post.feed_id) : undefined}
          />
        )}
      />
    </div>
  );
}
