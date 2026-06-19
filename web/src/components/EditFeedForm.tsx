import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { FiCheck, FiCopy } from "react-icons/fi";
import { useUpdateFeed } from "../hooks/useFeeds";
import { useFeedWebhooks, webhookKeys } from "../hooks/useWebhooks";
import { attachWebhook, detachWebhook } from "../api/webhooks";
import type { Feed } from "../types/Feed";
import { FeedWebhooks } from "./webhook/FeedWebhooks";

interface Props {
  feed: Feed;
  onClose: () => void;
}

export function EditFeedForm({ feed, onClose }: Props) {
  const [title, setTitle] = useState(feed.title);
  const [url, setUrl] = useState(feed.url);
  const [ttl, setTtl] = useState(String(feed.ttl));
  const [tag, setTag] = useState(feed.tag ?? "");
  const [tagColor, setTagColor] = useState(feed.tag_color ?? "#6b7280");
  const [copied, setCopied] = useState(false);
  const [urlFilters, setUrlFilters] = useState<string[]>(
    feed.url_filter.filter((f): f is string => f !== null)
  );
  const update = useUpdateFeed();
  const qc = useQueryClient();

  // Webhook assignments are staged locally and only committed on Save (so
  // Cancel discards them). Seed the selection from the feed's current webhooks
  // once they load, using the setState-during-render pattern.
  const { data: attachedWebhooks } = useFeedWebhooks(feed.id);
  const [selectedWebhooks, setSelectedWebhooks] = useState<Set<number>>(new Set());
  const [seeded, setSeeded] = useState(false);
  if (attachedWebhooks && !seeded) {
    setSeeded(true);
    setSelectedWebhooks(new Set(attachedWebhooks.map((w) => w.id)));
  }

  const [saving, setSaving] = useState(false);

  function updateFilter(index: number, value: string) {
    setUrlFilters(urlFilters.map((f, i) => (i === index ? value : f)));
  }

  function removeFilter(index: number) {
    setUrlFilters(urlFilters.filter((_, i) => i !== index));
  }

  function toggleWebhook(webhookId: number, checked: boolean) {
    setSelectedWebhooks((prev) => {
      const next = new Set(prev);
      if (checked) next.add(webhookId);
      else next.delete(webhookId);
      return next;
    });
  }

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setSaving(true);
    try {
      await update.mutateAsync({
        id: feed.id,
        body: {
          title: title.trim() || null,
          url: url.trim() || null,
          ttl: Number(ttl) || null,
          tag: tag.trim() || null,
          tag_color: tag.trim() ? tagColor : null,
          url_filter: urlFilters.map((f) => f.trim()).filter(Boolean),
        },
      });

      // Commit only the webhook changes: attach the added, detach the removed.
      const original = new Set((attachedWebhooks ?? []).map((w) => w.id));
      const toAttach = [...selectedWebhooks].filter((id) => !original.has(id));
      const toDetach = [...original].filter((id) => !selectedWebhooks.has(id));
      await Promise.all([
        ...toAttach.map((id) => attachWebhook(feed.id, id)),
        ...toDetach.map((id) => detachWebhook(feed.id, id)),
      ]);
      if (toAttach.length || toDetach.length) {
        qc.invalidateQueries({ queryKey: webhookKeys.forFeed(feed.id) });
      }
      onClose();
    } catch {
      setSaving(false);
    }
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
      <div className="feed-webhooks-section">
        <span className="feed-webhooks-label">Webhooks</span>
        <FeedWebhooks selected={selectedWebhooks} onToggle={toggleWebhook} />
      </div>
      <div className="add-feed-form-actions">
        <button
          className="btn btn-primary"
          type="submit"
          disabled={saving}
        >
          {saving ? "Saving…" : "Save"}
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
