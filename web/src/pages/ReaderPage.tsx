import { useState } from "react";
import { Outlet } from "react-router-dom";
import { SideMenu } from "../components/SideMenu";
import { PostReader } from "../components/PostReader";

export type ReaderOutletContext = {
  excludedFeeds: Set<number>;
  onToggleExclude: (feedId: number) => void;
  activePostId: number | null;
  onOpenPost: (id: number) => void;
};

export function ReaderPage() {
  const [excludedFeeds, setExcludedFeeds] = useState<Set<number>>(new Set());
  const [activePostId, setActivePostId] = useState<number | null>(null);

  function toggleExclude(feedId: number) {
    setExcludedFeeds((prev) => {
      const next = new Set(prev);
      if (next.has(feedId)) next.delete(feedId);
      else next.add(feedId);
      return next;
    });
  }

  return (
    <div className="layout">
      <SideMenu excludedFeeds={excludedFeeds} onToggleExclude={toggleExclude} />
      <Outlet context={{ excludedFeeds, onToggleExclude: toggleExclude, activePostId, onOpenPost: setActivePostId } satisfies ReaderOutletContext} />
      <PostReader postId={activePostId} onClose={() => setActivePostId(null)} />
    </div>
  );
}
