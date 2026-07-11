import { useState } from "react";
import { Link } from "react-router-dom";
import { TbChecks, TbCircleOff, TbPencil, TbX } from "react-icons/tb";
import { useDeleteFeed } from "../hooks/useFeeds";
import { useMarkRead } from "../hooks/usePostMutations";
import type { FeedView } from "../types/FeedView";
import type { FeedId } from "../types/FeedId";
import { EditFeedForm } from "./EditFeedForm";
import { RowMenu } from "./RowMenu";

interface Props {
  feed: FeedView;
  active: boolean;
  excluded: boolean;
  onToggleExclude: (id: FeedId) => void;
}

export function FeedItem({ feed, active, excluded, onToggleExclude }: Props) {
  const deleteFeed = useDeleteFeed();
  const markRead = useMarkRead();
  const [editing, setEditing] = useState(false);

  if (editing) {
    return <EditFeedForm feed={feed} onClose={() => setEditing(false)} />;
  }

  return (
    <Link
      to={`/feeds/${feed.id}`}
      className={`feed-item${active ? " active" : ""}${excluded ? " excluded" : ""}`}
    >
      <span className="feed-item-title">{feed.title}</span>
      <RowMenu
        label={`Actions for ${feed.title}`}
        items={[
          {
            icon: <TbCircleOff />,
            label: excluded ? "Show posts" : "Mute posts",
            onSelect: () => onToggleExclude(feed.id),
          },
          {
            icon: <TbChecks />,
            label: "Mark as read",
            onSelect: () => markRead.mutate({ feed_id: feed.id, folder_id: null }),
          },
          {
            icon: <TbPencil />,
            label: "Edit",
            onSelect: () => setEditing(true),
          },
          {
            icon: <TbX />,
            label: "Remove",
            confirmLabel: "Confirm remove?",
            danger: true,
            onSelect: () => deleteFeed.mutate(feed.id),
          },
        ]}
      />
      {feed.unread_count > 0 && (
        <span className="feed-item-unread" title={`${feed.unread_count} unread`}>
          {feed.unread_count > 999 ? "999+" : feed.unread_count}
        </span>
      )}
    </Link>
  );
}
