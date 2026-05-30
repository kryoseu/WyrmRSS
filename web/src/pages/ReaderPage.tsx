import { useState, useRef } from "react";
import { Outlet } from "react-router-dom";
import { SideMenu } from "../components/SideMenu";
import { PostReader } from "../components/PostReader";

export type ReaderOutletContext = {
  excludedFeeds: Set<number>;
  onToggleExclude: (feedId: number) => void;
  activePostId: number | null;
  onOpenPost: (id: number) => void;
};

const MIN_WIDTH = 280;
const MAX_WIDTH = Math.round(window.innerWidth * 0.65);

export function ReaderPage() {
  const [excludedFeeds, setExcludedFeeds] = useState<Set<number>>(new Set());
  const [activePostId, setActivePostId] = useState<number | null>(null);
  const [readerWidth, setReaderWidth] = useState(() => Math.round(window.innerWidth * 0.40));
  const dragStart = useRef<{ x: number; width: number } | null>(null);

  function toggleExclude(feedId: number) {
    setExcludedFeeds((prev) => {
      const next = new Set(prev);
      if (next.has(feedId)) next.delete(feedId);
      else next.add(feedId);
      return next;
    });
  }

  function onResizeStart(e: React.PointerEvent) {
    e.currentTarget.setPointerCapture(e.pointerId);
    dragStart.current = { x: e.clientX, width: readerWidth };
  }

  function onResizeMove(e: React.PointerEvent) {
    if (!dragStart.current) return;
    const delta = dragStart.current.x - e.clientX;
    setReaderWidth(Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, dragStart.current.width + delta)));
  }

  function onResizeEnd() {
    dragStart.current = null;
  }

  return (
    <div className="layout">
      <SideMenu excludedFeeds={excludedFeeds} onToggleExclude={toggleExclude} />
      <Outlet context={{ excludedFeeds, onToggleExclude: toggleExclude, activePostId, onOpenPost: setActivePostId } satisfies ReaderOutletContext} />
      {activePostId !== null && (
        <div
          className="reader-resize-handle"
          onPointerDown={onResizeStart}
          onPointerMove={onResizeMove}
          onPointerUp={onResizeEnd}
        />
      )}
      <PostReader postId={activePostId} onClose={() => setActivePostId(null)} width={readerWidth} />
    </div>
  );
}
