import { useState } from "react";
import { useDemoMode } from "./useSettings";
import { DEFAULTS } from "../utils/constants";

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

  const unreadOnly = stored !== null ? stored !== "false" : DEFAULTS.unreadOnly[isDemo ? "demo" : "normal"];

  return { unreadOnly, setUnreadOnly };
}
