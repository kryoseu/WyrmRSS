import { useState, useRef } from "react";
import { Outlet, useLocation } from "react-router";
import { ArchiveReader } from "../components/ArchiveReader";
import { PostReader } from "../components/PostReader";
import type { PostId } from "../types/PostId";

export type ReaderOutletContext = {
  activePostId: PostId | null;
  onOpenPost: (id: PostId) => void;
};

const MIN_WIDTH = 280;

export function ReaderPage() {
  const { pathname } = useLocation();
  const isArchive = pathname.startsWith("/archive");
  const [activePostId, setActivePostId] = useState<PostId | null>(null);
  const [lastIsArchive, setLastIsArchive] = useState(isArchive);

  if (lastIsArchive !== isArchive) {
    setLastIsArchive(isArchive);
    setActivePostId(null);
  }
  const [readerWidth, setReaderWidth] = useState(() => Math.round(window.innerWidth * 0.40));
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
    setReaderWidth(Math.min(maxWidth, Math.max(MIN_WIDTH, dragStart.current.width + delta)));
  }

  function onResizeEnd() {
    dragStart.current = null;
    document.body.classList.remove("reader-resizing");
  }

  return (
    <>
      <Outlet context={{ activePostId, onOpenPost: setActivePostId } satisfies ReaderOutletContext} />
      {activePostId !== null && (
        <div
          className="reader-resize-handle"
          onPointerDown={onResizeStart}
          onPointerMove={onResizeMove}
          onPointerUp={onResizeEnd}
        />
      )}
      {isArchive
        ? <ArchiveReader postId={activePostId} onClose={() => setActivePostId(null)} width={readerWidth} />
        : <PostReader postId={activePostId} onClose={() => setActivePostId(null)} width={readerWidth} />
      }
    </>
  );
}
