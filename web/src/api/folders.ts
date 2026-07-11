import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import type { Folder } from "../types/Folder";
import type { FolderId } from "../types/FolderId";
import type { CreateFolder } from "../types/CreateFolder";
import type { UpdateFolder } from "../types/UpdateFolder";
import { json } from "../utils/response";
import { ApiError } from "../utils/error";

export const getFolders = (): Promise<Folder[]> =>
  fetchWithAuth(ENDPOINTS.folders.list()).then<Folder[]>(json);

export const createFolder = (body: CreateFolder): Promise<Folder> =>
  fetchWithAuth(ENDPOINTS.folders.create(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Folder>(json);

export const updateFolder = (id: FolderId, body: UpdateFolder): Promise<Folder> =>
  fetchWithAuth(ENDPOINTS.folders.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<Folder>(json);

export const deleteFolder = (id: FolderId): Promise<Folder> =>
  fetchWithAuth(ENDPOINTS.folders.delete(id), { method: "DELETE" }).then<Folder>(json);

/**
 * User-facing message for a failed folder rename. Duplicate names surface as
 * a 409; anything else (network, 500) gets the generic message.
 */
export function renameErrorMessage(error: Error): string {
  return error instanceof ApiError && error.status === 409
    ? "A folder with that name already exists."
    : "Rename failed.";
}

