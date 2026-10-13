import { useEffect, useState } from "react";
import { TbDeviceMobile, TbX } from "react-icons/tb";
import { useDemoMode } from "../hooks/useSettings";

const DISMISSED_KEY = "wyrm:demo-install-hint-dismissed";

// Chrome-only event, not in TypeScript's DOM lib.
interface BeforeInstallPromptEvent extends Event {
  prompt(): Promise<void>;
}

// Chrome fires this once, early, possibly before this component mounts (the
// settings query decides whether it renders at all), so capture it at import.
let installPrompt: BeforeInstallPromptEvent | null = null;
window.addEventListener("beforeinstallprompt", (e) => {
  e.preventDefault();
  installPrompt = e as BeforeInstallPromptEvent;
});

function isDismissed(): boolean {
  try {
    return localStorage.getItem(DISMISSED_KEY) === "1";
  } catch {
    return false;
  }
}

function isInstalled(): boolean {
  return (
    window.matchMedia("(display-mode: standalone)").matches ||
    // iOS Safari's own flag for a home-screen launch.
    (navigator as Navigator & { standalone?: boolean }).standalone === true
  );
}

const isIos = /iPad|iPhone|iPod/.test(navigator.userAgent);

/**
 * Demo-only nudge to try the PWA. Desktop gets a QR code to open the demo on a
 * phone; a phone in a browser tab gets Add to Home Screen instructions, or
 * Chrome's own install dialog when the browser offers one. Dismissal sticks.
 */
export function DemoInstallHint({ mobile }: { mobile: boolean }) {
  const isDemo = useDemoMode();
  const [dismissed, setDismissed] = useState(isDismissed);
  const [canPrompt, setCanPrompt] = useState(installPrompt !== null);

  useEffect(() => {
    const handle = () => setCanPrompt(true);
    const installed = () => setCanPrompt(false);
    window.addEventListener("beforeinstallprompt", handle);
    window.addEventListener("appinstalled", installed);
    return () => {
      window.removeEventListener("beforeinstallprompt", handle);
      window.removeEventListener("appinstalled", installed);
    };
  }, []);

  if (!isDemo || dismissed || isInstalled()) return null;

  function dismiss() {
    setDismissed(true);
    try {
      localStorage.setItem(DISMISSED_KEY, "1");
    } catch {
      // Private mode: the hint just comes back next visit.
    }
  }

  async function install() {
    if (!installPrompt) return;
    await installPrompt.prompt();
    installPrompt = null;
    setCanPrompt(false);
  }

  const closeBtn = (
    <button className="demo-install-close" onClick={dismiss} aria-label="Dismiss">
      <TbX />
    </button>
  );

  if (!mobile) {
    return (
      <div className="demo-install demo-install--desktop">
        {closeBtn}
        <img className="demo-install-qr" src="/demo-qr.svg" alt="QR code for the demo" />
        <div className="demo-install-text">
          <strong>Try it on your phone</strong>
          <span>Scan to open the demo, then Add to Home Screen for the app version.</span>
        </div>
      </div>
    );
  }

  return (
    <div className="demo-install demo-install--mobile">
      <TbDeviceMobile />
      <span className="demo-install-text">
        {canPrompt
          ? "Install WyrmRSS for the full app experience."
          : isIos
            ? "Tap Share → Add to Home Screen for the app version."
            : "Use your browser menu → Add to Home Screen for the app version."}
      </span>
      {canPrompt && (
        <button className="demo-install-btn" onClick={install}>
          Install
        </button>
      )}
      {closeBtn}
    </div>
  );
}
