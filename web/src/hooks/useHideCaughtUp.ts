import { useState } from "react";
import { DEFAULTS, STORAGE_KEYS } from "../utils/constants";

/**
 * Whether the feed list hides feeds with no unread posts.
 * Persisted per device in localStorage, like useUnreadOnly.
 */
export function useHideCaughtUp() {
  const [stored, setStored] = useState(() => localStorage.getItem(STORAGE_KEYS.hideCaughtUp));

  const setHideCaughtUp = (value: boolean) => {
    localStorage.setItem(STORAGE_KEYS.hideCaughtUp, String(value));
    setStored(String(value));
  };

  const hideCaughtUp = stored !== null ? stored === "true" : DEFAULTS.hideCaughtUp;

  return { hideCaughtUp, setHideCaughtUp };
}
