import { NavLink } from "react-router-dom";
import { useTheme, PALETTES } from "../hooks/useTheme";
import type { AccentKey } from "../hooks/useTheme";
import { FeedList } from "./FeedList";

interface Props {
  excludedFeeds: Set<number>;
  onToggleExclude: (feedId: number) => void;
}

export function SideMenu({ excludedFeeds, onToggleExclude }: Props) {
  const { theme, toggleTheme, accent, setAccent } = useTheme();

  return (
    <div className="sidebar">
      <div className="sidebar-brand">
        <div className="sidebar-logo">W</div>
        <span className="sidebar-name">Wyrm-RSS</span>
      </div>

      <nav className="sidebar-nav">
        <NavLink
          to="/feeds"
          end
          className={({ isActive }) => `sidebar-nav-item${isActive ? " active" : ""}`}
        >
          Home
        </NavLink>
        <NavLink
          to="/settings"
          className={({ isActive }) => `sidebar-nav-item${isActive ? " active" : ""}`}
        >
          Settings
        </NavLink>
        <NavLink
          to="/favorites"
          end
          className={({ isActive }) => `sidebar-nav-item${isActive ? " active" : ""}`}
        >
          Favorite Posts
        </NavLink>
      </nav>

      <FeedList excludedFeeds={excludedFeeds} onToggleExclude={onToggleExclude} />

      <div className="sidebar-footer">
        <div className="accent-picker">
          {(Object.keys(PALETTES) as AccentKey[]).map((key) => (
            <button
              key={key}
              className={`accent-dot${accent === key ? " active" : ""}`}
              style={{ "--dot-color": PALETTES[key].light } as React.CSSProperties}
              onClick={() => setAccent(key)}
              title={key}
            />
          ))}
        </div>
        <button
          className="theme-toggle"
          onClick={toggleTheme}
          title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
        >
          {theme === "dark" ? "☀" : "☾"}
        </button>
      </div>
    </div>
  );
}
