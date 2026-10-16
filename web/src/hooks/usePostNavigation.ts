import { useEffect, useRef } from "react";
import { useLocation, useNavigate } from "react-router";
import { useShortcutHandler } from "./useShortcuts";
import type { FeedId } from "../types/FeedId";
import type { PostId } from "../types/PostId";
import { listPath } from "../utils/posts";

interface Options<T> {
  items: T[] | undefined;
  activePostId: PostId | null;
  onOpenPost: (id: PostId) => void;
  pathFor: (item: T) => string;
  hasNextPage: boolean | undefined;
  fetchNextPage: () => void;
}

interface OpenedPost {
  id: PostId;
  index: number;
  feedId?: FeedId;
  list: string;
}

/** Next/previous post shortcuts, walking the list in the order it's displayed. */
export function usePostNavigation<T extends { id: PostId; feed_id?: FeedId }>({
  items = [],
  activePostId,
  onOpenPost,
  pathFor,
  hasNextPage,
  fetchNextPage,
}: Options<T>) {
  const navigate = useNavigate();
  const list = listPath(useLocation().pathname);
  const activeIndex = items.findIndex((item) => item.id === activePostId);
  const activeItem = activeIndex >= 0 ? items[activeIndex] : undefined;

  // Last opened post, to continue from once it leaves the list or the reader closes.
  const lastOpened = useRef<OpenedPost | null>(null);
  useEffect(() => {
    if (activeItem) lastOpened.current = { id: activeItem.id, index: activeIndex, feedId: activeItem.feed_id, list };
  }, [activeItem, activeIndex, list]);

  // Our position in the displayed list (all posts, a feed, read later or archive), as an
  // index into `items`; step() opens the first item after it or the last one before it.
  // A half index means "between two items", so neither neighbour is skipped.
  function currentPosition(last: OpenedPost | null): number {
    // The open post is in the list: start from it.
    if (activeIndex >= 0) return activeIndex;
    // Nothing opened in this list yet: just before the first item.
    if (!last) return -0.5;
    // The open post left the list, e.g. [A, B, C] → B unbookmarked → [A, C].
    // B was at 1, so 0.5 sits between A and C: next is C, previous is A.
    if (activePostId !== null) return last.index - 0.5;
    // The reader is closed: continue from the last opened post if it's still listed.
    const index = items.findIndex((item) => item.id === last.id);
    return index >= 0 ? index : -0.5;
  }

  // Opens the next or previous post from our position. With `sameFeed`, posts from
  // other feeds are skipped. Past the last loaded post, "next" loads the next page.
  function step(direction: "next" | "prev", { sameFeed = false } = {}) {
    // A post opened in another list says nothing about where we are in this one.
    const last = lastOpened.current?.list === list ? lastOpened.current : null;
    const position = currentPosition(last);
    const feedId = last?.feedId;
    // In-feed needs a current post to take the feed from; archived posts have none.
    if (sameFeed && (feedId === undefined || position < 0)) return;
    const matches = (item: T) => !sameFeed || item.feed_id === feedId;
    // First match after our position, or last match before it.
    const target = direction === "next"
      ? items.find((item, i) => i > position && matches(item))
      : items.findLast((item, i) => i < position && matches(item));
    if (target) {
      // Navigating alone won't reopen a closed post whose URL is still current.
      onOpenPost(target.id);
      // Replace while reading, so back still returns to the list.
      navigate(pathFor(target), { replace: activePostId !== null });
    } else if (direction === "next" && hasNextPage) {
      fetchNextPage();
    }
  }

  useShortcutHandler("next_post", () => step("next"));
  useShortcutHandler("prev_post", () => step("prev"));
  useShortcutHandler("next_post_in_feed", () => step("next", { sameFeed: true }));
  useShortcutHandler("prev_post_in_feed", () => step("prev", { sameFeed: true }));
}
