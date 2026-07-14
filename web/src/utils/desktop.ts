import { isTauri, invoke } from "@tauri-apps/api/core";
import { setApiOrigin } from "./api";
import { setApiKey } from "./auth";

interface ServerInfo {
  base_url: string;
  api_key: string;
}

// The embedded backend starts asynchronously after the window opens, so
// `server_info` returns null until it's ready. Poll briefly rather than
// requiring the Rust side to block window creation on Postgres/migrations.
async function waitForServerInfo(timeoutMs = 30_000, intervalMs = 150): Promise<ServerInfo> {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const info = await invoke<ServerInfo | null>("server_info");
    if (info) return info;
    if (Date.now() > deadline) throw new Error("timed out waiting for embedded backend");
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
}

// No-op outside the desktop build (self-hosted keeps its same-origin default).
export async function initDesktopRuntime(): Promise<void> {
  if (!isTauri()) return;
  const info = await waitForServerInfo();
  setApiOrigin(info.base_url);
  setApiKey(info.api_key);
}
