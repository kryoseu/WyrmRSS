import { useMemo } from "react";
import type { Feed } from "../types/Feed";
import type { FeedMeta } from "../components/PostItem";

/**
 * Maps feed id - display metadata for post rows. Memoised so the FeedMeta
 * object references stay stable between renders and React.memo on PostItem can bail out
 */
export function useFeedMap(feeds: Feed[] | undefined): Map<number, FeedMeta> {
  return useMemo(
    () =>
      new Map(
        (feeds ?? []).map((f): [number, FeedMeta] => [
          f.id,
          { name: f.title, tag: f.tag, tagColor: f.tag_color },
        ])
      ),
    [feeds]
  );
}
