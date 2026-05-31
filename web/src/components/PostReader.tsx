import { useEffect } from "react";
import YouTube from "react-youtube";
import { TbStar, TbStarFilled } from "react-icons/tb";
import { usePost, useUpdatePost } from "../hooks/usePosts";

function extractYouTubeId(url: string | null): string | null {
  if (!url) return null;
  try {
    const u = new URL(url);
    if (u.hostname === "www.youtube.com" || u.hostname === "youtube.com") {
      return u.searchParams.get("v");
    }
    if (u.hostname === "youtu.be") {
      return u.pathname.slice(1) || null;
    }
  } catch {
    // not a valid URL
  }
  return null;
}

function formatDate(iso: string | null): string {
  if (!iso) return "";
  return new Date(iso).toLocaleDateString(undefined, {
    month: "long",
    day: "numeric",
    year: "numeric",
  });
}

interface Props {
  postId: number | null;
  onClose: () => void;
  width: number;
}

export function PostReader({ postId, onClose, width }: Props) {
  const { data: post, isLoading } = usePost(postId ?? undefined);
  const { mutate: updatePost } = useUpdatePost();

  useEffect(() => {
    if (post && !post.is_read) {
      updatePost({ id: post.id, is_read: true, is_favorite: null });
    }
  }, [post, updatePost]);

  if (!postId) return null;

  const body = post?.content ?? post?.description;
  const date = formatDate(post?.published_at ?? null);
  const youtubeId = extractYouTubeId(post?.url ?? null);

  return (
    <div className="pane pane-reader" style={{ width }}>
      <div className="pane-reader-header">
        {post && (
          <button
            className={`pane-reader-fav${post.is_favorite ? " favorited" : ""}`}
            onClick={() => updatePost({ id: post.id, is_favorite: !post.is_favorite, is_read: null })}
            aria-label={post.is_favorite ? "Unfavorite" : "Favorite"}
          >
            {post.is_favorite ? <TbStarFilled /> : <TbStar />}
          </button>
        )}
        <button
          className="pane-reader-close"
          onClick={onClose}
          aria-label="Close reader"
        >
          ×
        </button>
      </div>
      <div className="pane-scroll">
        {isLoading && <div className="pane-empty">Loading…</div>}
        {!isLoading && !post && <div className="pane-empty">Post not found</div>}
        {!isLoading && post && (
          <div className="post-reader">
            <h1 className="post-reader-title">{post.title ?? "Untitled"}</h1>
            <div className="post-reader-meta">
              {post.authors && <span>{post.authors}</span>}
              {date && <span>{date}</span>}
              {post.url && (
                <a href={post.url} target="_blank" rel="noopener noreferrer">
                  Open original ↗
                </a>
              )}
            </div>
            {youtubeId && (
              <div className="post-reader-youtube">
                <YouTube videoId={youtubeId} opts={{ width: "100%" }} />
              </div>
            )}
            {body && !youtubeId && (
              <div
                className="post-reader-body"
                dangerouslySetInnerHTML={{ __html: body }}
              />
            )}
            {body && youtubeId && (
              <p className="post-reader-body" style={{ whiteSpace: "pre-wrap" }}>
                {body}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
