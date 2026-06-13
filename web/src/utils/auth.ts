export function getApiKey(): string | null {
  return import.meta.env.WYRM_API_KEY || localStorage.getItem("WYRM_API_KEY");
}

export function setApiKey(key: string): void {
  localStorage.setItem("WYRM_API_KEY", key);
}

export function clearApiKey(): void {
  localStorage.removeItem("WYRM_API_KEY");
}

export function fetchWithAuth(url: string, init: RequestInit = {}): Promise<Response> {
  const key = getApiKey();
  const headers = new Headers(init.headers);
  if (key) headers.set("x-api-key", `${key}`);
  return fetch(url, { ...init, headers });
}

export function handleUnauthorized(): void {
  clearApiKey();
  window.dispatchEvent(new Event("wyrm:unauthorized"));
}

export async function verifyApiKey(key: string): Promise<boolean> {
  const res = await fetch("/api/v1/auth/verify", {
    headers: { "x-api-key": `${key}` },
  });
  return res.ok;
}
