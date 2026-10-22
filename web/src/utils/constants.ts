import type { AccentKey, Density, FontSize, Theme } from "../hooks/useTheme";

// Values that change app behavior, kept in one place for easy tuning.

// Per-device UI defaults, used until the user picks a value (saved in localStorage).
// Server-side settings (page size, read mode, ...) default in the DB migrations instead.
export const DEFAULTS = {
  theme: "dark" as Theme,
  accent: "orange" as AccentKey,
  fontSize: "default" as FontSize,
  density: { mobile: "card", desktop: "card" } as Record<"mobile" | "desktop", Density>,
  // Demo visitors share read state, so show everything there.
  unreadOnly: { normal: true, demo: false },
  hideCaughtUp: false,
};

export const LAYOUT = {
  // Phone layout at or below this width; not used in CSS.
  mobileBreakpointPx: 768,
  // Reader pane opens at this fraction of the window width.
  readerWidthFraction: 0.4,
  readerMinWidthPx: 280,
  fontSizes: { small: "15px", default: "18px", large: "21px" } as Record<FontSize, string>,
  // Pull distance that triggers a refresh; must stay below MAX_PULL (128) in usePullToRefresh.ts.
  pullToRefreshTriggerPx: 100,
};

export const CONTENT = {
  // Max characters of plain-text preview.
  cardSnippetLength: 160,
  radarSnippetLength: 140,
  // Radar feeds with at least this many unread get a full card, up to the cap.
  radarFeaturedMinUnread: 10,
  radarFeaturedMaxCards: 8,
};

export const TIMING = {
  // Wait after the last keystroke before searching posts.
  searchDebounceMs: 300,
  // How long the demo banner shows "restricted" after a blocked write.
  demoRestrictedMs: 3000,
  desktopBackendTimeoutMs: 30_000,
};
