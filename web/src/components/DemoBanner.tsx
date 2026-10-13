import { useEffect, useState } from "react";
import { TbInfoCircle } from "react-icons/tb";
import { useDemoMode } from "../hooks/useSettings";

const RESTRICTED_MS = 3000;

/**
 * Shown on the public demo instance, where changes are reset nightly. Blocked
 * writes raise "wyrm:demo-restricted" (utils/error.ts), which briefly swaps the
 * text so actions without their own error UI don't fail silently.
 */
export function DemoBanner() {
  const isDemo = useDemoMode();
  const [restricted, setRestricted] = useState(false);

  useEffect(() => {
    let timer: number | undefined;
    const handle = () => {
      setRestricted(true);
      clearTimeout(timer);
      timer = window.setTimeout(() => setRestricted(false), RESTRICTED_MS);
    };
    window.addEventListener("wyrm:demo-restricted", handle);
    return () => {
      window.removeEventListener("wyrm:demo-restricted", handle);
      clearTimeout(timer);
    };
  }, []);

  if (!isDemo) return null;

  return (
    <div className={`demo-banner${restricted ? " restricted" : ""}`} role="status">
      <TbInfoCircle />
      <span>{restricted ? "That's disabled in the demo" : "Demo: changes reset nightly"}</span>
    </div>
  );
}
