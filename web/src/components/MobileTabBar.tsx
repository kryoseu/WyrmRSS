import { NavLink } from "react-router";
import { TbArchive, TbBookmark, TbHome, TbRadar } from "react-icons/tb";

const TABS = [
  { to: "/feeds", label: "Home", icon: <TbHome /> },
  { to: "/radar", label: "Radar", icon: <TbRadar /> },
  { to: "/read-later", label: "Read Later", icon: <TbBookmark /> },
  { to: "/archive", label: "Archive", icon: <TbArchive /> },
];

export function MobileTabBar() {
  return (
    <nav className="mobile-tabbar" aria-label="Sections">
      {TABS.map(({ to, label, icon }) => (
        <NavLink
          key={to}
          to={to}
          className={({ isActive }) => `mobile-tab${isActive ? " active" : ""}`}
        >
          {icon}
          <span>{label}</span>
        </NavLink>
      ))}
    </nav>
  );
}
