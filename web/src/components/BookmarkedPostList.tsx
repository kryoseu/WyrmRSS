import { useOutletContext, useParams } from "react-router-dom";
import { useBookmarkedPosts } from "../hooks/usePosts";
import { useFeeds } from "../hooks/useFeeds";
import { useDebouncedSearch } from "../hooks/useDebouncedSearch";
import { useOpenPostFromRoute } from "../hooks/useOpenPostFromRoute";
import { useFeedMap } from "../hooks/useFeedMap";
import { useFlattenedPages } from "../hooks/useFlattenedPages";
import { PostItem } from "./PostItem";
import { PostsToolbar } from "./PostsToolbar";
import { VirtualGroupedList } from "./VirtualGroupedList";
import type { ReaderOutletContext } from "../pages/ReaderPage";
import { postPath } from "../utils/posts";

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

  return (
    <div className="pane pane-posts">
      <PostsToolbar value={search} onChange={setSearch} placeholder="Search read later…" />
      <VirtualGroupedList
        items={posts}
        isLoading={isLoading}
        hasNextPage={hasNextPage}
        fetchNextPage={fetchNextPage}
        isFetchingNextPage={isFetchingNextPage}
        emptyMessage="Nothing saved for later"
        renderItem={(post) => (
          <PostItem
            post={post}
            to={postPath(post, undefined, true)}
            active={activePostId === post.id}
            feed={feedMap.get(post.feed_id)}
          />
        )}
      />
    </div>
  );
}
