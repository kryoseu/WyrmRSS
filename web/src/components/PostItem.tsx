import { memo, useMemo } from "react";
import { Link } from "react-router";
import { HiMail, HiMailOpen } from "react-icons/hi";
import { TbArchive, TbArchiveOff, TbBookmark, TbBookmarkFilled } from "react-icons/tb";
import type { Post } from "../types/Post";
import type { FeedId } from "../types/FeedId";
import { useArchivePost, useUnarchivePost, useSetPostRead, useSetPostBookmarked } from "../hooks/usePostMutations";
import { useSettings } from "../hooks/useSettings";
import { useFeedIcon } from "../hooks/useFeeds";
import { useTheme } from "../hooks/useTheme";
import { initials } from "../utils/posts";
import { plainTextSnippet } from "../utils/radar";
import { CONTENT } from "../utils/constants";
import { formatLastFetched } from "../utils/utils";

export interface FeedMeta {
  id: FeedId;
  name: string;
  hasIcon: boolean;
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
  const iconUrl = useFeedIcon(feed);
  const card = useTheme().density === "card";
  const snippet = useMemo(() => {
    const html = post.description ?? post.content;
    return card && html ? plainTextSnippet(html, CONTENT.cardSnippetLength) : "";
  }, [card, post.description, post.content]);
  const readMode = settings?.read_mode;
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

  const title = <span className="post-item-title">{post.title ?? "Untitled"}</span>;

  return (
    <Link
      to={to}
      className={`post-item${card ? " post-item--card" : ""}${active ? " active" : ""}${isRead ? " read" : ""}`}
    >
      {readMode !== "disabled" && (
        <button
          className={`post-item-read${!post.is_read ? " unread" : ""}`}
          onClick={handleReadToggle}
          aria-label={readLabel}
        >
          {mailIcon}
        </button>
      )}
      {feed &&
        (iconUrl ? (
          <img className="post-item-icon" src={iconUrl} alt="" loading="lazy" />
        ) : (
          <span className="post-item-feed">{initials(feed.name)}</span>
        ))}
      {card ? (
        <span className="post-item-main">
          {title}
          {snippet && <span className="post-item-snippet">{snippet}</span>}
          <span className="post-item-meta">
            {feed && `${feed.name} · `}
            {formatLastFetched(post.published_at)}
          </span>
        </span>
      ) : (
        title
      )}
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
