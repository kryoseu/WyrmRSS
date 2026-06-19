import { type SubmitEvent, useState } from "react";
import { useCreateWebhook } from "../../hooks/useWebhooks";
import type { CreateWebhook } from "../../types/CreateWebhook";
import type { WebhookKind } from "../../types/WebhookKind";
import { DEFAULT_WEBHOOK_TEMPLATE } from "../../utils/webhook";

interface Props {
  onClose: () => void;
}

export function AddWebhookForm({ onClose }: Props) {
  const [name, setName] = useState("");
  const [kind, setKind] = useState<WebhookKind>("discord");
  const [url, setUrl] = useState("");
  const [template, setTemplate] = useState("");
  const create = useCreateWebhook();

  const isCustom = kind === "custom";

  function selectKind(next: WebhookKind) {
    setKind(next);
    // Prefill a working template the first time Custom is selected.
    if (next === "custom" && template.trim() === "") setTemplate(DEFAULT_WEBHOOK_TEMPLATE);
  }

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    const body: CreateWebhook = { name, url, kind };
    if (isCustom) {
      body.payload_template = template;
    }
    create.mutate(body, { onSuccess: onClose });
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
      {create.isError && (
        <p className="form-error">{(create.error as Error).message}</p>
      )}
      <div className="add-feed-form-actions">
        <button className="btn btn-primary" type="submit" disabled={create.isPending}>
          {create.isPending ? "Adding…" : "Add"}
        </button>
        <button className="btn btn-ghost" type="button" onClick={onClose}>
          Cancel
        </button>
      </div>
    </form>
  );
}
