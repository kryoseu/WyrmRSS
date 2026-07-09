import { useState } from "react";
import { Appearance } from "../components/settings/Appearance";
import { Feeds } from "../components/settings/Feeds";
import { Folders } from "../components/settings/Folders";
import { General } from "../components/settings/General";
import { Opml } from "../components/settings/Opml";
import { Webhooks } from "../components/settings/Webhooks";

type Tab = "general" | "appearance" | "feeds" | "folders" | "webhooks" | "opml";

export function SettingsPage() {
  const [tab, setTab] = useState<Tab>("general");

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
        <button
          className={`settings-tab${tab === "feeds" ? " active" : ""}`}
          onClick={() => setTab("feeds")}
        >
          Feeds
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
        {tab === "feeds" && <Feeds />}
        {tab === "folders" && <Folders />}
        {tab === "webhooks" && <Webhooks />}
        {tab === "opml" && <Opml />}
      </div>
    </div>
  );
}
