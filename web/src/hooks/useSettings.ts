import { useMutation, useQueryClient } from "@tanstack/react-query";
import { importOpml } from "../api/settings";

export function useImportOpml() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (file: File) => importOpml(file),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["feeds"] });
    }
  });
}

