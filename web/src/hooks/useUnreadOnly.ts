import { useState } from "react";
import { useDemoMode } from "./useSettings";

const STORAGE_KEY = "wyrm:unread-only";

/**
 * Whether post lists show only unread posts (the default) or everything.
 * Persisted in localStorage so the choice survives refreshes and navigation.
 */
export function useUnreadOnly() {
  const isDemo = useDemoMode();
  const [stored, setStored] = useState(() => localStorage.getItem(STORAGE_KEY));

  const setUnreadOnly = (value: boolean) => {
    localStorage.setItem(STORAGE_KEY, String(value));
    setStored(String(value));
  };

  // Demo visitors share read state, so default to everything there.
  const unreadOnly = stored !== null ? stored !== "false" : !isDemo;

  return { unreadOnly, setUnreadOnly };
}
