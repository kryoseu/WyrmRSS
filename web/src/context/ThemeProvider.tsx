import { useEffect, useState, type ReactNode } from "react";
import { ThemeContext, type ThemeContextValue } from "./ThemeContext";
import { PALETTES } from "../hooks/useTheme";
import type { AccentKey, FontSize, Theme } from "../hooks/useTheme";

const FONT_SIZE_MAP: Record<FontSize, string> = {
  small: "15px",
  default: "18px",
  large: "21px",
};

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
  return "dark";
}

function getInitialAccent(): AccentKey {
  const saved = localStorage.getItem("accent");
  if (saved && saved in PALETTES) return saved as AccentKey;
  return "orange";
}

function getInitialFontSize(): FontSize {
  const saved = localStorage.getItem("font-size");
  if (saved === "small" || saved === "default" || saved === "large") return saved;
  return "default";
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(getInitialTheme);
  const [accent, setAccentState] = useState<AccentKey>(getInitialAccent);
  const [fontSize, setFontSizeState] = useState<FontSize>(getInitialFontSize);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("theme", theme);
    applyAccent(PALETTES[accent][theme], theme === "dark");
  }, [theme, accent]);

  useEffect(() => {
    document.documentElement.style.setProperty("--font-base", FONT_SIZE_MAP[fontSize]);
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
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
