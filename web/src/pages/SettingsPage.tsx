import { useState } from "react";
import { Appearance } from "../components/settings/Appearance";
import { Feeds } from "../components/settings/Feeds";
import { General } from "../components/settings/General";
import { Opml } from "../components/settings/Opml";

type Tab = "general" | "appearance" | "feeds" | "opml";

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
        {tab === "opml" && <Opml />}
      </div>
    </div>
  );
}
