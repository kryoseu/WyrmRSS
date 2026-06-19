import { type SubmitEvent, useState } from "react";
import { useUpdateWebhook } from "../../hooks/useWebhooks";
import type { UpdateWebhook } from "../../types/UpdateWebhook";
import type { Webhook } from "../../types/Webhook";
import type { WebhookKind } from "../../types/WebhookKind";
import { DEFAULT_WEBHOOK_TEMPLATE } from "../../utils/webhook";

interface Props {
  webhook: Webhook;
  onClose: () => void;
}

export function EditWebhookForm({ webhook, onClose }: Props) {
  const [name, setName] = useState(webhook.name);
  const [kind, setKind] = useState<WebhookKind>(webhook.kind);
  const [url, setUrl] = useState(webhook.url);
  const [template, setTemplate] = useState(webhook.payload_template ?? "");
  const update = useUpdateWebhook();

  const isCustom = kind === "custom";

  function selectKind(next: WebhookKind) {
    setKind(next);
    if (next === "custom" && template.trim() === "") setTemplate(DEFAULT_WEBHOOK_TEMPLATE);
  }

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    const body: UpdateWebhook = { name, url, kind };
    if (isCustom) {
      body.payload_template = template;
    }
    update.mutate({ id: webhook.id, body }, { onSuccess: onClose });
  }

  return (
    <form className="add-feed-form add-webhook-form" onSubmit={handleSubmit}>
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="Name"
        required
        autoFocus
      />
      <select value={kind} onChange={(e) => selectKind(e.target.value as WebhookKind)}>
        <option value="discord">Discord</option>
        <option value="slack">Slack</option>
        <option value="custom">Custom</option>
      </select>
      <input
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        placeholder="Webhook URL"
        type="url"
        required
      />
      {isCustom && (
        <>
          <textarea
            className="webhook-template"
            value={template}
            onChange={(e) => setTemplate(e.target.value)}
            placeholder="JSON payload template"
            rows={8}
            spellCheck={false}
          />
          <p className="webhook-vars">
            Variables: <code>{"${feed.title}"}</code> <code>{"${feed.url}"}</code>{" "}
            <code>{"${feed.tag}"}</code> <code>{"${posts}"}</code>{" "}
            <code>{"${posts_count}"}</code>
          </p>
        </>
      )}
      {update.isError && (
        <p className="form-error">{(update.error as Error).message}</p>
      )}
      <div className="add-feed-form-actions">
        <button className="btn btn-primary" type="submit" disabled={update.isPending}>
          {update.isPending ? "Saving…" : "Save"}
        </button>
        <button className="btn btn-ghost" type="button" onClick={onClose}>
          Cancel
        </button>
      </div>
    </form>
  );
}
