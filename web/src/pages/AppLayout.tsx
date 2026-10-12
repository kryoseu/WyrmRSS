import { Outlet } from "react-router";
import { ServerStatusBanner } from "../components/ServerStatusBanner";
import { SideMenu } from "../components/SideMenu";
import { useIsMobile } from "../hooks/useIsMobile";
import { MobileLayout } from "./MobileLayout";

export function AppLayout() {
  const isMobile = useIsMobile();

  if (isMobile) return <MobileLayout />;

  return (
    <div className="layout">
      <SideMenu />
      <Outlet />
      <ServerStatusBanner />
    </div>
  );
}
