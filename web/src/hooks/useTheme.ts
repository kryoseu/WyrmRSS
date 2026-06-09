import { useContext } from "react";
import { ThemeContext } from "../context/ThemeContext";

export const PALETTES = {
  orange: { light: "#e8500a", dark: "#fb8040" },
  purple: { light: "#aa3bff", dark: "#c084fc" },
  blue:   { light: "#2563eb", dark: "#60a5fa" },
  green:  { light: "#16a34a", dark: "#4ade80" },
  rose:   { light: "#e11d48", dark: "#fb7185" },
} as const;

export type AccentKey = keyof typeof PALETTES;
export type Theme = "light" | "dark";
export type FontSize = "small" | "default" | "large";

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme must be used within ThemeProvider");
  return ctx;
}
