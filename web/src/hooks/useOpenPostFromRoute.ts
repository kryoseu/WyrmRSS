import { useEffect } from "react";

/** Open the post named by the `:postId` route param, if one is present. */
export function useOpenPostFromRoute(
  postId: string | undefined,
  onOpenPost: (id: number) => void
) {
  useEffect(() => {
    if (postId) onOpenPost(Number(postId));
  }, [postId, onOpenPost]);
}
