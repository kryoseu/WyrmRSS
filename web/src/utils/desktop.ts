import { isTauri, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { setApiOrigin } from "./api";
import { setApiKey } from "./auth";

interface ServerInfo {
  base_url: string;
  api_key: string;
}

// The embedded backend starts asynchronously after the window opens and
// emits `wyrm://backend-ready` once `server_info` is populated. Check once up
// front in case it's already ready, then wait for that event instead of
// polling.
async function waitForServerInfo(timeoutMs = 30_000): Promise<ServerInfo> {
  const existing = await invoke<ServerInfo | null>("server_info");
  if (existing) return existing;

  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      void unlisten.then((fn) => fn());
      reject(new Error("timed out waiting for embedded backend"));
    }, timeoutMs);

    const unlisten = listen("wyrm://backend-ready", async () => {
      clearTimeout(timer);
      (await unlisten)();
      const info = await invoke<ServerInfo | null>("server_info");
      if (info) resolve(info);
      else reject(new Error("backend-ready fired without server info"));
    });
  });
}

// No-op outside the desktop build (self-hosted keeps its same-origin default).
// External links are handled by the Rust-side on_navigation hook (lib.rs),
// not here -- a JS click listener can't catch a context-menu "Open Link".
export async function initDesktopRuntime(): Promise<void> {
  if (!isTauri()) return;
  const info = await waitForServerInfo();
  setApiOrigin(info.base_url);
  setApiKey(info.api_key);
}
