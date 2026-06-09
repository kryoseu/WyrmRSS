import { useTheme, PALETTES } from "../../hooks/useTheme";
import type { AccentKey, FontSize, Theme } from "../../hooks/useTheme";

export function Appearance() {
  const { theme, setTheme, accent, setAccent, fontSize, setFontSize } = useTheme();

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
      <p className="settings-hint">Changes apply immediately.</p>
    </div>
  );
}
