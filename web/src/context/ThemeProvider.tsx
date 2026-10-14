import { useEffect, useState, type ReactNode } from "react";
import { ThemeContext, type ThemeContextValue } from "./ThemeContext";
import { PALETTES } from "../hooks/useTheme";
import type { AccentKey, Density, FontSize, Theme } from "../hooks/useTheme";
import { useIsMobile } from "../hooks/useIsMobile";
import { DEFAULTS, LAYOUT } from "../utils/constants";

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
}

function getInitialTheme(): Theme {
  const saved = localStorage.getItem("theme");
  if (saved === "light" || saved === "dark") return saved;
  return DEFAULTS.theme;
}

function getInitialAccent(): AccentKey {
  const saved = localStorage.getItem("accent");
  if (saved && saved in PALETTES) return saved as AccentKey;
  return DEFAULTS.accent;
}

function getInitialFontSize(): FontSize {
  const saved = localStorage.getItem("font-size");
  if (saved === "small" || saved === "default" || saved === "large") return saved;
  return DEFAULTS.fontSize;
}

function getSavedDensity(): Density | null {
  const saved = localStorage.getItem("post-density");
  if (saved === "title" || saved === "card") return saved;
  return null;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(getInitialTheme);
  const [accent, setAccentState] = useState<AccentKey>(getInitialAccent);
  const [fontSize, setFontSizeState] = useState<FontSize>(getInitialFontSize);
  // Unset until the user picks one, so each device keeps its own default.
  const [savedDensity, setSavedDensity] = useState<Density | null>(getSavedDensity);
  const isMobile = useIsMobile();

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("theme", theme);
    applyAccent(PALETTES[accent][theme], theme === "dark");
  }, [theme, accent]);

  useEffect(() => {
    document.documentElement.style.setProperty("--font-base", LAYOUT.fontSizes[fontSize]);
    localStorage.setItem("font-size", fontSize);
  }, [fontSize]);

  const value: ThemeContextValue = {
    theme,
    setTheme: (t) => setThemeState(t),
    toggleTheme: () => setThemeState((t) => (t === "dark" ? "light" : "dark")),
    accent,
    setAccent: (key) => {
      setAccentState(key);
      localStorage.setItem("accent", key);
    },
    fontSize,
    setFontSize: (size) => setFontSizeState(size),
    density: savedDensity ?? DEFAULTS.density[isMobile ? "mobile" : "desktop"],
    setDensity: (d) => {
      setSavedDensity(d);
      localStorage.setItem("post-density", d);
    },
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
