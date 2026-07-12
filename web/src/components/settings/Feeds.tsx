import { useState } from "react";
import { TbEdit, TbPlayerPause, TbPlayerPlay, TbPlus, TbTrash } from "react-icons/tb";
import { useDeleteFeed, useFeeds, useSetFeedPaused } from "../../hooks/useFeeds";
import { useFolders } from "../../hooks/useFolders";
import { AddFeedForm } from "../AddFeedForm";
import { EditFeedForm } from "../EditFeedForm";
import type { Feed } from "../../types/Feed";
import { formatLastFetched } from "../../utils/utils";

function FeedRow({ feed, folderName }: { feed: Feed; folderName?: string }) {
  const [editing, setEditing] = useState(false);
  const { mutate: deleteFeed } = useDeleteFeed();
  const setPaused = useSetFeedPaused();

  return (
    <>
      <div className={`settings-table-row${editing ? " editing" : ""}${feed.is_paused ? " paused" : ""}`}>
        <span className="settings-table-title">{feed.title}</span>
        <span className="settings-table-url" title={feed.url}>{feed.url}</span>
        <span className="settings-table-meta">{feed.ttl}m</span>
        <span className="settings-table-badge">
          {folderName && <span className="feed-item-folder">{folderName}</span>}
        </span>
        <span className="settings-table-meta">{formatLastFetched(feed.last_fetched_at)}</span>
        <div className="settings-table-actions">
          <button
            className="settings-table-btn"
            onClick={() => setPaused.mutate({ id: feed.id, paused: !feed.is_paused })}
            title={feed.is_paused ? "Resume feed updates" : "Pause feed updates"}
          >
            {feed.is_paused ? <TbPlayerPlay /> : <TbPlayerPause />}
          </button>
          <button
            className={`settings-table-btn${editing ? " active" : ""}`}
            onClick={() => setEditing((e) => !e)}
            title="Edit"
          >
            <TbEdit />
          </button>
          <button
            className="settings-table-btn settings-table-btn-delete"
            onClick={() => deleteFeed(feed.id)}
            title="Delete"
          >
            <TbTrash />
          </button>
        </div>
      </div>
      {editing && (
        <div className="settings-table-edit">
          <EditFeedForm feed={feed} onClose={() => setEditing(false)} />
        </div>
      )}
    </>
  );
}

export function Feeds() {
  const { data: feeds, isLoading } = useFeeds();
  const { data: folders } = useFolders();
  const [search, setSearch] = useState("");
  const [adding, setAdding] = useState(false);

  if (isLoading) return <div className="pane-empty">Loading…</div>;

  const folderNames = new Map((folders ?? []).map((f) => [f.id, f.name]));

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
    <div className="settings-table-wrap">
      <div className="settings-table-toolbar">
        <input
          className="settings-table-search"
          placeholder="Search feeds…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <button
          className={`btn ${adding ? "btn-ghost" : "btn-primary"} settings-table-add-btn`}
          onClick={() => setAdding((a) => !a)}
        >
          <TbPlus /> {adding ? "Cancel" : "Add feed"}
        </button>
      </div>
      {adding && (
        <div className="settings-table-edit">
          <AddFeedForm onClose={() => setAdding(false)} />
        </div>
      )}
      {!feeds?.length
        ? <div className="settings-empty">No feeds yet.</div>
        : <div className="settings-table feeds-table">
          <div className="settings-table-header">
            <span className="settings-table-title">Title</span>
            <span className="settings-table-url">URL</span>
            <span className="settings-table-meta">TTL</span>
            <span className="settings-table-badge">Folder</span>
            <span className="settings-table-meta">Last fetched</span>
            <span className="settings-table-actions" />
          </div>
          {filtered.map((feed) => (
            <FeedRow
              key={feed.id}
              feed={feed}
              folderName={feed.folder_id !== undefined ? folderNames.get(feed.folder_id) : undefined}
            />
          ))}
          {filtered.length === 0 && (
            <div className="pane-empty">No feeds match.</div>
          )}
        </div>
      }
    </div>
  );
}
