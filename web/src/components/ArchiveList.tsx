import { useEffect, useMemo, useState } from "react";
import { useOutletContext, useParams } from "react-router-dom";
import { useArchivedPosts } from "../hooks/usePosts";
import { ArchiveItem } from "./ArchiveItem";
import type { ReaderOutletContext } from "../pages/ReaderPage";
import { groupByDate } from "../utils/posts";

export function ArchiveList() {
  const { activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();
  const { postId } = useParams();

  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [activeTag, setActiveTag] = useState<string | undefined>(undefined);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search || ""), 300);
    return () => clearTimeout(timer);
  }, [search]);

  // Data kept in cache for tag chip discovery.
  // PostList doesn't need because tags are derived from feeds.
  const { data: cachedData } = useArchivedPosts();
  const { data, isLoading, hasNextPage, fetchNextPage, isFetchingNextPage } =
    useArchivedPosts(debouncedSearch || undefined, activeTag);

  useEffect(() => {
    if (postId) onOpenPost(Number(postId));
  }, [postId, onOpenPost]);

  const archives = useMemo(() => data?.pages.flatMap((p) => p.items) ?? [], [data]);
  const groups = groupByDate(archives);

  const tagMap = useMemo(() => {
    const map = new Map<string, string | undefined>();
    for (const page of cachedData?.pages ?? []) {
      for (const item of page.items) {
        if (item.tag) map.set(item.tag, item.tag_color ?? undefined);
      }
    }
    return map;
  }, [cachedData]);
  const tagKeys = [...tagMap.keys()];

  return (
    <div className="pane pane-posts">
      <div className="posts-toolbar">
        <input
          className="posts-search"
          placeholder="Search archive…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>
      {tagKeys.length > 0 && (
        <div className="posts-tags">
          {tagKeys.map((tag) => (
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
        {!isLoading && archives.length === 0 && (
          <div className="pane-empty">No archived posts</div>
        )}
        {groups.map(([label, items]) => (
          <div key={label} className="posts-group">
            <div className="posts-group-label">
              <span>{label}</span>
              <span className="posts-group-count">{items.length}</span>
            </div>
            <div className="posts-group-items">
              {items.map((a) => (
                <ArchiveItem
                  key={a.id}
                  archive={a}
                  active={activePostId === a.id}
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
