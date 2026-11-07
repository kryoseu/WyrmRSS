import { type SubmitEvent, useState } from "react";
import { TbToggleLeft, TbToggleRightFilled } from "react-icons/tb";
import { useUpdateSettings } from "../../hooks/useSettings";

export function RSSHubForm({ initialInstance }: { initialInstance: string }) {
  const [enabled, setEnabled] = useState(initialInstance !== "");
  // Kept while toggled off so re-enabling before saving restores it.
  const [instance, setInstance] = useState(initialInstance);
  const update = useUpdateSettings();

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    update.mutate({ rsshub_instance: enabled ? instance.trim() || null : null });
  }

  return (
    <form className="settings-section" onSubmit={handleSubmit}>
      <h2 className="settings-section-title">RSSHub</h2>
      <label className="feed-paused-option">
        Enabled
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => setEnabled(e.target.checked)}
        />
        {enabled ? <TbToggleRightFilled aria-hidden /> : <TbToggleLeft aria-hidden />}
      </label>
      {enabled && (
        <div className="settings-field">
          <label htmlFor="rsshub-instance">Instance URL</label>
          <input
            id="rsshub-instance"
            type="url"
            required
            value={instance}
            onChange={(e) => setInstance(e.target.value)}
            placeholder="https://rsshub.app"
          />
        </div>
      )}
      <p className="settings-hint">
        Feeds added as rsshub:// links pause while this is off.
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
