import { useState } from "react";
import { Outlet } from "react-router-dom";
import { SideMenu } from "../components/SideMenu";

export type AppLayoutContext = {
  excludedFeeds: Set<number>;
  onToggleExclude: (feedId: number) => void;
};

export function AppLayout() {
  const [excludedFeeds, setExcludedFeeds] = useState<Set<number>>(new Set());

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
      <Outlet context={{ excludedFeeds, onToggleExclude: toggleExclude } satisfies AppLayoutContext} />
    </div>
  );
}
