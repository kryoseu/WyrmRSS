import { Link, Outlet, useLocation } from "react-router";
import { TbSettings } from "react-icons/tb";
import { MobileTabBar } from "../components/MobileTabBar";
import { useFeedIcon } from "../hooks/useFeeds";
import { useScreenTitle } from "../hooks/useScreenTitle";
import { initials } from "../utils/posts";

export function MobileLayout() {
  const { pathname } = useLocation();
  const { title, subtitle, feed } = useScreenTitle();
  const iconUrl = useFeedIcon(feed);
  const onSettings = pathname.startsWith("/settings");

  return (
    <div className="layout layout--mobile">
      <header className="mobile-topbar">
        {feed &&
          (iconUrl ? (
            <img className="mobile-topbar-icon" src={iconUrl} alt="" />
          ) : (
            <span className="mobile-topbar-icon mobile-topbar-initials">{initials(feed.name)}</span>
          ))}
        <div className="mobile-topbar-heading">
          <h1 className="mobile-topbar-title">{title}</h1>
          {subtitle && <span className="mobile-topbar-subtitle">{subtitle}</span>}
        </div>
        <Link
          to="/settings"
          className={`mobile-topbar-settings${onSettings ? " active" : ""}`}
          aria-label="Settings"
          aria-current={onSettings ? "page" : undefined}
        >
          <TbSettings />
        </Link>
      </header>
      <div className="layout-mobile-content">
        <Outlet />
      </div>
      <MobileTabBar />
    </div>
  );
}
