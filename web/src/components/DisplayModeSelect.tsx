import { DISPLAY_MODE_DESCRIPTIONS, DISPLAY_MODE_LABELS } from "../utils/displayMode";
import type { DisplayMode } from "../types/DisplayMode";

interface Props {
  value: DisplayMode;
  onChange: (value: DisplayMode) => void;
}

// Shared by AddFeedForm/EditFeedForm: controls how a feed's posts are shown.
// Native <option title> tooltips render off-position in most browsers, so the
// description is shown as static text under the select instead.
export function DisplayModeSelect({ value, onChange }: Props) {
  return (
    <div className="display-mode-field">
      <select value={value} onChange={(e) => onChange(e.target.value as DisplayMode)}>
        <option value="river">{DISPLAY_MODE_LABELS.river}</option>
        <option value="feed_only">{DISPLAY_MODE_LABELS.feed_only}</option>
        <option value="radar">{DISPLAY_MODE_LABELS.radar}</option>
      </select>
      <p className="field-hint">{DISPLAY_MODE_DESCRIPTIONS[value]}</p>
    </div>
  );
}
