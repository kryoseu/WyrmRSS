import { useState } from "react";

const STORAGE_KEY = "wyrm:unread-only";

/**
 * Whether post lists show only unread posts (the default) or everything.
 * Persisted in localStorage so the choice survives refreshes and navigation.
 */
export function useUnreadOnly() {
  const [unreadOnly, setUnreadOnlyState] = useState(
    () => localStorage.getItem(STORAGE_KEY) !== "false",
  );

  const setUnreadOnly = (value: boolean) => {
    localStorage.setItem(STORAGE_KEY, String(value));
    setUnreadOnlyState(value);
  };

  return { unreadOnly, setUnreadOnly };
}
