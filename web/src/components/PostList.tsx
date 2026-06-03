import { useEffect, useState } from "react";
import { useLocation, useOutletContext, useParams } from "react-router-dom";
import { TbRefresh } from "react-icons/tb";
import { useFavoritePosts, usePosts } from "../hooks/usePosts";
import { useFeeds, usePollFeeds } from "../hooks/useFeeds";
import { PostItem, type FeedMeta } from "./PostItem";
import type { ReaderOutletContext } from "../pages/ReaderPage";
import { groupByDate, postPath } from "../utils/posts";

export function PostList() {
  const { excludedFeeds, activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();

  const { pathname } = useLocation();
  const isFavorites = pathname.startsWith("/favorites");

  const { feedId, postId } = useParams();
  const feedIdNum = feedId ? Number(feedId) : undefined;

  const { data: feeds } = useFeeds();

  const pollMutation = usePollFeeds();
  const favoritesQuery = useFavoritePosts();

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
    const timer = setTimeout(() => setDebouncedSearch(search || ""), 300);
    return () => clearTimeout(timer);
  }, [search]);

  const showTagChips = !feedIdNum && !isFavorites;
  const postsQuery = usePosts(feedIdNum, showTagChips ? activeTag : undefined, debouncedSearch || undefined);
  const {
    data,
    isLoading,
    hasNextPage,
    fetchNextPage,
    isFetchingNextPage,
    isRefetching
  } = isFavorites ? favoritesQuery : postsQuery;

  const feedMap = new Map(
    (feeds ?? []).map((f): [number, FeedMeta] => [
      f.id,
      { name: f.title, tag: f.tag, tagColor: f.tag_color },
    ])
  );

  const tagMap = new Map(
    (feeds ?? [])
      .filter((f) => f.tag !== undefined)
      .map((f) => [f.tag!, f.tag_color])
  );
  const tags = [...tagMap.keys()];

  useEffect(() => {
    if (postId) onOpenPost(Number(postId));
  }, [postId, onOpenPost]);

  const posts = data?.pages.flatMap((p) => p.items);
  // Filter out posts whose feed.id are in the excluded list, 
  // unless we're on the favorites or on a feed page.
  const filtered = posts?.filter((p) =>
    isFavorites || feedIdNum !== undefined || !excludedFeeds.has(p.feed_id)
  );
  const groups = filtered ? groupByDate(filtered) : [];

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
          onClick={() => pollMutation.mutate()}
          disabled={pollMutation.isPending || isRefetching || isLoading}
          title="Refresh"
        >
          <TbRefresh className={pollMutation.isPending || isRefetching ? "spinning" : ""} />
        </button>
      </div>
      {showTagChips && tags.length > 0 && (
        <div className="posts-tags">
          {tags.map((tag) => (
            <button
              key={tag}
              className={`posts-tag-chip${activeTag === tag ? " active" : ""}`}
              style={{ '--tag-color': tagMap.get(tag) ?? 'var(--text)' } as React.CSSProperties}
              onClick={() => setActiveTag((prev) => (prev === tag ? undefined : tag))}
            >
              {tag}
            </button>
          ))}
        </div>
      )}
      <div className="pane-scroll">
        {isLoading && <div className="pane-empty">Loading…</div>}
        {!isLoading && groups.length === 0 && (
          <div className="pane-empty">{isFavorites ? "No favorites yet" : "No posts"}</div>
        )}
        {groups.map(([label, groupPosts]) => (
          <div key={label} className="posts-group">
            <div className="posts-group-label">
              <span>{label}</span>
              <span className="posts-group-count">{groupPosts.length}</span>
            </div>
            <div className="posts-group-items">
              {groupPosts.map((p) => (
                <PostItem
                  key={p.id}
                  post={p}
                  to={postPath(p, feedIdNum, isFavorites)}
                  active={activePostId === p.id}
                  feed={feedIdNum === undefined ? feedMap.get(p.feed_id) : undefined}
                />
              ))}
            </div>
          </div>
        ))}
        {hasNextPage && (
          <div className="load-more">
            <button
              className="load-more-btn"
              onClick={() => fetchNextPage()}
              disabled={isFetchingNextPage}
            >
              {isFetchingNextPage ? "Loading…" : "Load more"}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
