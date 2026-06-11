import { useEffect, useMemo, useState } from "react";
import { useOutletContext, useParams } from "react-router-dom";
import { useArchivedPosts } from "../hooks/usePosts";
import { ArchiveItem } from "./ArchiveItem";
import { VirtualGroupedList } from "./VirtualGroupedList";
import type { ReaderOutletContext } from "../pages/ReaderPage";

export function ArchiveList() {
  const { activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();
  const { postId } = useParams();

  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [activeTag, setActiveTag] = useState<string | undefined>(undefined);

  // Delay search query until the user stops typing
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search), 300);
    return () => clearTimeout(timer);
  }, [search]);

  // Data kept in cache for tag chip discovery.
  // PostList doesn't need this because tags are derived from feeds.
  const { data: cachedData } = useArchivedPosts();
  const { data, isLoading, hasNextPage, fetchNextPage, isFetchingNextPage } =
    useArchivedPosts(debouncedSearch || undefined, activeTag);

  useEffect(() => {
    if (postId) onOpenPost(Number(postId));
  }, [postId, onOpenPost]);

  const items = useMemo(
    () => data?.pages.flatMap((p) => p.items),
    [data]
  );

  const { tagMap, tagKeys } = useMemo(() => {
    const tagMap = new Map<string, string | undefined>();
    for (const page of cachedData?.pages ?? []) {
      for (const item of page.items) {
        if (item.tag) tagMap.set(item.tag, item.tag_color ?? undefined);
      }
    }
    return { tagMap, tagKeys: [...tagMap.keys()] };
  }, [cachedData]);

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
        emptyMessage="No archived posts"
        renderItem={(archive) => (
          <ArchiveItem
            archive={archive}
            active={activePostId === archive.id}
          />
        )}
      />
    </div>
  );
}
