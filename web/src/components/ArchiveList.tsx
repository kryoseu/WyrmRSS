import { useEffect } from "react";
import { useOutletContext, useParams } from "react-router-dom";
import { useArchivedPosts } from "../hooks/usePosts";
import { ArchiveItem } from "./ArchiveItem";
import type { ReaderOutletContext } from "../pages/ReaderPage";
import { groupByDate } from "../utils/posts";

export function ArchiveList() {
  const { activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();
  const { postId } = useParams();

  const { data, isLoading, hasNextPage, fetchNextPage, isFetchingNextPage } = useArchivedPosts();

  useEffect(() => {
    if (postId) onOpenPost(Number(postId));
  }, [postId, onOpenPost]);

  const archives = data?.pages.flatMap((p) => p.items) ?? [];
  const groups = groupByDate(archives);

  return (
    <div className="pane pane-posts">
      <div className="posts-toolbar">
        <span className="posts-toolbar-title">Archive</span>
      </div>
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
