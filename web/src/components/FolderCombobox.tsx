import { useRef, useState } from "react";
import { useFolders } from "../hooks/useFolders";

interface Props {
  value: string;
  onChange: (name: string) => void;
}

/**
 * Free-text folder picker for the feed forms: type to filter existing
 * folders, pick one, or keep typing to create a new one on save (the server
 * resolves-or-creates by name). Selecting an existing folder adopts its
 * canonical casing.
 */
export function FolderCombobox({ value, onChange }: Props) {
  const { data: folders } = useFolders();
  const [open, setOpen] = useState(false);
  const [dropUp, setDropUp] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // The list is capped at 160px (App.css); open it upward when the input sits
  // too close to the bottom of the viewport for the list to fit — e.g. the
  // sidebar's add-feed form, which is anchored at the bottom.
  function openList() {
    const rect = inputRef.current?.getBoundingClientRect();
    if (rect) setDropUp(window.innerHeight - rect.bottom < 180);
    setOpen(true);
  }

  const query = value.trim().toLowerCase();
  const matches = (folders ?? [])
    .filter((f) => f.name.toLowerCase().includes(query))
    .sort((a, b) => a.name.localeCompare(b.name));
  const exactMatch = matches.some((f) => f.name.toLowerCase() === query);
  const showCreate = query !== "" && !exactMatch;

  return (
    <div className="folder-combobox">
      <input
        ref={inputRef}
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
          openList();
        }}
        onFocus={openList}
        onBlur={() => setOpen(false)}
        placeholder="Folder (optional)"
        aria-label="Folder"
      />
      {open && (matches.length > 0 || showCreate) && (
        <div className={`folder-combobox-list${dropUp ? " drop-up" : ""}`}>
          {matches.map((f) => (
            <button
              key={f.id}
              type="button"
              className="folder-combobox-option"
              // mousedown so the pick lands before the input's blur closes the list
              onMouseDown={(e) => {
                e.preventDefault();
                onChange(f.name);
                setOpen(false);
              }}
            >
              {f.name}
            </button>
          ))}
          {showCreate && (
            <button
              type="button"
              className="folder-combobox-option folder-combobox-create"
              // The create row keeps the typed text as-is; the server creates it on save.
              onMouseDown={(e) => {
                e.preventDefault();
                setOpen(false);
              }}
            >
              Create “{value.trim()}”
            </button>
          )}
        </div>
      )}
    </div>
  );
}
