import type { HostCredentialKind } from "../types/HostCredentialKind";

/** Placeholders for `name` and `secret`, which mean different things per kind. */
export function credentialFieldLabels(kind: HostCredentialKind) {
  return kind === "basic_auth"
    ? { name: "Username", secret: "Password" }
    : { name: "Header name, e.g. X-Api-Key", secret: "Header value" };
}
