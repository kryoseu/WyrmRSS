import { useState } from "react";
import { TbEdit, TbPlus, TbTrash } from "react-icons/tb";
import { useDeleteHostCredential, useHostCredentials } from "../../hooks/useHostCredentials";
import { AddCredentialForm } from "../credentials/AddCredentialForm";
import { EditCredentialForm } from "../credentials/EditCredentialForm";
import type { HostCredential } from "../../types/HostCredential";

function CredentialRow({ credential }: { credential: HostCredential }) {
  const [editing, setEditing] = useState(false);
  const { mutate: deleteCredential } = useDeleteHostCredential();

  return (
    <>
      <div className={`settings-table-row${editing ? " editing" : ""}`}>
        <span className="settings-table-title" title={credential.host}>
          {credential.host}
        </span>
        <span className="settings-table-badge">
          <span className="credential-kind">
            {credential.kind === "basic_auth" ? "basic" : "header"}
          </span>
        </span>
        <span className="settings-table-url" title={credential.name}>
          {credential.name}
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
            onClick={() => deleteCredential(credential.id)}
            title="Delete"
          >
            <TbTrash />
          </button>
        </div>
      </div>
      {editing && (
        <div className="settings-table-edit">
          <EditCredentialForm credential={credential} onClose={() => setEditing(false)} />
        </div>
      )}
    </>
  );
}

export function Credentials() {
  const { data: credentials, isLoading } = useHostCredentials();
  const [search, setSearch] = useState("");
  const [adding, setAdding] = useState(false);

  if (isLoading) return <div className="pane-empty">Loading…</div>;

  const query = search.toLowerCase();
  const filtered = (credentials ?? [])
    .filter((c) => c.host.includes(query) || c.name.toLowerCase().includes(query))
    .sort((a, b) => a.host.localeCompare(b.host));

  return (
    <div className="settings-table-wrap">
      <div className="settings-table-toolbar">
        <input
          className="settings-table-search"
          placeholder="Search credentials…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <button
          className={`btn ${adding ? "btn-ghost" : "btn-primary"} settings-table-add-btn`}
          onClick={() => setAdding((a) => !a)}
        >
          <TbPlus /> {adding ? "Cancel" : "Add credential"}
        </button>
      </div>
      {adding && (
        <div className="settings-table-edit">
          <AddCredentialForm onClose={() => setAdding(false)} />
        </div>
      )}
      {!credentials?.length ? (
        <div className="settings-empty">No credentials yet.</div>
      ) : (
        <div className="settings-table credentials-table">
          <div className="settings-table-header">
            <span className="settings-table-title">Host</span>
            <span className="settings-table-badge">Kind</span>
            <span className="settings-table-url">Name</span>
            <span className="settings-table-actions" />
          </div>
          {filtered.map((credential) => (
            <CredentialRow key={credential.id} credential={credential} />
          ))}
          {filtered.length === 0 && <div className="pane-empty">No credentials match.</div>}
        </div>
      )}
    </div>
  );
}
