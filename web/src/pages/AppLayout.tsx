import { Outlet } from "react-router";
import { SideMenu } from "../components/SideMenu";

export function AppLayout() {
  return (
    <div className="layout">
      <SideMenu />
      <Outlet />
    </div>
  );
}
