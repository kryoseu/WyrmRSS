import { Link } from "react-router-dom";
import { HiMail, HiMailOpen } from "react-icons/hi";
import { TbStar, TbStarFilled } from "react-icons/tb";
import type { Post } from "../types/Post";
import { useUpdatePost } from "../hooks/usePosts";
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

export function PostItem({ post, to, active, feed }: Props) {
  const { mutate: updatePost } = useUpdatePost();

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

  return (
    <Link to={to} className={`post-item${active ? " active" : ""}${post.is_read ? " read" : ""}`}>
      <button
        className={`post-item-read${!post.is_read ? " unread" : ""}`}
        onClick={handleReadToggle}
        aria-label={post.is_read ? "Mark as unread" : "Mark as read"}
      >
        {post.is_read ? <HiMailOpen /> : <HiMail />}
      </button>
      {feed && <span className="post-item-feed">{initials(feed.name)}</span>}
      {feed?.tag && (
        <span
          className="post-item-tag"
          style={feed.tagColor ? ({ '--tag-color': feed.tagColor } as React.CSSProperties) : undefined}
        >
          {feed.tag}
        </span>
      )}
      <span className="post-item-title">{post.title ?? "Untitled"}</span>
      <button
        className={`post-item-fav${post.is_favorite ? " favorited" : ""}`}
        onClick={handleFavToggle}
        aria-label={post.is_favorite ? "Unfavorite" : "Favorite"}
      >
        {post.is_favorite ? <TbStarFilled /> : <TbStar />}
      </button>
    </Link>
  );
}
