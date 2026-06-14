import { useMemo } from "react";
import type { InfiniteData } from "@tanstack/react-query";
import type { PagedResponse } from "../types/PagedResponse";

/** Flattens the pages of an infinite query into a single list. */
export function useFlattenedPages<T>(
  data: InfiniteData<PagedResponse<T[]>> | undefined
): T[] | undefined {
  return useMemo(() => data?.pages.flatMap((p) => p.items), [data]);
}
