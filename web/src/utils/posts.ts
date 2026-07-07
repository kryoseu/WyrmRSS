import type { Post } from "../types/Post";
import type { FeedId } from "../types/FeedId";
import { getDateLabel } from "./utils";

// Builds the route path for a post depending on context (all posts, a specific feed, or bookmarks).
export function postPath(p: Post, feedId?: FeedId, isBookmarked?: boolean): string {
  if (isBookmarked) return `/read-later/${p.id}`;
  if (feedId !== undefined) return `/feeds/${p.feed_id}/posts/${p.id}`;
  return `/feeds/posts/${p.id}`;
}

// Generates 1–2 character initials from a name for use in avatars.
export function initials(name: string): string {
  const words = name.trim().split(/\s+/).filter((w) => /^[a-zA-Z0-9]/.test(w));
  if (words.length === 0) return "?";
  if (words.length === 1) return words[0].replace(/[^a-zA-Z0-9]/g, "").slice(0, 2).toUpperCase();
  return (words[0][0] + words[1][0]).toUpperCase();
}

// Groups consecutive items sharing a date label, preserving list order.
// `getDate` must return the field the list is sorted by — labeling by any
// other field puts headers out of order. Grouping runs (not a global map)
// guarantees grouping never moves an item relative to the backend sort.
export function groupByDate<T>(items: T[], getDate: (item: T) => string): [string, T[]][] {
  const groups: [string, T[]][] = [];
  for (const item of items) {
    const label = getDateLabel(getDate(item));
    const last = groups[groups.length - 1];
    if (last && last[0] === label) {
      last[1].push(item);
    } else {
      groups.push([label, [item]]);
    }
  }
  return groups;
}
