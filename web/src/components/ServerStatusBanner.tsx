import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { TbWifiOff } from "react-icons/tb";

/** Shown while API requests can't reach the server; hides on the next answer. */
export function ServerStatusBanner() {
  const qc = useQueryClient();
  const [unreachable, setUnreachable] = useState(false);

  useEffect(() => {
    const down = () => setUnreachable(true);
    const up = () => setUnreachable(false);
    window.addEventListener("wyrm:unreachable", down);
    window.addEventListener("wyrm:reachable", up);
    return () => {
      window.removeEventListener("wyrm:unreachable", down);
      window.removeEventListener("wyrm:reachable", up);
    };
  }, []);

  if (!unreachable) return null;

  return (
    <div className="server-banner" role="alert">
      <TbWifiOff />
      <span>{navigator.onLine ? "Can't reach the Wyrm server" : "You're offline"}</span>
      <button className="server-banner-retry" onClick={() => qc.refetchQueries({ type: "active" })}>
        Retry
      </button>
    </div>
  );
}
