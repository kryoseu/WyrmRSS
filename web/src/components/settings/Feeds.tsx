import { useState } from "react";
import { TbEdit, TbPlus, TbTrash } from "react-icons/tb";
import { useDeleteFeed, useFeeds } from "../../hooks/useFeeds";
import { AddFeedForm } from "../AddFeedForm";
import { EditFeedForm } from "../EditFeedForm";
import type { Feed } from "../../types/Feed";
import { formatLastFetched } from "../../utils/utils";

function FeedRow({ feed }: { feed: Feed }) {
  const [editing, setEditing] = useState(false);
  const { mutate: deleteFeed } = useDeleteFeed();

  return (
    <>
      <div className={`feeds-table-row${editing ? " editing" : ""}`}>
        <span className="feeds-table-title">{feed.title}</span>
        <span className="feeds-table-url" title={feed.url}>{feed.url}</span>
        <span className="feeds-table-ttl">{feed.ttl}m</span>
        <span className="feeds-table-tag">
          {feed.tag && (
            <span
              className="feed-item-tag"
              style={feed.tag_color ? ({ "--tag-color": feed.tag_color } as React.CSSProperties) : undefined}
            >
              {feed.tag}
            </span>
          )}
        </span>
        <span className="feeds-table-fetched">{formatLastFetched(feed.last_fetched_at)}</span>
        <div className="feeds-table-actions">
          <button
            className={`feeds-table-btn${editing ? " active" : ""}`}
            onClick={() => setEditing((e) => !e)}
            title="Edit"
          >
            <TbEdit />
          </button>
          <button
            className="feeds-table-btn feeds-table-btn-delete"
            onClick={() => deleteFeed(feed.id)}
            title="Delete"
          >
            <TbTrash />
          </button>
        </div>
      </div>
      {editing && (
        <div className="feeds-table-edit">
          <EditFeedForm feed={feed} onClose={() => setEditing(false)} />
        </div>
      )}
    </>
  );
}

export function Feeds() {
  const { data: feeds, isLoading } = useFeeds();
  const [search, setSearch] = useState("");
  const [adding, setAdding] = useState(false);

  if (isLoading) return <div className="pane-empty">Loading…</div>;

  const filtered = (feeds ?? [])
    .filter((f) => f.title.toLowerCase().includes(search.toLowerCase()))
    // Most recently fetched first; never-fetched feeds sink to the bottom
    .sort((a, b) => {
      if (!a.last_fetched_at && !b.last_fetched_at) return 0;
      if (!a.last_fetched_at) return 1;
      if (!b.last_fetched_at) return -1;
      return new Date(b.last_fetched_at).getTime() - new Date(a.last_fetched_at).getTime();
    });

  return (
    <div className="feeds-table-wrap">
      <div className="feeds-table-toolbar">
        <input
          className="feeds-table-search"
          placeholder="Search feeds…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <button
          className={`btn ${adding ? "btn-ghost" : "btn-primary"} feeds-table-add-btn`}
          onClick={() => setAdding((a) => !a)}
        >
          <TbPlus /> {adding ? "Cancel" : "Add feed"}
        </button>
      </div>
      {adding && (
        <div className="feeds-table-edit">
          <AddFeedForm onClose={() => setAdding(false)} />
        </div>
      )}
      {!feeds?.length
        ? <div className="settings-empty">No feeds yet.</div>
        : <div className="feeds-table">
          <div className="feeds-table-header">
            <span className="feeds-table-title">Title</span>
            <span className="feeds-table-url">URL</span>
            <span className="feeds-table-ttl">TTL</span>
            <span className="feeds-table-tag">Tag</span>
            <span className="feeds-table-fetched">Last fetched</span>
            <span className="feeds-table-actions" />
          </div>
          {filtered.map((feed) => (
            <FeedRow key={feed.id} feed={feed} />
          ))}
          {filtered.length === 0 && (
            <div className="pane-empty">No feeds match.</div>
          )}
        </div>
      }
    </div>
  );
}
