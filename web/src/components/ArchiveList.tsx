import { useOutletContext, useParams } from "react-router";
import { useArchivedPosts } from "../hooks/usePosts";
import { useDebouncedSearch } from "../hooks/useDebouncedSearch";
import { useOpenPostFromRoute } from "../hooks/useOpenPostFromRoute";
import { useFlattenedPages } from "../hooks/useFlattenedPages";
import { ArchiveItem } from "./ArchiveItem";
import { PostsToolbar } from "./PostsToolbar";
import { VirtualGroupedList } from "./VirtualGroupedList";
import type { ReaderOutletContext } from "../pages/ReaderPage";
import type { PostArchive } from "../types/PostArchive";

const groupDate = (archive: PostArchive) => archive.archived_at;

export function ArchiveList() {
  const { activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();

  const { postId } = useParams();

  const { search, setSearch, debouncedSearch } = useDebouncedSearch();

  const {
    data,
    isLoading,
    hasNextPage,
    fetchNextPage,
    isFetchingNextPage
  } = useArchivedPosts({ search: debouncedSearch || undefined });

  useOpenPostFromRoute(postId, onOpenPost);

  const archives = useFlattenedPages(data);

  return (
    <div className="pane pane-posts">
      <PostsToolbar value={search} onChange={setSearch} placeholder="Search archive…" />
      <VirtualGroupedList
        items={archives}
        isLoading={isLoading}
        hasNextPage={hasNextPage}
        fetchNextPage={fetchNextPage}
        isFetchingNextPage={isFetchingNextPage}
        groupDate={groupDate}
        emptyMessage="No archived posts"
        renderItem={(archive) => (
          <ArchiveItem
            archive={archive}
            active={activePostId === archive.id}
          />
        )}
      />
    </div>
  );
}
