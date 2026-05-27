import { Link } from "react-router-dom";
import type { Post } from "../types/Post";
import { useToggleFavorite } from "../hooks/usePosts";

interface Props {
  post: Post;
  to: string;
  active: boolean;
  feedName?: string;
}

function initials(name: string): string {
  const words = name.trim().split(/\s+/).filter((w) => /^[a-zA-Z0-9]/.test(w));
  if (words.length === 0) return "?";
  if (words.length === 1) return words[0].replace(/[^a-zA-Z0-9]/g, "").slice(0, 2).toUpperCase();
  return (words[0][0] + words[1][0]).toUpperCase();
}

export function PostItem({ post, to, active, feedName }: Props) {
  const { mutate: toggleFavorite } = useToggleFavorite();

  return (
    <Link to={to} className={`post-item${active ? " active" : ""}`}>
      {feedName && <span className="post-item-feed">{initials(feedName)}</span>}
      <span className="post-item-title">{post.title ?? "Untitled"}</span>
      <button
        className={`post-item-fav${post.is_favorite ? " favorited" : ""}`}
        onClick={(e) => { e.preventDefault(); e.stopPropagation(); toggleFavorite(post.id); }}
        aria-label={post.is_favorite ? "Unfavorite" : "Favorite"}
      >
        {post.is_favorite ? "★" : "☆"}
      </button>
    </Link>
  );
}
