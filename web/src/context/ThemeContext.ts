import { createContext } from "react";
import type { AccentKey, Density, FontSize, Theme } from "../hooks/useTheme";

export interface ThemeContextValue {
  theme: Theme;
  setTheme: (t: Theme) => void;
  toggleTheme: () => void;
  accent: AccentKey;
  setAccent: (key: AccentKey) => void;
  fontSize: FontSize;
  setFontSize: (size: FontSize) => void;
  density: Density;
  setDensity: (d: Density) => void;
}

export const ThemeContext = createContext<ThemeContextValue | null>(null);
