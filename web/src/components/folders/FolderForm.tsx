import { useState } from "react";

interface FolderFormProps {
  initial?: string;
  submitLabel: string;
  pendingLabel: string;
  isPending: boolean;
  error?: string;
  onSubmit: (name: string) => void;
  onCancel: () => void;
}

/**
 * Name input + Save/Cancel, shared by the Settings→Folders add/rename flows
 * and the sidebar inline rename.
 */
export function FolderForm({
  initial = "",
  submitLabel,
  pendingLabel,
  isPending,
  error,
  onSubmit,
  onCancel,
}: FolderFormProps) {
  const [name, setName] = useState(initial);

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    onSubmit(name);
  }

  return (
    <form className="entity-form" onSubmit={handleSubmit}>
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="Folder name"
        required
        autoFocus
      />
      {error && <div className="form-error">{error}</div>}
      <div className="entity-form-actions">
        <button
          className="btn btn-primary"
          type="submit"
          disabled={isPending || !name.trim()}
        >
          {isPending ? pendingLabel : submitLabel}
        </button>
        <button className="btn btn-ghost" type="button" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </form>
  );
}
