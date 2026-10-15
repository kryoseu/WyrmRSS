import { useEffect, useEffectEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getShortcuts, resetShortcut, updateShortcut } from "../api/shortcuts";
import type { ShortcutAction } from "../types/ShortcutAction";
import { keyFromEvent, shouldIgnoreKeypress } from "../utils/shortcuts";

export const shortcutKeys = {
  all: ["shortcuts"] as const,
};

export function useShortcuts() {
  return useQuery({
    queryKey: shortcutKeys.all,
    queryFn: getShortcuts,
    staleTime: Infinity,
  });
}

export function useUpdateShortcut() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ action, key }: { action: ShortcutAction; key: string }) => updateShortcut(action, { key }),
    onSuccess: (data) => qc.setQueryData(shortcutKeys.all, data),
  });
}

export function useResetShortcut() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: resetShortcut,
    onSuccess: (data) => qc.setQueryData(shortcutKeys.all, data),
  });
}

/** Runs `handler` when the key bound to `action` is pressed outside a text field or dialog. */
export function useShortcutHandler(action: ShortcutAction, handler: () => void, enabled = true) {
  const key = useShortcuts().data?.[action];
  const onPress = useEffectEvent(handler);

  useEffect(() => {
    if (!key || !enabled) return;
    function onKeyDown(e: KeyboardEvent) {
      if (shouldIgnoreKeypress(e) || keyFromEvent(e) !== key) return;
      e.preventDefault();
      onPress();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [key, enabled]);
}
