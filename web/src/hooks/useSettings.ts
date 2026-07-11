import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getSettings, importOpml, updateSettings } from "../api/settings";
import { pollFeeds } from "../api/feeds";
import { feedKeys } from "../cache/feeds";
import { postKeys } from "../cache/posts";
import { folderKeys } from "./useFolders";
import type { UpdateSettings } from "../types/UpdateSettings";

export function useSettings() {
  return useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
    staleTime: Infinity,
  });
}

export function useUpdateSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: UpdateSettings) => updateSettings(body),
    onSuccess: (data) => {
      qc.setQueryData(["settings"], data);
    },
  });
}

export function useImportOpml() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (file: File) => importOpml(file),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: feedKeys.all });
      // Import resolves-or-creates folders from OPML categories.
      qc.invalidateQueries({ queryKey: folderKeys.all });
      // The import response waits (bounded) for the post-import poll, so the
      // invalidations above usually land with posts and icons already there.
      // Poll again as a safety net for huge imports that outlast the server's
      // wait (the worker queue is sequential, and re-polling already-fetched
      // feeds is a no-op), then pull in the new posts and unread counts.
      const refresh = () => {
        qc.invalidateQueries({ queryKey: postKeys.all });
        qc.invalidateQueries({ queryKey: feedKeys.all });
      };
      pollFeeds().then(refresh).catch(refresh);
    }
  });
}

