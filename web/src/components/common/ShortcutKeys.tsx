import { keyLabels } from "../../utils/shortcuts";

/** A stored key as keycaps, e.g. "shift+j" → [Shift] [J]. */
export function ShortcutKeys({ shortcut }: { shortcut: string }) {
  return keyLabels(shortcut).map((part) => <kbd key={part}>{part}</kbd>);
}
