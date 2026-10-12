import { ENDPOINTS } from "./api";

export function getApiKey(): string | null {
  return import.meta.env.WYRM_API_KEY || localStorage.getItem("WYRM_API_KEY");
}

export function setApiKey(key: string): void {
  localStorage.setItem("WYRM_API_KEY", key);
}

export function clearApiKey(): void {
  localStorage.removeItem("WYRM_API_KEY");
}

// A reverse proxy answers with these when the backend behind it is down.
// Not 504: the server itself sends that when a manual poll outlasts the HTTP timeout.
const GATEWAY_ERRORS = new Set([502, 503]);

// Raises "wyrm:unreachable" when the server can't be reached and
// "wyrm:reachable" once it answers again (see ServerStatusBanner).
export async function fetchWithAuth(url: string, init: RequestInit = {}): Promise<Response> {
  const key = getApiKey();
  const headers = new Headers(init.headers);
  if (key) headers.set("x-api-key", `${key}`);
  let res: Response;
  try {
    res = await fetch(url, { ...init, headers });
  } catch (e) {
    // An aborted request was cancelled on purpose, not lost.
    if (!(e instanceof DOMException && e.name === "AbortError")) {
      window.dispatchEvent(new Event("wyrm:unreachable"));
    }
    throw e;
  }
  const reachable = !GATEWAY_ERRORS.has(res.status);
  window.dispatchEvent(new Event(reachable ? "wyrm:reachable" : "wyrm:unreachable"));
  return res;
}

export function handleUnauthorized(): void {
  clearApiKey();
  window.dispatchEvent(new Event("wyrm:unauthorized"));
}

export async function verifyApiKey(key: string): Promise<boolean> {
  const res = await fetch(ENDPOINTS.auth.verify(), {
    headers: { "x-api-key": `${key}` },
  });
  return res.ok;
}
