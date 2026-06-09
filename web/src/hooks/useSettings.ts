import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getSettings, importOpml, updateSettings } from "../api/settings";
import type { UpdateSettings } from "../types/UpdateSettings";

export function useSettings() {
  return useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
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
      qc.invalidateQueries({ queryKey: ["feeds"] });
    }
  });
}

