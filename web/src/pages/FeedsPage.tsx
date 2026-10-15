import { Navigate } from "react-router";
import { FeedList } from "../components/feeds/FeedList";
import { useIsMobile } from "../hooks/useIsMobile";

/** Phone-only Feeds tab; on desktop the sidebar already shows this list. */
export function FeedsPage() {
  const isMobile = useIsMobile();
  if (!isMobile) return <Navigate to="/" replace />;

  return (
    <div className="sidebar feeds-page">
      <FeedList />
    </div>
  );
}
