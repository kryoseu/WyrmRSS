import { useState } from "react";
import { Link } from "react-router";
import { TbChecks, TbPencil, TbPlayerPause, TbPlayerPlay, TbX } from "react-icons/tb";
import { useDeleteFeed, useSetFeedPaused } from "../hooks/useFeeds";
import { useMarkRead } from "../hooks/usePostMutations";
import type { FeedView } from "../types/FeedView";
import { EditFeedForm } from "./EditFeedForm";
import { RowMenu } from "./RowMenu";

interface Props {
  feed: FeedView;
  active: boolean;
}

export function FeedItem({ feed, active }: Props) {
  const deleteFeed = useDeleteFeed();
  const setPaused = useSetFeedPaused();
  const markRead = useMarkRead();
  const [editing, setEditing] = useState(false);

  if (editing) {
    return <EditFeedForm feed={feed} onClose={() => setEditing(false)} />;
  }

  return (
    <Link
      to={`/feeds/${feed.id}`}
      className={`feed-item${active ? " active" : ""}${feed.is_paused ? " paused" : ""}`}
    >
      <span className="feed-item-title">{feed.title}</span>
      <RowMenu
        label={`Actions for ${feed.title}`}
        items={[
          {
            icon: feed.is_paused ? <TbPlayerPlay /> : <TbPlayerPause />,
            label: feed.is_paused ? "Resume feed" : "Pause feed",
            onSelect: () => setPaused.mutate({ id: feed.id, paused: !feed.is_paused }),
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
      {feed.display_mode !== "radar" && feed.unread_count > 0 && (
        <span className="feed-item-unread" title={`${feed.unread_count} unread`}>
          {feed.unread_count > 999 ? "999+" : feed.unread_count}
        </span>
      )}
    </Link>
  );
}
