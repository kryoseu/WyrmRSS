import { useState } from "react";
import { useFeeds } from "../hooks/useFeeds";
import { FeedItem } from "./FeedItem";
import { useParams } from "react-router-dom";
import { AddFeedForm } from "./AddFeedForm";

interface Props {
  excludedFeeds: Set<number>;
  onToggleExclude: (feedId: number) => void;
}

export function FeedList({ excludedFeeds, onToggleExclude }: Props) {
  const { feedId } = useParams();
  const { data: feeds, isLoading } = useFeeds();

  const [showAdd, setShowAdd] = useState(false);
  const [search, setSearch] = useState("");

  const filtered = feeds?.filter((f) =>
    f.title.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <>
      <div className="sidebar-feeds-header">
        <span>Feeds</span>
        <input
          className="sidebar-feeds-search"
          placeholder="Search…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      <div className="sidebar-feeds-scroll">
        {isLoading && <div className="pane-empty">Loading…</div>}
        {filtered?.map((f) => (
          <FeedItem
            key={f.id}
            feed={f}
            active={feedId === String(f.id)}
            excluded={excludedFeeds.has(f.id)}
            onToggleExclude={onToggleExclude}
          />
        ))}
        {filtered?.length === 0 && !isLoading && (
          <div className="pane-empty">No feeds</div>
        )}
      </div>

      {showAdd ? (
        <AddFeedForm onClose={() => setShowAdd(false)} />
      ) : (
        <button className="add-feed-btn" onClick={() => setShowAdd(true)}>
          + Add Feed
        </button>
      )}

    </>);
}
