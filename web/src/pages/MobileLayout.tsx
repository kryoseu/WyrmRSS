import { useEffect, useState } from "react";
import { Outlet, useLocation } from "react-router";
import { TbMenu2 } from "react-icons/tb";
import { SideMenu } from "../components/SideMenu";

export function MobileLayout() {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const { pathname } = useLocation();

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
      </header>
      <div className="layout-mobile-content">
        <Outlet />
      </div>
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
