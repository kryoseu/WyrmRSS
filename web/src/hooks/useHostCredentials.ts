import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createHostCredential,
  deleteHostCredential,
  getHostCredentials,
  updateHostCredential,
} from "../api/hostCredentials";
import type { CreateHostCredential } from "../types/CreateHostCredential";
import type { UpdateHostCredential } from "../types/UpdateHostCredential";
import type { HostCredentialId } from "../types/HostCredentialId";

export const hostCredentialKeys = {
  all: ["hostCredentials"] as const,
};

export function useHostCredentials() {
  return useQuery({
    queryKey: hostCredentialKeys.all,
    queryFn: getHostCredentials,
    // Every mutation invalidates this key, so don't refetch on each mount.
    staleTime: Infinity,
  });
}

export function useCreateHostCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateHostCredential) => createHostCredential(body),
    onSuccess: () => qc.invalidateQueries({ queryKey: hostCredentialKeys.all }),
  });
}

export function useUpdateHostCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, body }: { id: HostCredentialId; body: UpdateHostCredential }) =>
      updateHostCredential(id, body),
    onSuccess: () => qc.invalidateQueries({ queryKey: hostCredentialKeys.all }),
  });
}

export function useDeleteHostCredential() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: HostCredentialId) => deleteHostCredential(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: hostCredentialKeys.all }),
  });
}
