import { memo } from "react";
import { Link } from "react-router-dom";
import { HiMail, HiMailOpen } from "react-icons/hi";
import { TbArchive, TbArchiveOff, TbStar, TbStarFilled } from "react-icons/tb";
import type { Post } from "../types/Post";
import { useArchivePost, useUnarchivePost, useUpdatePost } from "../hooks/usePostMutations";
import { useSettings } from "../hooks/useSettings";
import { initials } from "../utils/posts";

export interface FeedMeta {
  name: string;
  tag: string | undefined;
  tagColor: string | undefined;
}

interface Props {
  post: Post;
  to: string;
  active: boolean;
  feed?: FeedMeta;
}

export const PostItem = memo(function PostItem({ post, to, active, feed }: Props) {
  const { mutate: updatePost } = useUpdatePost();
  const { mutate: archivePost } = useArchivePost();
  const { mutate: unarchivePost } = useUnarchivePost();
  const { data: settings } = useSettings();
  const readMode = settings?.read_mode ?? "on_open";
  const isRead = readMode === "disabled" ? true : post.is_read;

  function handleReadToggle(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    updatePost({ id: post.id, is_read: !post.is_read, is_favorite: null });
  }

  function handleFavToggle(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    updatePost({ id: post.id, is_favorite: !post.is_favorite, is_read: null });
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

  const favoriteIcon = post.is_favorite ? <TbStarFilled /> : <TbStar />;
  const favoriteLabel = post.is_favorite ? "Unfavorite" : "Favorite";

  const tagColor = feed?.tagColor ? ({ '--tag-color': feed.tagColor } as React.CSSProperties) : undefined;

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
      {feed?.tag && (
        <span
          className="post-item-tag"
          style={tagColor}
        >
          {feed.tag}
        </span>
      )}
      <span className="post-item-title">{post.title ?? "Untitled"}</span>
      <button
        className={`post-item-archive${post.is_archived ? " archived" : ""}`}
        onClick={handleArchiveToggle}
        aria-label={archiveLabel}
      >
        {archiveIcon}
      </button>
      <button
        className={`post-item-fav${post.is_favorite ? " favorited" : ""}`}
        onClick={handleFavToggle}
        aria-label={favoriteLabel}
      >
        {favoriteIcon}
      </button>
    </Link>
  );
});
