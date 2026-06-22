import { useEffect } from "react";
import type { PostId } from "../types/PostId";

/** Open the post named by the `:postId` route param, if one is present. */
export function useOpenPostFromRoute(
  postId: string | undefined,
  onOpenPost: (id: PostId) => void
) {
  useEffect(() => {
    if (postId) onOpenPost(Number(postId));
  }, [postId, onOpenPost]);
}
