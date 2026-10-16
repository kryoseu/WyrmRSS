import { Outlet } from "react-router";
import { DemoBanner } from "../components/layout/DemoBanner";
import { DemoInstallHint } from "../components/layout/DemoInstallHint";
import { ServerStatusBanner } from "../components/layout/ServerStatusBanner";
import { ShortcutsHelp } from "../components/layout/ShortcutsHelp";
import { SideMenu } from "../components/layout/SideMenu";
import { useIsMobile } from "../hooks/useIsMobile";
import { useNavigationShortcuts } from "../hooks/useShortcuts";
import { MobileLayout } from "./MobileLayout";

export function AppLayout() {
  const isMobile = useIsMobile();
  useNavigationShortcuts();

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
      {/* Outside both layouts: shortcuts work at any window width. */}
      <ShortcutsHelp />
    </>
  );
}
