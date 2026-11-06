import { type SubmitEvent, useState } from "react";
import { useSettings, useUpdateSettings } from "../../hooks/useSettings";

function RSSHubForm({ initialInstance }: { initialInstance: string }) {
  const [instance, setInstance] = useState(initialInstance);
  const update = useUpdateSettings();

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    update.mutate({ rsshub_instance: instance.trim() });
  }

  return (
    <form className="settings-section" onSubmit={handleSubmit}>
      <h2 className="settings-section-title">RSSHub</h2>
      <div className="settings-field">
        <label htmlFor="rsshub-instance">Instance URL</label>
        <input
          id="rsshub-instance"
          type="url"
          value={instance}
          onChange={(e) => setInstance(e.target.value)}
          placeholder="https://rsshub.app"
        />
      </div>
      <p className="settings-hint">
        Resolve rsshub:// feed links using this instance. Leave blank to disable.
      </p>
      <div className="settings-form-actions">
        <button className="btn btn-primary" type="submit" disabled={update.isPending}>
          {update.isPending ? "Saving…" : "Save"}
        </button>
        {update.isSuccess && <span className="settings-status settings-status-ok">Saved.</span>}
        {update.isError && <span className="settings-status settings-status-err">Failed to save.</span>}
      </div>
      <p className="settings-hint">Changes take effect immediately.</p>
    </form>
  );
}

export function Integrations() {
  const { data } = useSettings();
  if (!data) return null;
  return <RSSHubForm initialInstance={data.rsshub_instance ?? ""} />;
}
