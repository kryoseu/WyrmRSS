import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFolder, deleteFolder, getFolders, updateFolder } from "../api/folders";
import { feedKeys } from "../cache/feeds";
import type { CreateFolder } from "../types/CreateFolder";
import type { UpdateFolder } from "../types/UpdateFolder";
import type { FolderId } from "../types/FolderId";

export const folderKeys = {
  all: ["folders"] as const,
};

export function useFolders() {
  return useQuery({
    queryKey: folderKeys.all,
    queryFn: getFolders,
    // Config-ish data: mutations (and feed create/update, which can
    // resolve-or-create a folder) invalidate this key explicitly.
    staleTime: Infinity,
  });
}

export function useCreateFolder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateFolder) => createFolder(body),
    onSuccess: () => qc.invalidateQueries({ queryKey: folderKeys.all }),
  });
}

export function useUpdateFolder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, body }: { id: FolderId; body: UpdateFolder }) =>
      updateFolder(id, body),
    // Feeds only store folder_id and names are looked up from this query,
    // so a rename doesn't touch the feeds cache.
    onSuccess: () => qc.invalidateQueries({ queryKey: folderKeys.all }),
  });
}

export function useDeleteFolder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: FolderId) => deleteFolder(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: folderKeys.all });
      // Deleting SET-NULLs feeds.folder_id server-side; cached feeds still
      // point at the dead folder until refetched.
      qc.invalidateQueries({ queryKey: feedKeys.all });
    },
  });
}
