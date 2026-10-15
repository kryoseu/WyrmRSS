import type { ShortcutAction } from "../types/ShortcutAction";
import type { Shortcuts } from "../types/Shortcuts";
import type { UpdateShortcut } from "../types/UpdateShortcut";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import { json } from "../utils/response";

export const getShortcuts = (): Promise<Shortcuts> =>
  fetchWithAuth(ENDPOINTS.shortcuts.list()).then(json<Shortcuts>);

export const updateShortcut = (action: ShortcutAction, body: UpdateShortcut): Promise<Shortcuts> =>
  fetchWithAuth(ENDPOINTS.shortcuts.update(action), {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(json<Shortcuts>);

/** Drops the user's override, restoring the action's default key. */
export const resetShortcut = (action: ShortcutAction): Promise<Shortcuts> =>
  fetchWithAuth(ENDPOINTS.shortcuts.reset(action), { method: "DELETE" }).then(json<Shortcuts>);
