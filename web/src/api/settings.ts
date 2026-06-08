import { ENDPOINTS, noContent } from "../utils/api";

export const importOpml = (file: File): Promise<void> =>
  fetch(ENDPOINTS.settings.import(), {
    method: "POST",
    headers: { "Content-Type": "application/xml" },
    body: file,
  }).then(noContent);

