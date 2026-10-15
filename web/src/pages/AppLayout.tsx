import { Outlet } from "react-router";
import { DemoBanner } from "../components/layout/DemoBanner";
import { DemoInstallHint } from "../components/layout/DemoInstallHint";
import { ServerStatusBanner } from "../components/layout/ServerStatusBanner";
import { SideMenu } from "../components/layout/SideMenu";
import { useIsMobile } from "../hooks/useIsMobile";
import { MobileLayout } from "./MobileLayout";

export function AppLayout() {
  const isMobile = useIsMobile();

  if (isMobile) return <MobileLayout />;

  return (
    <div className="layout">
      <SideMenu />
      <Outlet />
      <DemoBanner />
      <DemoInstallHint mobile={false} />
      <ServerStatusBanner />
    </div>
  );
}
