import { useEffect, useState } from "react";
import { TIMING } from "../utils/constants";

export function useDebouncedSearch(delay = TIMING.searchDebounceMs) {
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");

  // Delay the query until the user stops typing
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search), delay);
    return () => clearTimeout(timer);
  }, [search, delay]);

  return { search, setSearch, debouncedSearch };
}
