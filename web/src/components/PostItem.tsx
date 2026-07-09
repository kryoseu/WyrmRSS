import { memo } from "react";
import { Link } from "react-router-dom";
import { HiMail, HiMailOpen } from "react-icons/hi";
import { TbArchive, TbArchiveOff, TbBookmark, TbBookmarkFilled } from "react-icons/tb";
import type { Post } from "../types/Post";
import { useArchivePost, useUnarchivePost, useSetPostRead, useSetPostBookmarked } from "../hooks/usePostMutations";
import { useSettings } from "../hooks/useSettings";
import { initials } from "../utils/posts";

export interface FeedMeta {
  name: string;
}

interface Props {
  post: Post;
  to: string;
  active: boolean;
  feed?: FeedMeta;
}

export const PostItem = memo(function PostItem({ post, to, active, feed }: Props) {
  const { mutate: setRead } = useSetPostRead();
  const { mutate: setBookmarked } = useSetPostBookmarked();
  const { mutate: archivePost } = useArchivePost();
  const { mutate: unarchivePost } = useUnarchivePost();
  const { data: settings } = useSettings();
  const readMode = settings?.read_mode ?? "on_open";
  const isRead = readMode === "disabled" ? true : post.is_read;

  function handleReadToggle(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    setRead({ id: post.id, isRead: !post.is_read });
  }

  function handleBookmarkToggle(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    setBookmarked({ id: post.id, bookmarked: !post.bookmarked });
  }

  function handleArchiveToggle(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (post.is_archived) {
      unarchivePost(post.id);
    } else {
      archivePost(post.id);
    }
  }

  const mailIcon = post.is_read ? <HiMailOpen /> : <HiMail />;
  const readLabel = post.is_read ? "Mark as unread" : "Mark as read";

  const archiveIcon = post.is_archived ? <TbArchiveOff /> : <TbArchive />;
  const archiveLabel = post.is_archived ? "Unarchive" : "Archive";

  const bookmarkIcon = post.bookmarked ? <TbBookmarkFilled /> : <TbBookmark />;
  const bookmarkLabel = post.bookmarked ? "Remove from read later" : "Read later";

  return (
    <Link to={to} className={`post-item${active ? " active" : ""}${isRead ? " read" : ""}`}>
      {readMode !== "disabled" && (
        <button
          className={`post-item-read${!post.is_read ? " unread" : ""}`}
          onClick={handleReadToggle}
          aria-label={readLabel}
        >
          {mailIcon}
        </button>
      )}
      {feed && <span className="post-item-feed">{initials(feed.name)}</span>}
      <span className="post-item-title">{post.title ?? "Untitled"}</span>
      <button
        className={`post-item-archive${post.is_archived ? " archived" : ""}`}
        onClick={handleArchiveToggle}
        aria-label={archiveLabel}
      >
        {archiveIcon}
      </button>
      <button
        className={`post-item-bookmark${post.bookmarked ? " bookmarked" : ""}`}
        onClick={handleBookmarkToggle}
        aria-label={bookmarkLabel}
      >
        {bookmarkIcon}
      </button>
    </Link>
  );
});
