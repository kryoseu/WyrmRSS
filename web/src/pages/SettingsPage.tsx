import { useState } from "react";
import { Appearance } from "../components/settings/Appearance";
import { Credentials } from "../components/settings/Credentials";
import { Feeds } from "../components/settings/Feeds";
import { Folders } from "../components/settings/Folders";
import { General } from "../components/settings/General";
import { Opml } from "../components/settings/Opml";
import { Shortcuts } from "../components/settings/Shortcuts";
import { Webhooks } from "../components/settings/Webhooks";
import { useIsMobile } from "../hooks/useIsMobile";
import { version } from "../../package.json";

type Tab =
  | "general"
  | "appearance"
  | "shortcuts"
  | "feeds"
  | "credentials"
  | "folders"
  | "webhooks"
  | "opml";

export function SettingsPage() {
  const [tab, setTab] = useState<Tab>("general");
  const isMobile = useIsMobile();

  return (
    <div className="settings-page">
      <div className="settings-tabs">
        <button
          className={`settings-tab${tab === "general" ? " active" : ""}`}
          onClick={() => setTab("general")}
        >
          General
        </button>
        <button
          className={`settings-tab${tab === "appearance" ? " active" : ""}`}
          onClick={() => setTab("appearance")}
        >
          Appearance
        </button>
        {!isMobile && (
          <button
            className={`settings-tab${tab === "shortcuts" ? " active" : ""}`}
            onClick={() => setTab("shortcuts")}
          >
            Shortcuts
          </button>
        )}
        <button
          className={`settings-tab${tab === "feeds" ? " active" : ""}`}
          onClick={() => setTab("feeds")}
        >
          Feeds
        </button>
        <button
          className={`settings-tab${tab === "credentials" ? " active" : ""}`}
          onClick={() => setTab("credentials")}
        >
          Credentials
        </button>
        <button
          className={`settings-tab${tab === "folders" ? " active" : ""}`}
          onClick={() => setTab("folders")}
        >
          Folders
        </button>
        <button
          className={`settings-tab${tab === "webhooks" ? " active" : ""}`}
          onClick={() => setTab("webhooks")}
        >
          Webhooks
        </button>
        <button
          className={`settings-tab${tab === "opml" ? " active" : ""}`}
          onClick={() => setTab("opml")}
        >
          Import / Export
        </button>
      </div>

      <div className="settings-content">
        {tab === "general" && <General />}
        {tab === "appearance" && <Appearance />}
        {tab === "shortcuts" && !isMobile && <Shortcuts />}
        {tab === "feeds" && <Feeds />}
        {tab === "credentials" && <Credentials />}
        {tab === "folders" && <Folders />}
        {tab === "webhooks" && <Webhooks />}
        {tab === "opml" && <Opml />}
        <div className="settings-app-version">v{version}</div>
      </div>
    </div>
  );
}
