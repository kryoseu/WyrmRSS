import { useState } from "react";
import { SideMenu } from "../components/SideMenu";
import { PostList } from "../components/PostList";
import { PostReader } from "../components/PostReader";

export function ReaderPage() {
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
      <PostList excludedFeeds={excludedFeeds} />
      <PostReader />
    </div>
  );
}
