import { useState } from "react";
import { DEFAULTS } from "../utils/constants";

const STORAGE_KEY = "wyrm:hide-caught-up";

/**
 * Whether the feed list hides feeds with no unread posts.
 * Persisted per device in localStorage, like useUnreadOnly.
 */
export function useHideCaughtUp() {
  const [stored, setStored] = useState(() => localStorage.getItem(STORAGE_KEY));

  const setHideCaughtUp = (value: boolean) => {
    localStorage.setItem(STORAGE_KEY, String(value));
    setStored(String(value));
  };

  const hideCaughtUp = stored !== null ? stored === "true" : DEFAULTS.hideCaughtUp;

  return { hideCaughtUp, setHideCaughtUp };
}
