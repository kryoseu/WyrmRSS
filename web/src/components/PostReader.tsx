import { useEffect, useRef } from "react";
import YouTube from "react-youtube";
import { TbArchive, TbArchiveOff, TbStar, TbStarFilled } from "react-icons/tb";
import { usePost } from "../hooks/usePosts";
import { useArchivePost, useUnarchivePost, useSetPostRead, useSetPostFavorite } from "../hooks/usePostMutations";
import { useSettings } from "../hooks/useSettings";
import { extractYouTubeId, formatDate } from "../utils/utils";
import { ReaderPane } from "./ReaderPane";
import { SanitizedHtml } from "./SanitizedHtml";
import type { PostId } from "../types/PostId";

interface Props {
  postId: PostId | null;
  onClose: () => void;
  width: number;
}

export function PostReader({ postId, onClose, width }: Props) {
  const { data: post, isLoading } = usePost(postId ?? undefined);
  const { mutate: setRead } = useSetPostRead();
  const { mutate: setFavorite } = useSetPostFavorite();
  const { mutate: archivePost } = useArchivePost();
  const { mutate: unarchivePost } = useUnarchivePost();
  const { data: settings } = useSettings();

  // Track the last seen post ID with a ref so the effect only fires
  // when the user switches to a new post, not on every re-render.
  const lastSeenPostId = useRef<PostId | null>(null);
  useEffect(() => {
    if (!post) return;
    if (lastSeenPostId.current !== post.id) {
      lastSeenPostId.current = post.id;
      if (!post.is_read && settings?.read_mode === "on_open") {
        setRead({ id: post.id, isRead: true });
      }
    }
  }, [post, setRead, settings?.read_mode]);

  if (!postId) return null;

  const body = post?.content ?? post?.description;
  const date = formatDate(post?.published_at ?? null);
  const youtubeId = extractYouTubeId(post?.url ?? null);

  const actions = post && (
    <>
      <button
        className={`pane-reader-fav${post.is_favorite ? " favorited" : ""}`}
        onClick={() => setFavorite({ id: post.id, isFavorite: !post.is_favorite })}
        aria-label={post.is_favorite ? "Unfavorite" : "Favorite"}
      >
        {post.is_favorite ? <TbStarFilled /> : <TbStar />}
      </button>
      <button
        className={`pane-reader-archive${post.is_archived ? " archived" : ""}`}
        onClick={() => post.is_archived ? unarchivePost(post.id) : archivePost(post.id)}
        aria-label={post.is_archived ? "Unarchive" : "Archive"}
      >
        {post.is_archived ? <TbArchiveOff /> : <TbArchive />}
      </button>
    </>
  );

  return (
    <ReaderPane width={width} onClose={onClose} actions={actions} isLoading={isLoading} notFound={!isLoading && !post} notFoundMessage="Post not found">
      {post && (
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
            <SanitizedHtml className="post-reader-body" html={body} />
          )}
          {body && youtubeId && (
            <p className="post-reader-body" style={{ whiteSpace: "pre-wrap" }}>{body}</p>
          )}
        </div>
      )}
    </ReaderPane>
  );
}
