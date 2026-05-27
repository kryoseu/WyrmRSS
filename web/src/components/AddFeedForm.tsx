import { useState } from "react";
import { useCreateFeed } from "../hooks/useFeeds";

interface Props {
  onClose: () => void;
}

export function AddFeedForm({ onClose }: Props) {
  const [title, setTitle] = useState("");
  const [url, setUrl] = useState("");
  const [ttl, setTtl] = useState(60);
  const [urlFilters, setUrlFilters] = useState<string[]>([]);
  const create = useCreateFeed();

  function updateFilter(index: number, value: string) {
    setUrlFilters(urlFilters.map((f, i) => (i === index ? value : f)));
  }

  function removeFilter(index: number) {
    setUrlFilters(urlFilters.filter((_, i) => i !== index));
  }

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    create.mutate(
      { title, url, ttl, url_filter: urlFilters.map((f) => f.trim()).filter(Boolean) },
      { onSuccess: onClose }
    );
  }

  return (
    <form className="add-feed-form" onSubmit={handleSubmit}>
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
        onChange={(e) => setTtl(Number(e.target.value))}
        placeholder="Refresh (min)"
        type="number"
        min={1}
        required
      />
      <div className="url-filters">
        {urlFilters.map((filter, i) => (
          <div key={i} className="url-filter-row">
            <input
              value={filter}
              onChange={(e) => updateFilter(i, e.target.value)}
              placeholder="e.g. /unwanted"
            />
            <button type="button" className="btn-remove-filter" onClick={() => removeFilter(i)}>×</button>
          </div>
        ))}
        <button
          type="button"
          className="btn btn-ghost"
          onClick={() => setUrlFilters([...urlFilters, ""])}
        >
          + URL filter
        </button>
      </div>
      <div className="add-feed-form-actions">
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
