import { useEffect, useRef } from "react";
import { isTauri } from "@tauri-apps/api/core";
import YouTube from "react-youtube";
import { TbArchive, TbArchiveOff, TbBookmark, TbBookmarkFilled } from "react-icons/tb";
import { usePost } from "../../hooks/usePosts";
import { useArchivePost, useUnarchivePost, useSetPostRead, useSetPostBookmarked } from "../../hooks/usePostMutations";
import { useSettings } from "../../hooks/useSettings";
import { useShortcutHandler } from "../../hooks/useShortcuts";
import { ENDPOINTS } from "../../utils/api";
import { extractYouTubeId, formatDate } from "../../utils/utils";
import { extractHlsVideo } from "../../utils/hls";
import { ReaderPane } from "../layout/ReaderPane";
import { SanitizedHtml } from "../common/SanitizedHtml";
import { HlsVideo } from "../common/HlsVideo";
import type { PostId } from "../../types/PostId";
import { useFeed } from "../../hooks/useFeeds";

interface Props {
  postId: PostId | null;
  onClose: () => void;
  width?: number;
}

export function PostReader({ postId, onClose, width }: Props) {
  const { data: post, isLoading } = usePost(postId ?? undefined);
  const { data: feed } = useFeed(post?.feed_id);
  const { mutate: setRead } = useSetPostRead();
  const { mutate: setBookmarked } = useSetPostBookmarked();
  const { mutate: archivePost } = useArchivePost();
  const { mutate: unarchivePost } = useUnarchivePost();
  const { data: settings } = useSettings();
  // Clicking the link reuses its mark-read and desktop handling.
  const originalLinkRef = useRef<HTMLAnchorElement>(null);

  // Mark read after the post stays open for read_delay_secs. Switching posts,
  // closing, or a manual read toggle cancels it; at most once per selection.
  const handledPostId = useRef<PostId | null>(null);
  const loadedId = post?.id;
  const isRead = post?.is_read;
  const readMode = settings?.read_mode;
  const readDelaySecs = settings?.read_delay_secs;
  useEffect(() => {
    if (loadedId === undefined || readMode === undefined || readDelaySecs === undefined) return;
    if (handledPostId.current === loadedId) return;
    if (isRead || readMode !== "on_open") {
      handledPostId.current = loadedId;
      return;
    }
    const timer = setTimeout(() => {
      handledPostId.current = loadedId;
      setRead({ id: loadedId, isRead: true });
    }, readDelaySecs * 1000);
    return () => clearTimeout(timer);
  }, [loadedId, isRead, readMode, readDelaySecs, setRead]);

  // Opening the original is a clear read signal, so skip the delay.
  function markReadOnOpenOriginal() {
    if (post && !post.is_read && readMode === "on_open") {
      setRead({ id: post.id, isRead: true });
    }
  }

  function toggleRead() {
    if (post && readMode !== "disabled") setRead({ id: post.id, isRead: !post.is_read });
  }

  function toggleBookmarked() {
    if (post) setBookmarked({ id: post.id, bookmarked: !post.bookmarked });
  }

  function toggleArchived() {
    if (!post) return;
    if (post.is_archived) unarchivePost(post.id);
    else archivePost(post.id);
  }

  useShortcutHandler("toggle_read", toggleRead);
  useShortcutHandler("toggle_read_later", toggleBookmarked);
  useShortcutHandler("toggle_archive", toggleArchived);
  useShortcutHandler("open_original", () => originalLinkRef.current?.click());

  if (!postId) return null;

  const rawBody = post?.content ?? post?.description;
  const date = formatDate(post?.published_at ?? null);
  const youtubeId = extractYouTubeId(post?.url ?? null);
  const video = youtubeId || !rawBody ? null : extractHlsVideo(rawBody);
  const body = video?.html ?? rawBody;

  const actions = post && (
    <>
      <button
        className={`pane-reader-bookmark${post.bookmarked ? " bookmarked" : ""}`}
        onClick={toggleBookmarked}
        aria-label={post.bookmarked ? "Remove from read later" : "Read later"}
      >
        {post.bookmarked ? <TbBookmarkFilled /> : <TbBookmark />}
      </button>
      <button
        className={`pane-reader-archive${post.is_archived ? " archived" : ""}`}
        onClick={toggleArchived}
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
            {feed && <span>{feed.title}</span>}
            {post.authors && <span>{post.authors}</span>}
            {date && <span>{date}</span>}
            {post.url && (
              // target="_blank" bypasses the desktop link handler; drop it there.
              <a
                ref={originalLinkRef}
                href={post.url}
                target={isTauri() ? undefined : "_blank"}
                rel="noopener noreferrer"
                onClick={markReadOnOpenOriginal}
                onAuxClick={(e) => e.button === 1 && markReadOnOpenOriginal()}
              >
                Open original ↗
              </a>
            )}
          </div>
          {youtubeId && (
            <div className="post-reader-youtube">
              {isTauri() ? (
                // The WebKit-based webviews (macOS, Linux) serve the app from a
                // custom tauri:// scheme with no valid HTTP referer, which fails
                // YouTube's embed domain check ("Error 153") no matter what
                // origin is passed to the player. Loading the iframe from a page
                // served by our own embedded backend instead works around it.
                //
                // https://github.com/tauri-apps/tauri/issues/14422
                <iframe
                  src={ENDPOINTS.desktop.youtubeEmbed(youtubeId)}
                  width="100%"
                  height="390"
                  style={{ border: "none" }}
                  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                  allowFullScreen
                />
              ) : (
                <YouTube videoId={youtubeId} opts={{ width: "100%" }} />
              )}
            </div>
          )}
          {video && <HlsVideo src={video.src} poster={video.poster} />}
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
