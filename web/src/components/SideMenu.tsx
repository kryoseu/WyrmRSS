import { NavLink } from "react-router-dom";
import { useTheme, PALETTES } from "../hooks/useTheme";
import type { AccentKey } from "../hooks/useTheme";
import { FeedList } from "./FeedList";

export function SideMenu() {
  const { theme, toggleTheme, accent, setAccent } = useTheme();

  return (
    <div className="sidebar">
      <div className="sidebar-brand">
        <img
          className="sidebar-logo"
          src={theme === "light" ? "/l-o-dark.svg" : "/l-o-light.svg"}
          alt="Wyrm-RSS"
        />
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
          to="/read-later"
          end
          className={({ isActive }) => `sidebar-nav-item${isActive ? " active" : ""}`}
        >
          Read Later
        </NavLink>
        <NavLink
          to="/archive"
          end
          className={({ isActive }) => `sidebar-nav-item${isActive ? " active" : ""}`}
        >
          Archive
        </NavLink>
      </nav>

      <FeedList />

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
