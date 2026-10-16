import { useState } from "react";
import { useResetShortcut, useShortcuts, useUpdateShortcut } from "../../hooks/useShortcuts";
import type { ShortcutAction } from "../../types/ShortcutAction";
import type { Shortcuts as ShortcutBindings } from "../../types/Shortcuts";
import { keyFromEvent, SHORTCUT_ACTIONS, SHORTCUT_LABELS } from "../../utils/shortcuts";
import { ShortcutKeys } from "../common/ShortcutKeys";

interface RowProps {
  action: ShortcutAction;
  shortcuts: ShortcutBindings;
}

function ShortcutRow({ action, shortcuts }: RowProps) {
  const update = useUpdateShortcut();
  const reset = useResetShortcut();
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function record(e: React.KeyboardEvent) {
    // Tab moves focus on as usual, and the blur cancels recording.
    if (e.key === "Tab") return;
    // Handled here, so the window shortcut handlers skip this keypress.
    e.preventDefault();
    const key = keyFromEvent(e.nativeEvent);
    // A lone modifier: wait for the key it goes with.
    if (!key) return;
    setRecording(false);
    if (e.key === "Escape") return;
    const owner = SHORTCUT_ACTIONS.find((other) => other !== action && shortcuts[other] === key);
    if (owner) {
      setError(`Already used by ${SHORTCUT_LABELS[owner]}`);
      return;
    }
    setError(null);
    update.mutate({ action, key }, { onError: (err) => setError(err.message) });
  }

  function resetToDefault() {
    setError(null);
    reset.mutate(action, { onError: (err) => setError(err.message) });
  }

  return (
    <div>
      <div className="settings-field">
        <label>{SHORTCUT_LABELS[action]}</label>
        {recording ? (
          <button
            className="btn btn-ghost shortcut-recording"
            autoFocus
            onKeyDown={record}
            onBlur={() => setRecording(false)}
          >
            Press a key…
          </button>
        ) : (
          <span className="shortcut-keys">
            <ShortcutKeys shortcut={shortcuts[action]} />
          </span>
        )}
        <button className="btn btn-ghost shortcut-btn" onClick={() => setRecording(true)}>
          Change
        </button>
        <button className="btn btn-ghost shortcut-btn" onClick={resetToDefault}>
          Reset
        </button>
      </div>
      {error && <p className="settings-status settings-status-err">{error}</p>}
    </div>
  );
}

export function Shortcuts() {
  const { data: shortcuts } = useShortcuts();
  if (!shortcuts) return null;

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">Keyboard shortcuts</h2>
      <p className="settings-hint shortcut-intro-hint">
        Press <ShortcutKeys shortcut={shortcuts.show_help} /> anywhere to see all shortcuts.
      </p>
      {SHORTCUT_ACTIONS.map((action) => (
        <ShortcutRow key={action} action={action} shortcuts={shortcuts} />
      ))}
      <p className="settings-hint shortcut-footer-hint">Press Escape to cancel. Reset restores the default key.</p>
    </div>
  );
}
