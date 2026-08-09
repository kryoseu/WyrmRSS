import type { DisplayMode } from "../types/DisplayMode";

// Shared between DisplayModeSelect and the feeds settings table, so the raw
// "feed_only" value never has to be displayed (or CSS-capitalized) directly.
export const DISPLAY_MODE_LABELS: Record<DisplayMode, string> = {
  river: "River",
  feed_only: "Feed only",
  radar: "Radar",
};

export const DISPLAY_MODE_DESCRIPTIONS: Record<DisplayMode, string> = {
  river: "Shows on the home feed.",
  feed_only: "Only visible from the feed itself.",
  radar: "Only visible from the feed and the Radar section. No unread count.",
};
