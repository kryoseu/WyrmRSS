import { useOutletContext, useParams } from "react-router";
import { useBookmarkedPosts } from "../../hooks/usePosts";
import { useFeeds } from "../../hooks/useFeeds";
import { useDebouncedSearch } from "../../hooks/useDebouncedSearch";
import { useOpenPostFromRoute } from "../../hooks/useOpenPostFromRoute";
import { usePostNavigation } from "../../hooks/usePostNavigation";
import { useFeedMap } from "../../hooks/useFeedMap";
import { useFlattenedPages } from "../../hooks/useFlattenedPages";
import { DensityToggle } from "./DensityToggle";
import { PostItem } from "./PostItem";
import { PostsToolbar } from "./PostsToolbar";
import { VirtualGroupedList } from "../common/VirtualGroupedList";
import type { ReaderOutletContext } from "../../pages/ReaderPage";
import type { Post } from "../../types/Post";
import { postPath } from "../../utils/posts";

const groupDate = (post: Post) => post.created_at;
const pathFor = (post: Post) => postPath(post, undefined, true);

export function BookmarkedPostList() {
  const { activePostId, onOpenPost } = useOutletContext<ReaderOutletContext>();

  const { postId } = useParams();

  const { data: feeds } = useFeeds();

  const { search, setSearch, debouncedSearch } = useDebouncedSearch();

  const { data, isLoading, hasNextPage, fetchNextPage, isFetchingNextPage } =
    useBookmarkedPosts(debouncedSearch || undefined);

  useOpenPostFromRoute(postId, onOpenPost);

  const feedMap = useFeedMap(feeds);

  const posts = useFlattenedPages(data);

  usePostNavigation({
    items: posts,
    activePostId,
    onOpenPost,
    pathFor,
    hasNextPage,
    fetchNextPage,
  });

  return (
    <div className="pane pane-posts">
      <PostsToolbar value={search} onChange={setSearch} placeholder="Search read later…">
        <DensityToggle />
      </PostsToolbar>
      <VirtualGroupedList
        items={posts}
        isLoading={isLoading}
        hasNextPage={hasNextPage}
        fetchNextPage={fetchNextPage}
        isFetchingNextPage={isFetchingNextPage}
        groupDate={groupDate}
        emptyMessage="Nothing saved for later"
        activeId={activePostId}
        renderItem={(post) => (
          <PostItem
            post={post}
            to={pathFor(post)}
            active={activePostId === post.id}
            feed={feedMap.get(post.feed_id)}
          />
        )}
      />
    </div>
  );
}
