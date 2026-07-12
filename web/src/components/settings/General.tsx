import { useState } from "react";
import { useSettings, useUpdateSettings } from "../../hooks/useSettings";
import type { Settings } from "../../types/Settings";
import type { ReadMode } from "../../types/ReadMode";

const READ_MODE_OPTIONS: { value: ReadMode; label: string }[] = [
  { value: "on_open", label: "On open" },
  { value: "manually", label: "Manually" },
  { value: "disabled", label: "Disabled" },
];

function GeneralForm({ initial }: { initial: Settings }) {
  const [pageSize, setPageSize] = useState(String(initial.page_size));
  const [pollInterval, setPollInterval] = useState(String(initial.feed_poll_interval_secs));
  const [httpTimeout, setHttpTimeout] = useState(String(initial.http_timeout));
  const [connectTimeout, setConnectTimeout] = useState(String(initial.http_connect_timeout));
  const [retries, setRetries] = useState(String(initial.http_retries));
  const [userAgent, setUserAgent] = useState(initial.http_user_agent ?? "");
  const [readMode, setReadMode] = useState<ReadMode>(initial.read_mode);
  const [expireReadAfterDays, setExpireReadAfterDays] = useState(String(initial.expire_read_after_days ?? ""));
  const [expireUnreadAfterDays, setExpireUnreadAfterDays] = useState(String(initial.expire_unread_after_days ?? ""));

  const update = useUpdateSettings();

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    update.mutate({
      page_size: parseInt(pageSize),
      feed_poll_interval_secs: parseInt(pollInterval),
      http_timeout: parseInt(httpTimeout),
      http_connect_timeout: parseInt(connectTimeout),
      http_retries: parseInt(retries),
      http_user_agent: userAgent.trim() || null,
      read_mode: readMode,
      expire_read_after_days: parseInt(expireReadAfterDays),
      expire_unread_after_days: parseInt(expireUnreadAfterDays),
    });
  }

  return (
    <form className="settings-section" onSubmit={handleSubmit}>
      <h2 className="settings-section-title">Reading</h2>
      <div className="settings-field">
        <label>Mark as read</label>
        <div className="segmented">
          {READ_MODE_OPTIONS.map(({ value, label }) => (
            <button
              key={value}
              type="button"
              className={`segmented-btn${readMode === value ? " active" : ""}`}
              onClick={() => setReadMode(value)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>
      <div className="settings-field">
        <label>Page size</label>
        <input type="number" min={1} value={pageSize} onChange={(e) => setPageSize(e.target.value)} required />
      </div>
      <div className="settings-field">
        <label>Expire read posts after</label>
        <input type="number" min={1} placeholder="Never expire" value={expireReadAfterDays} onChange={(e) => setExpireReadAfterDays(e.target.value)} />
        <label>days</label>
      </div>
      <div className="settings-field">
        <label>Expire unread posts after</label>
        <input type="number" min={1} placeholder="Never expire" value={expireUnreadAfterDays} onChange={(e) => setExpireUnreadAfterDays(e.target.value)} />
        <label>days</label>
      </div>

      <h2 className="settings-section-title">Polling</h2>
      <div className="settings-field">
        <label>Poll interval (s)</label>
        <input type="number" min={60} value={pollInterval} onChange={(e) => setPollInterval(e.target.value)} required />
      </div>

      <h2 className="settings-section-title">HTTP</h2>
      <div className="settings-field">
        <label>Timeout (s)</label>
        <input type="number" min={1} value={httpTimeout} onChange={(e) => setHttpTimeout(e.target.value)} required />
      </div>
      <div className="settings-field">
        <label>Connect timeout (s)</label>
        <input type="number" min={1} value={connectTimeout} onChange={(e) => setConnectTimeout(e.target.value)} required />
      </div>
      <div className="settings-field">
        <label>Retries</label>
        <input type="number" min={0} value={retries} onChange={(e) => setRetries(e.target.value)} required />
      </div>
      <div className="settings-field">
        <label>User agent</label>
        <input
          type="text"
          value={userAgent}
          onChange={(e) => setUserAgent(e.target.value)}
          placeholder="Default (wyrm-rss/version)"
        />
      </div>

      <div className="settings-form-actions">
        <button className="btn btn-primary" type="submit" disabled={update.isPending}>
          {update.isPending ? "Saving…" : "Save"}
        </button>
        {update.isSuccess && <span className="settings-status settings-status-ok">Saved.</span>}
        {update.isError && <span className="settings-status settings-status-err">Failed to save.</span>}
      </div>
      <p className="settings-hint">All changes take effect immediately.</p>
    </form>
  );
}

export function General() {
  const { data } = useSettings();
  if (!data) return null;
  return <GeneralForm initial={data} />;
}
