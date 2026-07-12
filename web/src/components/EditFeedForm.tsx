import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useUpdateFeed } from "../hooks/useFeeds";
import { useFolders } from "../hooks/useFolders";
import { useFeedWebhooks, webhookKeys } from "../hooks/useWebhooks";
import { attachWebhook, detachWebhook } from "../api/webhooks";
import type { Feed } from "../types/Feed";
import type { WebhookId } from "../types/WebhookId";
import { FeedWebhooks } from "./webhook/FeedWebhooks";
import { FolderCombobox } from "./FolderCombobox";

interface Props {
  feed: Feed;
  onClose: () => void;
}

export function EditFeedForm({ feed, onClose }: Props) {
  const [title, setTitle] = useState(feed.title);
  const [url, setUrl] = useState(feed.url);
  const [ttl, setTtl] = useState(String(feed.ttl));
  const [urlFilters, setUrlFilters] = useState<string[]>(
    feed.filters.filter((f): f is string => f !== null)
  );
  const update = useUpdateFeed();
  const qc = useQueryClient();

  // The feed only carries folder_id; the name comes from the folders query.
  // Seed the input once folders load (setState-during-render pattern).
  const { data: folders } = useFolders();
  const [folder, setFolder] = useState("");
  const [folderSeeded, setFolderSeeded] = useState(false);
  if (folders && !folderSeeded) {
    setFolderSeeded(true);
    const current = folders.find((f) => f.id === feed.folder_id);
    if (current) setFolder(current.name);
  }

  // Webhook assignments are staged locally and only committed on Save (so
  // Cancel discards them). Seed the selection from the feed's current webhooks
  // once they load, using the setState-during-render pattern.
  const { data: attachedWebhooks } = useFeedWebhooks(feed.id);
  const [selectedWebhooks, setSelectedWebhooks] = useState<Set<WebhookId>>(new Set());
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

  function toggleWebhook(id: WebhookId, checked: boolean) {
    setSelectedWebhooks((prev) => {
      const next = new Set(prev);
      if (checked) next.add(id);
      else next.delete(id);
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
          // null = unassign, name = assign. Until folders load we can't tell
          // "cleared" from "not yet seeded", so omit the key (= keep current).
          folder: folderSeeded ? folder.trim() || null : undefined,
          filters: urlFilters.map((f) => f.trim()).filter(Boolean),
          // Pause is toggled from the feed's row menu, not this form.
          is_paused: null,
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
        required
      />
      <FolderCombobox value={folder} onChange={setFolder} />
      <div className="url-filters">
        {urlFilters.map((filter, i) => (
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
          onClick={() => setUrlFilters([...urlFilters, ""])}
        >
          + Filter
        </button>
      </div>
      <div className="feed-webhooks-section">
        <span className="feed-webhooks-label">Webhooks</span>
        <FeedWebhooks selected={selectedWebhooks} onToggle={toggleWebhook} />
      </div>
      {update.isError && <div className="form-error">{update.error.message}</div>}
      <div className="entity-form-actions">
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
