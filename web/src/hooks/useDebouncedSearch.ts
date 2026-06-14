import { useEffect, useState } from "react";

export function useDebouncedSearch(delay = 300) {
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");

  // Delay the query until the user stops typing
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search), delay);
    return () => clearTimeout(timer);
  }, [search, delay]);

  return { search, setSearch, debouncedSearch };
}
