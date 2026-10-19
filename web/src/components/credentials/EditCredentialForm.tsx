import { type SubmitEvent, useState } from "react";
import { hostCredentialErrorMessage } from "../../api/hostCredentials";
import { useUpdateHostCredential } from "../../hooks/useHostCredentials";
import type { HostCredential } from "../../types/HostCredential";
import type { HostCredentialKind } from "../../types/HostCredentialKind";
import { credentialFieldLabels } from "../../utils/hostCredential";
import { IS_TOUCH } from "../../utils/utils";
import { CredentialHostHint } from "./CredentialHostHint";
import { SecretInput } from "./SecretInput";

interface Props {
  credential: HostCredential;
  onClose: () => void;
}

export function EditCredentialForm({ credential, onClose }: Props) {
  const [host, setHost] = useState(credential.host);
  const [kind, setKind] = useState<HostCredentialKind>(credential.kind);
  const [name, setName] = useState(credential.name);
  // The API never returns the secret; left empty, the server keeps the saved one.
  const [secret, setSecret] = useState("");
  const update = useUpdateHostCredential();

  const labels = credentialFieldLabels(kind);
  // A password reused as a header value (or vice versa) is almost certainly wrong.
  const kindChanged = kind !== credential.kind;

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    update.mutate({ id: credential.id, body: { host, kind, name, secret } }, { onSuccess: onClose });
  }

  return (
    <form className="entity-form" onSubmit={handleSubmit}>
      <input
        value={host}
        onChange={(e) => setHost(e.target.value)}
        placeholder="Host, e.g. feeds.example.com"
        required
        autoFocus={!IS_TOUCH}
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
        placeholder={kindChanged ? labels.secret : `${labels.secret} saved — type to replace`}
        autoComplete="new-password"
        required={kindChanged}
      />
      {update.isError && (
        <p className="form-error">{hostCredentialErrorMessage(update.error as Error)}</p>
      )}
      <div className="entity-form-actions">
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
