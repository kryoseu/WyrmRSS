export interface Tag {
  name: string;
  color?: string;
}

interface Props {
  tags: Tag[];
  activeTag: string | undefined;
  onToggle: (tag: string) => void;
}

export function PostsTagChips({ tags, activeTag, onToggle }: Props) {
  if (tags.length === 0) return null;
  return (
    <div className="posts-tags">
      {tags.map(({ name, color }) => (
        <button
          key={name}
          className={`posts-tag-chip${activeTag === name ? " active" : ""}`}
          style={{ "--tag-color": color ?? "var(--text)" } as React.CSSProperties}
          onClick={() => onToggle(name)}
        >
          {name}
        </button>
      ))}
    </div>
  );
}
