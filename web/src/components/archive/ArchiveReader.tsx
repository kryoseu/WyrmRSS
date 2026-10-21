import { useRef } from "react";
import { TbArchiveOff } from "react-icons/tb";
import { useArchivedPost } from "../../hooks/usePosts";
import { useUnarchivePost } from "../../hooks/usePostMutations";
import { useShortcutHandler } from "../../hooks/useShortcuts";
import { formatDate } from "../../utils/utils";
import { extractHlsVideo } from "../../utils/hls";
import { ReaderPane } from "../layout/ReaderPane";
import { SanitizedHtml } from "../common/SanitizedHtml";
import { HlsVideo } from "../common/HlsVideo";
import { PostSource } from "../posts/PostSource";
import type { PostId } from "../../types/PostId";

interface Props {
  postId: PostId | null;
  onClose: () => void;
  width?: number;
}

export function ArchiveReader({ postId, onClose, width }: Props) {
  const { data: archive, isLoading } = useArchivedPost(postId ?? undefined);
  const { mutate: unarchive } = useUnarchivePost();

  // Unarchiving removes the post from this list, so close its reader too.
  function unarchiveAndClose() {
    if (archive) unarchive(archive.id, { onSuccess: onClose });
  }

  const originalLinkRef = useRef<HTMLAnchorElement>(null);

  useShortcutHandler("toggle_archive", unarchiveAndClose);
  useShortcutHandler("open_original", () => originalLinkRef.current?.click());

  if (!postId) return null;

  const rawBody = archive?.content ?? archive?.description;
  const video = rawBody ? extractHlsVideo(rawBody) : null;
  const body = video?.html ?? rawBody;
  const date = formatDate(archive?.published_at ?? null);

  const actions = archive && (
    <>
      {rawBody && <PostSource html={rawBody} />}
      <button
        className="pane-reader-archive archived"
        onClick={unarchiveAndClose}
        aria-label="Unarchive"
      >
        <TbArchiveOff />
      </button>
    </>
  );

  return (
    <ReaderPane width={width} onClose={onClose} actions={actions} isLoading={isLoading} notFound={!isLoading && !archive} notFoundMessage="Archive not found">
      {archive && (
        <div className="post-reader">
          <h1 className="post-reader-title">
            {archive.title ?? "Untitled"}
            {archive.url && (
              <a
                ref={originalLinkRef}
                className="post-reader-original"
                href={archive.url}
                target="_blank"
                rel="noopener noreferrer"
                aria-label="Open original"
                title="Open original"
              >
                ↗
              </a>
            )}
          </h1>
          <div className="post-reader-meta">
            {date && <span>{date}</span>}
            {archive.authors && <span>{archive.authors}</span>}
          </div>
          {video && <HlsVideo src={video.src} poster={video.poster} />}
          {body && (
            <SanitizedHtml className="post-reader-body" html={body} />
          )}
        </div>
      )}
    </ReaderPane>
  );
}
