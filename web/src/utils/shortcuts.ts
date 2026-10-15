import type { ShortcutAction } from "../types/ShortcutAction";

// Display order for the settings tab and the help dialog.
export const SHORTCUT_LABELS: Record<ShortcutAction, string> = {
  next_post: "Next post",
  prev_post: "Previous post",
  next_post_in_feed: "Next post in same feed",
  prev_post_in_feed: "Previous post in same feed",
  close_reader: "Close reader",
  toggle_read: "Mark read / unread",
  toggle_read_later: "Read later",
  toggle_archive: "Archive / unarchive",
  open_original: "Open original",
  focus_search: "Search",
  show_help: "Show shortcuts",
};

const MODIFIER_KEYS = new Set(["Shift", "Control", "Alt", "Meta"]);

// Keys that can't be stored as typed: " " is invisible and "+" is the separator.
const RENAMED_KEYS: Record<string, string> = { " ": "Space", "+": "Plus" };

/**
 * The stored key for a keypress, e.g. "j", "shift+j", "?", "ctrl+k", "Escape".
 * Returns null for a lone modifier.
 */
export function keyFromEvent(e: KeyboardEvent): string | null {
  if (MODIFIER_KEYS.has(e.key)) return null;
  const isLetter = e.key.length === 1 && e.key.toLowerCase() !== e.key.toUpperCase();
  const isNamedKey = e.key.length > 1;
  const key = RENAMED_KEYS[e.key] ?? (isLetter ? e.key.toLowerCase() : e.key);
  // Symbols already include shift ("?" is shift+/), so only letters and named keys get it.
  const shift = e.shiftKey && (isLetter || isNamedKey);
  const modifiers = [e.ctrlKey && "ctrl", e.altKey && "alt", e.metaKey && "meta", shift && "shift"].filter(Boolean);
  return [...modifiers, key].join("+");
}

const KEY_NAMES: Record<string, string> = {
  ctrl: "Ctrl",
  alt: "Alt",
  meta: "Meta",
  shift: "Shift",
  Plus: "+",
  Escape: "Esc",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
};

/** Splits a stored key for display, e.g. "shift+j" → ["Shift", "J"]. */
export function keyLabels(key: string): string[] {
  return key.split("+").map((part) => KEY_NAMES[part] ?? (part.length === 1 ? part.toUpperCase() : part));
}

/** True for keypresses meant for something else: a text field, a dialog, or an IME. */
export function shouldIgnoreKeypress(e: KeyboardEvent): boolean {
  if (e.defaultPrevented || e.isComposing) return true;
  if (!(e.target instanceof HTMLElement)) return false;
  return e.target.isContentEditable
    || ["INPUT", "TEXTAREA", "SELECT"].includes(e.target.tagName)
    || e.target.closest("dialog") !== null;
}
