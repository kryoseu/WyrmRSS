import { memo } from "react";
import { Link } from "react-router-dom";
import { TbArchiveOff } from "react-icons/tb";
import type { PostArchive } from "../types/PostArchive";
import { useUnarchivePost } from "../hooks/usePostMutations";

interface Props {
  archive: PostArchive;
  active: boolean;
}

export const ArchiveItem = memo(function ArchiveItem({ archive, active }: Props) {
  const { mutate: unarchive } = useUnarchivePost();

  function handleUnarchive(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    unarchive(archive.id);
  }

  return (
    <Link
      to={`/archive/${archive.id}`}
      className={`post-item${active ? " active" : ""}`}
    >
      {archive.tag && (
        <span
          className="post-item-tag"
          style={archive.tag_color ? ({ "--tag-color": archive.tag_color } as React.CSSProperties) : undefined}
        >
          {archive.tag}
        </span>
      )}
      <span className="post-item-title">{archive.title ?? "Untitled"}</span>
      <button
        className="post-item-archive"
        onClick={handleUnarchive}
        aria-label="Unarchive"
      >
        <TbArchiveOff />
      </button>
    </Link>
  );
});
