import { useState } from "react";
import { useTheme, PALETTES } from "../../hooks/useTheme";
import { BRANDING, DEFAULTS } from "../../utils/constants";
import type { AccentKey, Density, FontSize, SnippetLines, Theme } from "../../hooks/useTheme";

const DENSITY_OPTIONS: { value: Density; label: string }[] = [
  { value: "title", label: "Title only" },
  { value: "card", label: "Card" },
];

const SNIPPET_LINE_OPTIONS: { value: SnippetLines; label: string }[] = [
  { value: 0, label: "Off" },
  { value: 1, label: "1" },
  { value: 2, label: "2" },
  { value: 3, label: "3" },
];

export function Appearance() {
  const {
    theme,
    setTheme,
    accent,
    setAccent,
    fontSize,
    setFontSize,
    density,
    setDensity,
    snippetLines,
    setSnippetLines,
    hideThumbnails,
    setHideThumbnails,
    highContrast,
    setHighContrast,
    siteTitle,
    setSiteTitle,
    customLogo,
    setCustomLogo,
  } = useTheme();
  const [logoError, setLogoError] = useState<string | null>(null);

  function saveLogo(logo: string | null) {
    setLogoError(setCustomLogo(logo) ? null : "Could not save the logo: browser storage is full.");
  }

  function handleLogoFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (!file) return;
    if (!file.type.startsWith("image/")) {
      setLogoError("Please choose an image file.");
      return;
    }
    if (file.size > BRANDING.maxLogoBytes) {
      setLogoError(`Image is too large (max ${BRANDING.maxLogoBytes / 1024} KB).`);
      return;
    }
    const reader = new FileReader();
    reader.onload = () => saveLogo(reader.result as string);
    reader.onerror = () => setLogoError("Could not read the image.");
    reader.readAsDataURL(file);
  }

  const logoUrl = customLogo && !customLogo.startsWith("data:") ? customLogo : "";

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">Theme</h2>
      <div className="settings-field">
        <label>Mode</label>
        <div className="segmented">
          {(["light", "dark"] as Theme[]).map((t) => (
            <button
              key={t}
              className={`segmented-btn${theme === t ? " active" : ""}`}
              onClick={() => setTheme(t)}
            >
              {t === "light" ? "Light" : "Dark"}
            </button>
          ))}
        </div>
      </div>
      <div className="settings-field">
        <label>High contrast</label>
        <input type="checkbox" checked={highContrast} onChange={(e) => setHighContrast(e.target.checked)} />
      </div>

      <h2 className="settings-section-title">Accent color</h2>
      <div className="appearance-accents">
        {(Object.keys(PALETTES) as AccentKey[]).map((key) => (
          <button
            key={key}
            className={`appearance-accent-btn${accent === key ? " active" : ""}`}
            style={{ "--dot-color": PALETTES[key].light } as React.CSSProperties}
            onClick={() => setAccent(key)}
            title={key}
          >
            <span className="appearance-accent-dot" />
            <span className="appearance-accent-label">{key}</span>
          </button>
        ))}
      </div>

      <h2 className="settings-section-title">Font size</h2>
      <div className="settings-field">
        <label>Size</label>
        <div className="segmented">
          {(["small", "default", "large"] as FontSize[]).map((s) => (
            <button
              key={s}
              className={`segmented-btn${fontSize === s ? " active" : ""}`}
              onClick={() => setFontSize(s)}
            >
              {s.charAt(0).toUpperCase() + s.slice(1)}
            </button>
          ))}
        </div>
      </div>
      <h2 className="settings-section-title">Post list</h2>
      <div className="settings-field">
        <label>Rows</label>
        <div className="segmented">
          {DENSITY_OPTIONS.map(({ value, label }) => (
            <button
              key={value}
              className={`segmented-btn${density === value ? " active" : ""}`}
              onClick={() => setDensity(value)}
            >
              {label}
            </button>
          ))}
        </div>
      </div>
      {density === "card" && (
        <div className="settings-field">
          <label>Hide thumbnails</label>
          <input
            type="checkbox"
            checked={hideThumbnails}
            onChange={(e) => setHideThumbnails(e.target.checked)}
          />
        </div>
      )}
      {density === "card" && (
        <div className="settings-field">
          <label>Snippet lines</label>
          <div className="segmented">
            {SNIPPET_LINE_OPTIONS.map(({ value, label }) => (
              <button
                key={value}
                className={`segmented-btn${snippetLines === value ? " active" : ""}`}
                onClick={() => setSnippetLines(value)}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      )}

      <h2 className="settings-section-title">Branding</h2>
      <div className="settings-field">
        <label>Tab title</label>
        <input
          type="text"
          placeholder={DEFAULTS.siteTitle}
          value={siteTitle}
          onChange={(e) => setSiteTitle(e.target.value)}
        />
      </div>
      <div className="settings-field">
        <label>Logo URL</label>
        <input
          type="url"
          placeholder={customLogo?.startsWith("data:") ? "Uploaded image" : "https://example.com/logo.svg"}
          value={logoUrl}
          onChange={(e) => saveLogo(e.target.value.trim() || null)}
        />
      </div>
      <div className="settings-field">
        <label>Logo image</label>
        <div className="appearance-logo">
          {customLogo && <img className="appearance-logo-preview" src={customLogo} alt="Custom logo" />}
          <label className="btn btn-ghost">
            Upload…
            <input type="file" accept="image/*" onChange={handleLogoFile} style={{ display: "none" }} />
          </label>
          {customLogo && (
            <button className="btn btn-ghost" onClick={() => saveLogo(null)}>
              Reset
            </button>
          )}
        </div>
      </div>
      {logoError && <p className="settings-status settings-status-err">{logoError}</p>}
      <p className="settings-hint">Changes apply immediately and are saved on this device.</p>
    </div>
  );
}
