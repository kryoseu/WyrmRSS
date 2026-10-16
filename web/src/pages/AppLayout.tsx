import { Outlet } from "react-router";
import { DemoBanner } from "../components/layout/DemoBanner";
import { DemoInstallHint } from "../components/layout/DemoInstallHint";
import { ServerStatusBanner } from "../components/layout/ServerStatusBanner";
import { ShortcutsHelp } from "../components/layout/ShortcutsHelp";
import { SideMenu } from "../components/layout/SideMenu";
import { useIsMobile } from "../hooks/useIsMobile";
import { MobileLayout } from "./MobileLayout";

export function AppLayout() {
  const isMobile = useIsMobile();

  return (
    <>
      {isMobile ? (
        <MobileLayout />
      ) : (
        <div className="layout">
          <SideMenu />
          <Outlet />
          <DemoBanner />
          <DemoInstallHint mobile={false} />
          <ServerStatusBanner />
        </div>
      )}
      {/* Outside both layouts, so tablets with a keyboard get it too. */}
      <ShortcutsHelp />
    </>
  );
}
