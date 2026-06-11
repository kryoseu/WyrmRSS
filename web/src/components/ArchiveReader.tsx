import { TbArchiveOff } from "react-icons/tb";
import { useArchivedPost, useUnarchivePost } from "../hooks/usePosts";
import { formatDate } from "../utils/utils";
import { ReaderPane } from "./ReaderPane";

interface Props {
  postId: number | null;
  onClose: () => void;
  width: number;
}

export function ArchiveReader({ postId, onClose, width }: Props) {
  const { data: archive, isLoading } = useArchivedPost(postId ?? undefined);
  const { mutate: unarchive } = useUnarchivePost();

  if (!postId) return null;

  const body = archive?.content ?? archive?.description;
  const date = formatDate(archive?.published_at ?? null);

  const actions = archive && (
    <button
      className="pane-reader-archive archived"
      onClick={() => unarchive(archive.id, { onSuccess: onClose })}
      aria-label="Unarchive"
    >
      <TbArchiveOff />
    </button>
  );

  return (
    <ReaderPane width={width} onClose={onClose} actions={actions} isLoading={isLoading} notFound={!isLoading && !archive} notFoundMessage="Archive not found">
      {archive && (
        <div className="post-reader">
          <h1 className="post-reader-title">{archive.title ?? "Untitled"}</h1>
          <div className="post-reader-meta">
            {archive.authors && <span>{archive.authors}</span>}
            {date && <span>{date}</span>}
            {archive.url && (
              <a href={archive.url} target="_blank" rel="noopener noreferrer">
                Open original ↗
              </a>
            )}
          </div>
          {body && (
            <div className="post-reader-body" dangerouslySetInnerHTML={{ __html: body }} />
          )}
        </div>
      )}
    </ReaderPane>
  );
}
