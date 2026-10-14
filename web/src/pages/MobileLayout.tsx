import { Link, Outlet, useLocation, useNavigate, useParams } from "react-router";
import { TbChevronLeft, TbSettings } from "react-icons/tb";
import { MobileTabBar } from "../components/MobileTabBar";
import { DemoBanner } from "../components/DemoBanner";
import { DemoInstallHint } from "../components/DemoInstallHint";
import { ServerStatusBanner } from "../components/ServerStatusBanner";
import { useFeedIcon } from "../hooks/useFeeds";
import { useScreenTitle } from "../hooks/useScreenTitle";
import { initials } from "../utils/posts";

export function MobileLayout() {
  const { pathname } = useLocation();
  // The reader covers the screen on post routes (ReaderPage).
  const { postId } = useParams();
  const navigate = useNavigate();
  const { title, subtitle, feed } = useScreenTitle();
  const iconUrl = useFeedIcon(feed);
  const onSettings = pathname.startsWith("/settings");

  // Back to wherever the feed was opened from (Feeds, Radar…). A deep link
  // has no such entry, so swap it for the Feeds list rather than leaving the app.
  function goBack() {
    if ((window.history.state?.idx ?? 0) > 0) navigate(-1);
    else navigate("/feeds", { replace: true });
  }

  return (
    <div className="layout layout--mobile">
      <header className="mobile-topbar">
        {feed && (
          <button className="mobile-topbar-btn" onClick={goBack} aria-label="Back">
            <TbChevronLeft />
          </button>
        )}
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
          className={`mobile-topbar-btn${onSettings ? " active" : ""}`}
          aria-label="Settings"
          aria-current={onSettings ? "page" : undefined}
        >
          <TbSettings />
        </Link>
      </header>
      {postId === undefined && <DemoBanner />}
      <DemoInstallHint mobile />
      <ServerStatusBanner />
      <div className="layout-mobile-content">
        <Outlet />
      </div>
      <MobileTabBar />
    </div>
  );
}
