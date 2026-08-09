import { useState } from "react";
import type { FeedWebhookView } from "../../types/FeedWebhookView";
import type { WebhookId } from "../../types/WebhookId";

interface Props {
  /** All webhooks available to attach (attachment state lives in `selected`, not here). */
  all: FeedWebhookView[] | undefined;
  /** Currently-selected webhook ids (staged; committed by the parent (EditFeedForm) on save). */
  selected: Set<WebhookId>;
  onToggle: (id: WebhookId, checked: boolean) => void;
}

// Only show the filter box once the webhook list is long enough.
const FILTER_THRESHOLD = 8;

/** Controlled checklist of all webhooks; the parent (EditFeedForm) owns the selection state. */
export function FeedWebhooks({ all, selected, onToggle }: Props) {
  const [filter, setFilter] = useState("");

  if (!all?.length) {
    return (
      <p className="feed-webhooks-empty">
        Create a webhook in Settings
      </p>
    );
  }

  const query = filter.toLowerCase();
  const visible = all.filter((w) => w.name.toLowerCase().includes(query));

  return (
    <>
      {all.length > FILTER_THRESHOLD && (
        <input
          className="feed-webhooks-filter"
          placeholder="Filter webhooks…"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
      )}
      <div className="feed-webhooks">
        {visible.map((webhook) => (
          <label key={webhook.id} className="feed-webhook-option">
            <input
              type="checkbox"
              checked={selected.has(webhook.id)}
              onChange={(e) => onToggle(webhook.id, e.target.checked)}
            />
            <span className="feed-webhook-name">{webhook.name}</span>
            <span className={`webhook-kind webhook-kind-${webhook.kind}`}>{webhook.kind}</span>
          </label>
        ))}
        {visible.length === 0 && <p className="feed-webhooks-empty">No webhooks match.</p>}
      </div>
    </>
  );
}
