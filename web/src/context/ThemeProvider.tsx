import { useEffect, useState, type ReactNode } from "react";
import { ThemeContext, type ThemeContextValue } from "./ThemeContext";
import { PALETTES } from "../hooks/useTheme";
import type { AccentKey, Density, FontSize, SnippetLines, Theme } from "../hooks/useTheme";
import { useIsMobile } from "../hooks/useIsMobile";
import { DEFAULTS, LAYOUT, STORAGE_KEYS } from "../utils/constants";

function hexToRgb(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function applyAccent(hex: string, isDark: boolean) {
  const [r, g, b] = hexToRgb(hex);
  const alpha = isDark ? 0.15 : 0.1;
  const root = document.documentElement;
  root.style.setProperty("--accent", hex);
  root.style.setProperty("--accent-bg", `rgba(${r}, ${g}, ${b}, ${alpha})`);
  root.style.setProperty("--accent-border", `rgba(${r}, ${g}, ${b}, 0.5)`);
  // Dark text on very light accents (e.g. slate in dark mode), white otherwise.
  // https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color
  const brightness = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  root.style.setProperty("--accent-contrast", brightness > 0.75 ? "#16171d" : "#fff");
}

function getInitialTheme(): Theme {
  const saved = localStorage.getItem(STORAGE_KEYS.theme);
  if (saved === "light" || saved === "dark") return saved;
  return DEFAULTS.theme;
}

function getInitialAccent(): AccentKey {
  const saved = localStorage.getItem(STORAGE_KEYS.accent);
  if (saved && saved in PALETTES) return saved as AccentKey;
  return DEFAULTS.accent;
}

function getInitialFontSize(): FontSize {
  const saved = localStorage.getItem(STORAGE_KEYS.fontSize);
  if (saved === "small" || saved === "default" || saved === "large") return saved;
  return DEFAULTS.fontSize;
}

function getSavedDensity(): Density | null {
  const saved = localStorage.getItem(STORAGE_KEYS.density);
  if (saved === "title" || saved === "card") return saved;
  return null;
}

function getInitialSnippetLines(): SnippetLines {
  const saved = localStorage.getItem(STORAGE_KEYS.snippetLines);
  if (saved === "0" || saved === "1" || saved === "2" || saved === "3") return Number(saved) as SnippetLines;
  return DEFAULTS.snippetLines;
}

function getInitialHideThumbnails(): boolean {
  const saved = localStorage.getItem(STORAGE_KEYS.hideThumbnails);
  if (saved === null) return DEFAULTS.hideThumbnails;
  return saved === "true";
}

function getInitialFlag(key: string, fallback: boolean): boolean {
  const saved = localStorage.getItem(key);
  if (saved === null) return fallback;
  return saved === "true";
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(getInitialTheme);
  const [accent, setAccentState] = useState<AccentKey>(getInitialAccent);
  const [fontSize, setFontSizeState] = useState<FontSize>(getInitialFontSize);
  // Unset until the user picks one, so each device keeps its own default.
  const [savedDensity, setSavedDensity] = useState<Density | null>(getSavedDensity);
  const [snippetLines, setSnippetLinesState] = useState<SnippetLines>(getInitialSnippetLines);
  const [hideThumbnails, setHideThumbnailsState] = useState(getInitialHideThumbnails);
  const [highContrast, setHighContrastState] = useState(() =>
    getInitialFlag(STORAGE_KEYS.highContrast, DEFAULTS.highContrast),
  );
  const [navCollapsed, setNavCollapsed] = useState(() =>
    getInitialFlag(STORAGE_KEYS.navCollapsed, DEFAULTS.navCollapsed),
  );
  const [siteTitle, setSiteTitleState] = useState(() => localStorage.getItem(STORAGE_KEYS.siteTitle) ?? "");
  const [customLogo, setCustomLogoState] = useState(() => localStorage.getItem(STORAGE_KEYS.customLogo));
  const isMobile = useIsMobile();

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem(STORAGE_KEYS.theme, theme);
    applyAccent(PALETTES[accent][theme], theme === "dark");
  }, [theme, accent]);

  useEffect(() => {
    document.documentElement.style.setProperty("--font-base", LAYOUT.fontSizes[fontSize]);
    localStorage.setItem(STORAGE_KEYS.fontSize, fontSize);
  }, [fontSize]);

  useEffect(() => {
    document.documentElement.style.setProperty("--snippet-lines", String(snippetLines));
  }, [snippetLines]);

  useEffect(() => {
    if (highContrast) document.documentElement.dataset.contrast = "high";
    else delete document.documentElement.dataset.contrast;
    localStorage.setItem(STORAGE_KEYS.highContrast, String(highContrast));
  }, [highContrast]);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEYS.navCollapsed, String(navCollapsed));
  }, [navCollapsed]);

  useEffect(() => {
    document.title = siteTitle.trim() || DEFAULTS.siteTitle;
  }, [siteTitle]);

  const value: ThemeContextValue = {
    theme,
    setTheme: (t) => setThemeState(t),
    toggleTheme: () => setThemeState((t) => (t === "dark" ? "light" : "dark")),
    accent,
    setAccent: (key) => {
      setAccentState(key);
      localStorage.setItem(STORAGE_KEYS.accent, key);
    },
    fontSize,
    setFontSize: (size) => setFontSizeState(size),
    density: savedDensity ?? DEFAULTS.density[isMobile ? "mobile" : "desktop"],
    setDensity: (d) => {
      setSavedDensity(d);
      localStorage.setItem(STORAGE_KEYS.density, d);
    },
    snippetLines,
    setSnippetLines: (lines) => {
      setSnippetLinesState(lines);
      localStorage.setItem(STORAGE_KEYS.snippetLines, String(lines));
    },
    hideThumbnails,
    setHideThumbnails: (hide) => {
      setHideThumbnailsState(hide);
      localStorage.setItem(STORAGE_KEYS.hideThumbnails, String(hide));
    },
    highContrast,
    toggleHighContrast: () => setHighContrastState((on) => !on),
    setHighContrast: (on) => setHighContrastState(on),
    navCollapsed,
    toggleNavCollapsed: () => setNavCollapsed((c) => !c),
    siteTitle,
    setSiteTitle: (title) => {
      setSiteTitleState(title);
      if (title.trim()) localStorage.setItem(STORAGE_KEYS.siteTitle, title);
      else localStorage.removeItem(STORAGE_KEYS.siteTitle);
    },
    customLogo,
    setCustomLogo: (logo) => {
      try {
        if (logo) localStorage.setItem(STORAGE_KEYS.customLogo, logo);
        else localStorage.removeItem(STORAGE_KEYS.customLogo);
      } catch {
        return false;
      }
      setCustomLogoState(logo);
      return true;
    },
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
