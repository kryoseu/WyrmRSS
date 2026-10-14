import { Link, useLocation } from "react-router";
import { TbArchive, TbBookmark, TbHome, TbRadar, TbRss } from "react-icons/tb";

const TABS = [
  { to: "/", label: "Home", icon: <TbHome />, match: (p: string) => p === "/" || p.startsWith("/posts/") },
  { to: "/feeds", label: "Feeds", icon: <TbRss /> },
  { to: "/radar", label: "Radar", icon: <TbRadar /> },
  { to: "/read-later", label: "Read Later", icon: <TbBookmark /> },
  { to: "/archive", label: "Archive", icon: <TbArchive /> },
];

export function MobileTabBar() {
  const { pathname } = useLocation();

  return (
    <nav className="mobile-tabbar" aria-label="Sections">
      {TABS.map(({ to, label, icon, match }) => {
        const active = match ? match(pathname) : pathname.startsWith(to);
        return (
          <Link
            key={to}
            to={to}
            className={`mobile-tab${active ? " active" : ""}`}
            aria-current={active ? "page" : undefined}
          >
            {icon}
            <span>{label}</span>
          </Link>
        );
      })}
    </nav>
  );
}
