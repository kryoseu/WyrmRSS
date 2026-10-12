import { useSyncExternalStore } from "react";

// Phone-width layout. The desktop app's window has an 800px minimum width,
// so this only ever matches in a browser.
const MOBILE_QUERY = window.matchMedia("(max-width: 768px)");

function subscribe(onChange: () => void) {
  MOBILE_QUERY.addEventListener("change", onChange);
  return () => MOBILE_QUERY.removeEventListener("change", onChange);
}

/** Whether to render the phone layout (drawer, tab bar, full-screen reader). */
export function useIsMobile(): boolean {
  return useSyncExternalStore(subscribe, () => MOBILE_QUERY.matches);
}
