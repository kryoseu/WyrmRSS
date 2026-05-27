import { useEffect, useState } from "react";

export const PALETTES = {
  orange: { light: "#e8500a", dark: "#fb8040" },
  purple: { light: "#aa3bff", dark: "#c084fc" },
  blue:   { light: "#2563eb", dark: "#60a5fa" },
  green:  { light: "#16a34a", dark: "#4ade80" },
  rose:   { light: "#e11d48", dark: "#fb7185" },
} as const;

export type AccentKey = keyof typeof PALETTES;
type Theme = "light" | "dark";

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
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function getInitialAccent(): AccentKey {
  const saved = localStorage.getItem("accent");
  if (saved && saved in PALETTES) return saved as AccentKey;
  return "orange";
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(getInitialTheme);
  const [accent, setAccentState] = useState<AccentKey>(getInitialAccent);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("theme", theme);
    applyAccent(PALETTES[accent][theme], theme === "dark");
  }, [theme, accent]);

  const toggleTheme = () => setThemeState((t) => (t === "dark" ? "light" : "dark"));

  const setAccent = (key: AccentKey) => {
    setAccentState(key);
    localStorage.setItem("accent", key);
  };

  return { theme, toggleTheme, accent, setAccent };
}
