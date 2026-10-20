import { type SubmitEvent, useState } from "react";
import { hostCredentialErrorMessage } from "../../api/hostCredentials";
import { useCreateHostCredential } from "../../hooks/useHostCredentials";
import type { HostCredentialKind } from "../../types/HostCredentialKind";
import { credentialFieldLabels } from "../../utils/hostCredential";
import { CredentialHostHint } from "./CredentialHostHint";
import { SecretInput } from "./SecretInput";

interface Props {
  onClose: () => void;
}

export function AddCredentialForm({ onClose }: Props) {
  const [host, setHost] = useState("");
  const [kind, setKind] = useState<HostCredentialKind>("basic_auth");
  const [name, setName] = useState("");
  const [secret, setSecret] = useState("");
  const create = useCreateHostCredential();

  const labels = credentialFieldLabels(kind);

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    create.mutate({ host, kind, name, secret }, { onSuccess: onClose });
  }

  return (
    <form className="entity-form" onSubmit={handleSubmit}>
      <input
        value={host}
        onChange={(e) => setHost(e.target.value)}
        placeholder="Host, e.g. feeds.example.com"
        required
        autoFocus
      />
      <CredentialHostHint />
      <select value={kind} onChange={(e) => setKind(e.target.value as HostCredentialKind)}>
        <option value="basic_auth">Basic auth</option>
        <option value="custom_header">Custom header</option>
      </select>
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder={labels.name}
        autoComplete="off"
        required
      />
      <SecretInput
        value={secret}
        onChange={(e) => setSecret(e.target.value)}
        placeholder={labels.secret}
        autoComplete="new-password"
        required
      />
      {create.isError && (
        <p className="form-error">{hostCredentialErrorMessage(create.error as Error)}</p>
      )}
      <div className="entity-form-actions">
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
