import type { Post } from "../types/Post";
import { getDateLabel } from "./utils";

// Builds the route path for a post depending on context (all posts, a specific feed, or favorites).
export function postPath(p: Post, feedId?: number, isFavorites?: boolean): string {
  if (isFavorites) return `/favorites/${p.id}`;
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

export function groupByDate(posts: Post[]): [string, Post[]][] {
  const map = new Map<string, Post[]>();
  for (const post of posts) {
    const label = getDateLabel(post.published_at);
    if (!map.has(label)) map.set(label, []);
    map.get(label)!.push(post);
  }
  return Array.from(map.entries());
}
