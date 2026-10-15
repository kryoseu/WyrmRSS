interface Props {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  /** More actions displayed after the search input, e.g. a refresh button. */
  children?: React.ReactNode;
}

export function PostsToolbar({ value, onChange, placeholder = "Search posts…", children }: Props) {
  return (
    <div className="posts-toolbar">
      <input
        className="posts-search"
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
      {children}
    </div>
  );
}
