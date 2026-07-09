import { useState } from "react";
import { TbEdit, TbPlus, TbTrash } from "react-icons/tb";
import { useDeleteWebhook, useWebhooks } from "../../hooks/useWebhooks";
import { AddWebhookForm } from "../webhook/AddWebhookForm";
import { EditWebhookForm } from "../webhook/EditWebhookForm";
import type { Webhook } from "../../types/Webhook";

function WebhookRow({ webhook }: { webhook: Webhook }) {
  const [editing, setEditing] = useState(false);
  const { mutate: deleteWebhook } = useDeleteWebhook();

  return (
    <>
      <div className={`settings-table-row${editing ? " editing" : ""}`}>
        <span className="settings-table-title">{webhook.name}</span>
        <span className="settings-table-badge">
          <span className={`webhook-kind webhook-kind-${webhook.kind}`}>{webhook.kind}</span>
        </span>
        <span className="settings-table-url" title={webhook.url}>
          {webhook.url}
        </span>
        <div className="settings-table-actions">
          <button
            className={`settings-table-btn${editing ? " active" : ""}`}
            onClick={() => setEditing((e) => !e)}
            title="Edit"
          >
            <TbEdit />
          </button>
          <button
            className="settings-table-btn settings-table-btn-delete"
            onClick={() => deleteWebhook(webhook.id)}
            title="Delete"
          >
            <TbTrash />
          </button>
        </div>
      </div>
      {editing && (
        <div className="settings-table-edit">
          <EditWebhookForm webhook={webhook} onClose={() => setEditing(false)} />
        </div>
      )}
    </>
  );
}

export function Webhooks() {
  const { data: webhooks, isLoading } = useWebhooks();
  const [search, setSearch] = useState("");
  const [adding, setAdding] = useState(false);

  if (isLoading) return <div className="pane-empty">Loading…</div>;

  const query = search.toLowerCase();
  const filtered = (webhooks ?? [])
    .filter((w) => w.name.toLowerCase().includes(query) || w.url.toLowerCase().includes(query))
    .sort((a, b) => a.name.localeCompare(b.name));

  return (
    <div className="settings-table-wrap">
      <div className="settings-table-toolbar">
        <input
          className="settings-table-search"
          placeholder="Search webhooks…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <button
          className={`btn ${adding ? "btn-ghost" : "btn-primary"} settings-table-add-btn`}
          onClick={() => setAdding((a) => !a)}
        >
          <TbPlus /> {adding ? "Cancel" : "Add webhook"}
        </button>
      </div>
      {adding && (
        <div className="settings-table-edit">
          <AddWebhookForm onClose={() => setAdding(false)} />
        </div>
      )}
      {!webhooks?.length ? (
        <div className="settings-empty">No webhooks yet.</div>
      ) : (
        <div className="settings-table webhooks-table">
          <div className="settings-table-header">
            <span className="settings-table-title">Name</span>
            <span className="settings-table-badge">Kind</span>
            <span className="settings-table-url">URL</span>
            <span className="settings-table-actions" />
          </div>
          {filtered.map((webhook) => (
            <WebhookRow key={webhook.id} webhook={webhook} />
          ))}
          {filtered.length === 0 && <div className="pane-empty">No webhooks match.</div>}
        </div>
      )}
    </div>
  );
}
