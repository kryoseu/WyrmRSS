import { useRef } from "react";
import { TbX } from "react-icons/tb";
import { useShortcutHandler, useShortcuts } from "../../hooks/useShortcuts";
import { SHORTCUT_ACTIONS, SHORTCUT_LABELS } from "../../utils/shortcuts";
import { ShortcutKeys } from "../common/ShortcutKeys";

export function ShortcutsHelp() {
  const { data: shortcuts } = useShortcuts();
  const dialogRef = useRef<HTMLDialogElement>(null);

  // showModal() focuses the "x" button and shows a focus ring, so focus the dialog instead.
  useShortcutHandler("show_help", () => {
    dialogRef.current?.showModal();
    dialogRef.current?.focus();
  });

  // Clicks outside the box land on the dialog element itself.
  function closeOnBackdropClick(e: React.MouseEvent<HTMLDialogElement>) {
    if (e.target === e.currentTarget) e.currentTarget.close();
  }

  return (
    <dialog className="shortcuts-help" ref={dialogRef} tabIndex={-1} onClick={closeOnBackdropClick}>
      <div className="shortcuts-help-content">
        <div className="shortcuts-help-header">
          <h2 className="settings-section-title">Keyboard shortcuts</h2>
          <button className="pane-reader-close" onClick={() => dialogRef.current?.close()} aria-label="Close">
            <TbX />
          </button>
        </div>
        {shortcuts && (
          <dl className="shortcuts-help-list">
            {SHORTCUT_ACTIONS.map((action) => (
              <div key={action} className="shortcuts-help-row">
                <dt>{SHORTCUT_LABELS[action]}</dt>
                <dd><ShortcutKeys shortcut={shortcuts[action]} /></dd>
              </div>
            ))}
          </dl>
        )}
      </div>
    </dialog>
  );
}
