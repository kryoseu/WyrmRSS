import { useRef } from "react";
import { useShortcutHandler } from "../../hooks/useShortcuts";

interface Props {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  /** More actions displayed after the search input, e.g. a refresh button. */
  children?: React.ReactNode;
}

export function PostsToolbar({ value, onChange, placeholder = "Search posts…", children }: Props) {
  const searchRef = useRef<HTMLInputElement>(null);

  useShortcutHandler("focus_search", () => searchRef.current?.focus());

  return (
    <div className="posts-toolbar">
      <input
        ref={searchRef}
        className="posts-search"
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        // Hands the keyboard back to the shortcuts.
        onKeyDown={(e) => e.key === "Escape" && e.currentTarget.blur()}
      />
      {children}
    </div>
  );
}
