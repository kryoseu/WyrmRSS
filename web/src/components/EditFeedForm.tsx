import { useState } from "react";
import { FiCheck, FiCopy } from "react-icons/fi";
import { useUpdateFeed } from "../hooks/useFeeds";
import type { Feed } from "../types/Feed";

interface Props {
  feed: Feed;
  onClose: () => void;
}

export function EditFeedForm({ feed, onClose }: Props) {
  const [title, setTitle] = useState(feed.title);
  const [url, setUrl] = useState(feed.url);
  const [ttl, setTtl] = useState(String(feed.ttl));
  const [tag, setTag] = useState(feed.tag ?? "");
  console.log(feed);
  const [tagColor, setTagColor] = useState(feed.tag_color ?? "#6b7280");
  const [copied, setCopied] = useState(false);
  const [urlFilters, setUrlFilters] = useState<string[]>(
    feed.url_filter.filter((f): f is string => f !== null)
  );
  const update = useUpdateFeed();

  function updateFilter(index: number, value: string) {
    setUrlFilters(urlFilters.map((f, i) => (i === index ? value : f)));
  }

  function removeFilter(index: number) {
    setUrlFilters(urlFilters.filter((_, i) => i !== index));
  }

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    update.mutate(
      {
        id: feed.id,
        body: {
          title: title.trim() || null,
          url: url.trim() || null,
          ttl: Number(ttl) || null,
          tag: tag.trim() || null,
          tag_color: tag.trim() ? tagColor : null,
          url_filter: urlFilters.map((f) => f.trim()).filter(Boolean),
        },
      },
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
        onChange={(e) => setTtl(e.target.value.replace(/\D/g, ""))}
        placeholder="Refresh (min)"
        type="number"
        min={1}
        required
      />
      <div className="feed-tag-row">
        <input
          value={tag}
          onChange={(e) => setTag(e.target.value)}
          placeholder="Tag (optional)"
        />
        <input
          type="color"
          value={tagColor}
          onChange={(e) => setTagColor(e.target.value)}
          disabled={!tag.trim()}
          title="Tag color"
        />
        <button
          type="button"
          className="btn-copy-color"
          disabled={!tag.trim()}
          title={tagColor}
          onClick={() => {
            navigator.clipboard.writeText(tagColor);
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
          }}
        >
          {copied ? <FiCheck size={13} /> : <FiCopy size={13} />}
        </button>
      </div>
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
          disabled={update.isPending}
        >
          {update.isPending ? "Saving…" : "Save"}
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
