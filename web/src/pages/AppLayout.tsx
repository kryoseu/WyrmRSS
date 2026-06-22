import { useState } from "react";
import { Outlet } from "react-router-dom";
import { SideMenu } from "../components/SideMenu";
import type { FeedId } from "../types/FeedId";

export type AppLayoutContext = {
  excludedFeeds: Set<FeedId>;
  onToggleExclude: (id: FeedId) => void;
};

export function AppLayout() {
  const [excludedFeeds, setExcludedFeeds] = useState<Set<FeedId>>(new Set());

  function toggleExclude(id: FeedId) {
    setExcludedFeeds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  return (
    <div className="layout">
      <SideMenu excludedFeeds={excludedFeeds} onToggleExclude={toggleExclude} />
      <Outlet context={{ excludedFeeds, onToggleExclude: toggleExclude } satisfies AppLayoutContext} />
    </div>
  );
}
