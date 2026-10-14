import type { SettingsResponse } from "../types/SettingsResponse";
import type { UpdateSettings } from "../types/UpdateSettings";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import { json, noContent } from "../utils/response";

export const getSettings = (): Promise<SettingsResponse> =>
  fetchWithAuth(ENDPOINTS.settings.get()).then(json<SettingsResponse>);

export const updateSettings = (body: UpdateSettings): Promise<SettingsResponse> =>
  fetchWithAuth(ENDPOINTS.settings.update(), {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then(json<SettingsResponse>);

export const importOpml = (file: File): Promise<void> =>
  fetchWithAuth(ENDPOINTS.settings.import(), {
    method: "POST",
    headers: { "Content-Type": "application/xml" },
    body: file,
  }).then(noContent);
