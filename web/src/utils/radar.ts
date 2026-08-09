// Strips HTML and truncates for a short one-line preview (Radar cards).
export function plainTextSnippet(html: string, maxLength = 140): string {
  const text = new DOMParser().parseFromString(html, "text/html").body.textContent ?? "";
  const trimmed = text.trim().replace(/\s+/g, " ");
  return trimmed.length > maxLength ? `${trimmed.slice(0, maxLength).trimEnd()}…` : trimmed;
}
