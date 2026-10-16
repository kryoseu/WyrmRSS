import { useState } from "react";
import { useResetShortcut, useShortcuts, useUpdateShortcut } from "../../hooks/useShortcuts";
import type { ShortcutAction } from "../../types/ShortcutAction";
import { keyFromEvent, keyLabels, SHORTCUT_LABELS } from "../../utils/shortcuts";

const ACTIONS = Object.keys(SHORTCUT_LABELS) as ShortcutAction[];

function Keys({ shortcut }: { shortcut: string }) {
  return keyLabels(shortcut).map((part) => <kbd key={part}>{part}</kbd>);
}

export function Shortcuts() {
  const { data: shortcuts } = useShortcuts();
  const update = useUpdateShortcut();
  const reset = useResetShortcut();
  const [recording, setRecording] = useState<ShortcutAction | null>(null);
  const [error, setError] = useState<{ action: ShortcutAction; message: string } | null>(null);

  if (!shortcuts) return null;

  function record(action: ShortcutAction, e: React.KeyboardEvent) {
    // Tab moves focus on as usual, and the blur cancels recording.
    if (e.key === "Tab") return;
    // Handled here, so the window shortcut handlers skip this keypress.
    e.preventDefault();
    const key = keyFromEvent(e.nativeEvent);
    // A lone modifier: wait for the key it goes with.
    if (!key) return;
    setRecording(null);
    if (e.key === "Escape") return;
    const owner = ACTIONS.find((other) => other !== action && shortcuts?.[other] === key);
    if (owner) {
      setError({ action, message: `Already used by ${SHORTCUT_LABELS[owner]}` });
      return;
    }
    setError(null);
    update.mutate({ action, key }, { onError: (err) => setError({ action, message: err.message }) });
  }

  function resetToDefault(action: ShortcutAction) {
    setError(null);
    reset.mutate(action, { onError: (err) => setError({ action, message: err.message }) });
  }

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">Keyboard shortcuts</h2>
      <p className="settings-hint shortcut-intro-hint">
        Press <Keys shortcut={shortcuts.show_help} /> anywhere to see all shortcuts.
      </p>
      {ACTIONS.map((action) => (
        <div key={action}>
          <div className="settings-field">
            <label>{SHORTCUT_LABELS[action]}</label>
            {recording === action ? (
              <button
                className="btn btn-ghost shortcut-recording"
                autoFocus
                onKeyDown={(e) => record(action, e)}
                onBlur={() => setRecording(null)}
              >
                Press a key…
              </button>
            ) : (
              <span className="shortcut-keys">
                <Keys shortcut={shortcuts[action]} />
              </span>
            )}
            <button className="btn btn-ghost shortcut-btn" onClick={() => setRecording(action)}>
              Change
            </button>
            <button className="btn btn-ghost shortcut-btn" onClick={() => resetToDefault(action)}>
              Reset
            </button>
          </div>
          {error?.action === action && <p className="settings-status settings-status-err">{error.message}</p>}
        </div>
      ))}
      <p className="settings-hint shortcut-footer-hint">Press Escape to cancel. Reset restores the default key.</p>
    </div>
  );
}
