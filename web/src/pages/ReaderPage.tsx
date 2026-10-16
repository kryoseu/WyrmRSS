import { useState, useRef } from "react";
import { Outlet, useLocation, useNavigate, useParams } from "react-router";
import { ArchiveReader } from "../components/archive/ArchiveReader";
import { PostReader } from "../components/posts/PostReader";
import { useIsMobile } from "../hooks/useIsMobile";
import { useShortcutHandler } from "../hooks/useShortcuts";
import type { PostId } from "../types/PostId";
import { LAYOUT } from "../utils/constants";
import { listPath } from "../utils/posts";

export type ReaderOutletContext = {
  activePostId: PostId | null;
  onOpenPost: (id: PostId) => void;
};

export function ReaderPage() {
  const { pathname } = useLocation();
  const { postId } = useParams();
  const navigate = useNavigate();
  const isMobile = useIsMobile();
  const isArchive = pathname.startsWith("/archive");
  const [activePostId, setActivePostId] = useState<PostId | null>(null);
  const [lastIsArchive, setLastIsArchive] = useState(isArchive);

  if (lastIsArchive !== isArchive) {
    setLastIsArchive(isArchive);
    setActivePostId(null);
  }

  // On a phone the reader covers the list, so it follows the URL: the back
  // gesture leaves the post route, and that has to close the reader.
  if (isMobile && postId === undefined && activePostId !== null) {
    setActivePostId(null);
  }

  function closeReader() {
    if (!isMobile) {
      setActivePostId(null);
      return;
    }
    // Back to the list entry the post was opened from. A deep link has no
    // such entry, so swap it for the list rather than leaving the app.
    if ((window.history.state?.idx ?? 0) > 0) navigate(-1);
    else navigate(listPath(pathname), { replace: true });
  }

  useShortcutHandler("close_reader", closeReader, activePostId !== null);

  const [readerWidth, setReaderWidth] = useState(() => Math.round(window.innerWidth * LAYOUT.readerWidthFraction));
  const dragStart = useRef<{ x: number; width: number } | null>(null);

  function onResizeStart(e: React.PointerEvent) {
    // Stops the drag from also selecting text underneath the cursor.
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    document.body.classList.add("reader-resizing");
    dragStart.current = { x: e.clientX, width: readerWidth };
  }

  function onResizeMove(e: React.PointerEvent) {
    if (!dragStart.current) return;
    const delta = dragStart.current.x - e.clientX;
    const maxWidth = Math.round(window.innerWidth * 0.65);
    setReaderWidth(Math.min(maxWidth, Math.max(LAYOUT.readerMinWidthPx, dragStart.current.width + delta)));
  }

  function onResizeEnd() {
    dragStart.current = null;
    document.body.classList.remove("reader-resizing");
  }

  const width = isMobile ? undefined : readerWidth;

  return (
    <>
      <Outlet context={{ activePostId, onOpenPost: setActivePostId } satisfies ReaderOutletContext} />
      {!isMobile && activePostId !== null && (
        <div
          className="reader-resize-handle"
          onPointerDown={onResizeStart}
          onPointerMove={onResizeMove}
          onPointerUp={onResizeEnd}
        />
      )}
      {isArchive
        ? <ArchiveReader postId={activePostId} onClose={closeReader} width={width} />
        : <PostReader postId={activePostId} onClose={closeReader} width={width} />
      }
    </>
  );
}
