import { useMemo } from "react";
import type { Feed } from "../types/Feed";
import type { FeedId } from "../types/FeedId";
import type { FeedMeta } from "../components/PostItem";

/**
 * Maps feed id - display metadata for post rows. Memoised so the FeedMeta
 * object references stay stable between renders and React.memo on PostItem can bail out
 */
export function useFeedMap(feeds: Feed[] | undefined): Map<FeedId, FeedMeta> {
  return useMemo(
    () =>
      new Map(
        (feeds ?? []).map((f): [FeedId, FeedMeta] => [f.id, { name: f.title }])
      ),
    [feeds]
  );
}
