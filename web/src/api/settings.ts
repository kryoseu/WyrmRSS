import type { Settings } from "../types/Settings";
import type { UpdateSettings } from "../types/UpdateSettings";
import { ENDPOINTS, json, noContent } from "../utils/api";

export const getSettings = (): Promise<Settings> =>
  fetch(ENDPOINTS.settings.get()).then(json<Settings>);

export const updateSettings = (body: UpdateSettings): Promise<Settings> =>
  fetch(ENDPOINTS.settings.update(), {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(json<Settings>);

export const importOpml = (file: File): Promise<void> =>
  fetch(ENDPOINTS.settings.import(), {
    method: "POST",
    headers: { "Content-Type": "application/xml" },
    body: file,
  }).then(noContent);

