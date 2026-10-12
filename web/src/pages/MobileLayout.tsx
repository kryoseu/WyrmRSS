import { useEffect, useState } from "react";
import { Outlet, useLocation } from "react-router";
import { TbMenu2 } from "react-icons/tb";
import { MobileTabBar } from "../components/MobileTabBar";
import { SideMenu } from "../components/SideMenu";
import { useFeedIcon } from "../hooks/useFeeds";
import { useScreenTitle } from "../hooks/useScreenTitle";
import { initials } from "../utils/posts";

export function MobileLayout() {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const { pathname } = useLocation();
  const { title, subtitle, feed } = useScreenTitle();
  const iconUrl = useFeedIcon(feed);

  // Close the drawer on any navigation, including the back gesture.
  const [lastPathname, setLastPathname] = useState(pathname);
  if (lastPathname !== pathname) {
    setLastPathname(pathname);
    setDrawerOpen(false);
  }

  useEffect(() => {
    if (!drawerOpen) return;
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") setDrawerOpen(false);
    }
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [drawerOpen]);

  return (
    <div className="layout layout--mobile">
      <header className="mobile-topbar">
        <button className="mobile-topbar-menu" onClick={() => setDrawerOpen(true)} aria-label="Open menu">
          <TbMenu2 />
        </button>
        <div className="mobile-topbar-heading">
          <h1 className="mobile-topbar-title">{title}</h1>
          {subtitle && <span className="mobile-topbar-subtitle">{subtitle}</span>}
        </div>
        {feed &&
          (iconUrl ? (
            <img className="mobile-topbar-icon" src={iconUrl} alt="" />
          ) : (
            <span className="mobile-topbar-icon mobile-topbar-initials">{initials(feed.name)}</span>
          ))}
      </header>
      <div className="layout-mobile-content">
        <Outlet />
      </div>
      <MobileTabBar />
      <div
        className={`drawer-backdrop${drawerOpen ? " open" : ""}`}
        onClick={() => setDrawerOpen(false)}
      />
      <div
        className={`drawer${drawerOpen ? " open" : ""}`}
        inert={!drawerOpen}
        // Also covers tapping the feed that's already open, which leaves the
        // pathname unchanged.
        onClick={(e) => {
          if ((e.target as Element).closest("a")) setDrawerOpen(false);
        }}
      >
        <SideMenu />
      </div>
    </div>
  );
}
