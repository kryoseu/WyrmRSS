import { useState } from "react";
import { Link } from "react-router-dom";
import { TbCircleOff, TbPencil, TbX } from "react-icons/tb";
import { useDeleteFeed } from "../hooks/useFeeds";
import type { FeedView } from "../types/FeedView";
import type { FeedId } from "../types/FeedId";
import { EditFeedForm } from "./EditFeedForm";

interface Props {
  feed: FeedView;
  active: boolean;
  excluded: boolean;
  onToggleExclude: (id: FeedId) => void;
}

export function FeedItem({ feed, active, excluded, onToggleExclude }: Props) {
  const deleteFeed = useDeleteFeed();
  const [editing, setEditing] = useState(false);

  function handleDelete(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    deleteFeed.mutate(feed.id);
  }

  function handleEdit(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    setEditing(true);
  }

  function handleFilter(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    onToggleExclude(feed.id);
  }

  if (editing) {
    return <EditFeedForm feed={feed} onClose={() => setEditing(false)} />;
  }

  return (
    <Link
      to={`/feeds/${feed.id}`}
      className={`feed-item${active ? " active" : ""}${excluded ? " excluded" : ""}`}
    >
      <span className="feed-item-title">{feed.title}</span>
      <button
        className={`feed-item-filter${excluded ? " active" : ""}`}
        onClick={handleFilter}
        title={excluded ? "Show feed posts" : "Hide feed posts"}
        aria-label={excluded ? `Show ${feed.title} posts` : `Hide ${feed.title} posts`}
      >
        <TbCircleOff />
      </button>
      <button
        className="feed-item-edit"
        onClick={handleEdit}
        title="Edit feed"
        aria-label={`Edit ${feed.title}`}
      >
        <TbPencil />
      </button>
      <button
        className="feed-item-delete"
        onClick={handleDelete}
        title="Remove feed"
        aria-label={`Remove ${feed.title}`}
      >
        <TbX />
      </button>
      {feed.tag && (
        <span
          className="feed-item-tag"
          style={feed.tag_color ? ({ '--tag-color': feed.tag_color } as React.CSSProperties) : undefined}
        >
          {feed.tag}
        </span>
      )}

      {feed.unread_count > 0 && (
        <span className="feed-item-unread" title={`${feed.unread_count} unread`}>
          {feed.unread_count > 999 ? "999+" : feed.unread_count}
        </span>
      )}
    </Link>
  );
}
