import { createContext } from "react";
import type { AccentKey, Density, FontSize, SnippetLines, Theme } from "../hooks/useTheme";

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
  snippetLines: SnippetLines;
  setSnippetLines: (lines: SnippetLines) => void;
  hideThumbnails: boolean;
  setHideThumbnails: (hide: boolean) => void;
  highContrast: boolean;
  toggleHighContrast: () => void;
  setHighContrast: (on: boolean) => void;
  navCollapsed: boolean;
  toggleNavCollapsed: () => void;
  /** Browser tab title; empty means the default. */
  siteTitle: string;
  setSiteTitle: (title: string) => void;
  /** Sidebar logo URL (http(s) or data URL); null means the default logo. */
  customLogo: string | null;
  /** Returns false if the logo could not be saved (storage quota). */
  setCustomLogo: (logo: string | null) => boolean;
}

export const ThemeContext = createContext<ThemeContextValue | null>(null);
