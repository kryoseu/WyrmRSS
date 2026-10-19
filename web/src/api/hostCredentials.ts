import type { HostCredential } from "../types/HostCredential";
import type { HostCredentialId } from "../types/HostCredentialId";
import type { CreateHostCredential } from "../types/CreateHostCredential";
import type { UpdateHostCredential } from "../types/UpdateHostCredential";
import { ENDPOINTS } from "../utils/api";
import { fetchWithAuth } from "../utils/auth";
import { ApiError } from "../utils/error";
import { json } from "../utils/response";

export const getHostCredentials = (): Promise<HostCredential[]> =>
  fetchWithAuth(ENDPOINTS.hostCredentials.list()).then<HostCredential[]>(json);

export const createHostCredential = (body: CreateHostCredential): Promise<HostCredential> =>
  fetchWithAuth(ENDPOINTS.hostCredentials.create(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<HostCredential>(json);

export const updateHostCredential = (
  id: HostCredentialId,
  body: UpdateHostCredential,
): Promise<HostCredential> =>
  fetchWithAuth(ENDPOINTS.hostCredentials.update(id), {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  }).then<HostCredential>(json);

export const deleteHostCredential = (id: HostCredentialId): Promise<HostCredential> =>
  fetchWithAuth(ENDPOINTS.hostCredentials.delete(id), { method: "DELETE" }).then<HostCredential>(
    json,
  );

/**
 * User-facing message for a failed save. A duplicate host surfaces as a bare
 * 409 "conflict"; validation errors (400) already carry a readable message.
 */
export function hostCredentialErrorMessage(error: Error): string {
  return error instanceof ApiError && error.status === 409
    ? "Credentials for this host already exist."
    : error.message;
}
