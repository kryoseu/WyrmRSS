import { type SubmitEvent, useState } from "react";
import { useCreateFeed } from "../hooks/useFeeds";
import { FolderCombobox } from "./FolderCombobox";
import { DisplayModeSelect } from "./DisplayModeSelect";
import type { DisplayMode } from "../types/DisplayMode";

interface Props {
  onClose: () => void;
}

export function AddFeedForm({ onClose }: Props) {
  const [title, setTitle] = useState("");
  const [url, setUrl] = useState("");
  const [ttl, setTtl] = useState("");
  const [folder, setFolder] = useState("");
  const [filters, setFilters] = useState<string[]>([]);
  const [displayMode, setDisplayMode] = useState<DisplayMode>("river");
  const create = useCreateFeed();

  function updateFilter(index: number, value: string) {
    setFilters(filters.map((f, i) => (i === index ? value : f)));
  }

  function removeFilter(index: number) {
    setFilters(filters.filter((_, i) => i !== index));
  }

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    create.mutate(
      {
        title,
        url,
        ttl: ttl === "" ? 60 : Number(ttl),
        folder: folder.trim() || null,
        filters: filters.map((f) => f.trim()).filter(Boolean),
        display_mode: displayMode,
      },
      { onSuccess: onClose }
    );
  }

  return (
    <form className="entity-form" onSubmit={handleSubmit}>
      <input
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        placeholder="Title"
        required
        autoFocus
      />
      <input
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        placeholder="Feed URL"
        type="url"
        required
      />
      <input
        value={ttl}
        onChange={(e) => setTtl(e.target.value.replace(/\D/g, ""))}
        placeholder="Refresh (min)"
        type="number"
        min={1}
      />
      <FolderCombobox value={folder} onChange={setFolder} />
      <DisplayModeSelect value={displayMode} onChange={setDisplayMode} />
      <div className="url-filters">
        {filters.map((filter, i) => (
          <div key={i} className="url-filter-row">
            <input
              value={filter}
              onChange={(e) => updateFilter(i, e.target.value)}
              placeholder="e.g. title:sponsored"
              title="url: title: content: to scope; no prefix matches everywhere"
            />
            <button type="button" className="btn-remove-filter" onClick={() => removeFilter(i)}>×</button>
          </div>
        ))}
        <button
          type="button"
          className="btn btn-ghost"
          onClick={() => setFilters([...filters, ""])}
        >
          + Filter
        </button>
      </div>
      {create.isError && <div className="form-error">{create.error.message}</div>}
      <div className="entity-form-actions">
        <button
          className="btn btn-primary"
          type="submit"
          disabled={create.isPending}
        >
          {create.isPending ? "Adding…" : "Add"}
        </button>
        <button
          className="btn btn-ghost"
          type="button"
          onClick={onClose}
        >
          Cancel
        </button>
      </div>
    </form>
  );
}
