import { useEffect, useState } from "react";
import { useLocation, useOutletContext, useParams } from "react-router-dom";
import { TbRefresh } from "react-icons/tb";
import { useFavoritePosts, usePosts } from "../hooks/usePosts";
import { useFeeds, usePollFeeds } from "../hooks/useFeeds";
import { PostItem } from "./PostItem";
import type { Post } from "../types/Post";
import type { ReaderOutletContext } from "../pages/ReaderPage";

function getDateLabel(iso: string): string {
  const date = new Date(iso);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);

  const fmt = (d: Date) =>
    d.toLocaleDateString(undefined, { month: "long", day: "numeric", year: "numeric" });

  if (date.toDateString() === today.toDateString()) return `Today · ${fmt(date)}`;
  if (date.toDateString() === yesterday.toDateString()) return `Yesterday · ${fmt(date)}`;
  return fmt(date);
}

function postPath(p: Post, feedId?: number, isFavorites?: boolean): string {
  if (isFavorites) return `/favorites/${p.id}`;
  if (feedId !== undefined) return `/feeds/${p.feed_id}/posts/${p.id}`;
  return `/feeds/posts/${p.id}`;
}

function groupByDate(posts: Post[]): [string, Post[]][] {
  const map = new Map<string, Post[]>();
  for (const post of posts) {
    const label = getDateLabel(post.published_at);
    if (!map.has(label)) map.set(label, []);
    map.get(label)!.push(post);
  }
  return Array.from(map.entries());
}

export function PostList() {
  const { excludedFeeds, activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();
  const { pathname } = useLocation();
  const isFavorites = pathname.startsWith("/favorites");
  const { feedId, postId } = useParams();
  const [search, setSearch] = useState("");
  const { data: feeds } = useFeeds();
  const feedIdNum = feedId ? Number(feedId) : undefined;
  const pollMutation = usePollFeeds();
  const postsQuery = usePosts(feedIdNum);
  const favoritesQuery = useFavoritePosts();
  const {
    data,
    isLoading,
    hasNextPage,
    fetchNextPage,
    isFetchingNextPage,
    isRefetching
  } = isFavorites ? favoritesQuery : postsQuery;
  const feedMap = feeds ? new Map(feeds.map((f) => [f.id, f.title])) : new Map<number, string>();

  useEffect(() => {
    if (postId) onOpenPost(Number(postId));
  }, [postId, onOpenPost]);

  const posts = data?.pages.flatMap((p) => p.items);
  const filtered = posts?.filter(
    (p) =>
      (!search || (p.title ?? "").toLowerCase().includes(search.toLowerCase())) &&
      (isFavorites || feedIdNum !== undefined || !excludedFeeds.has(p.feed_id))
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
                  feedName={feedIdNum === undefined ? feedMap.get(p.feed_id) : undefined}
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
