export function formatDate(iso: string | null): string {
  if (!iso) return "";
  return new Date(iso).toLocaleDateString(undefined, {
    month: "long",
    day: "numeric",
    year: "numeric",
  });
}

// Returns a post-list group label: "Today · Jan 1, 2025", "Yesterday · ...", or just the date.
export function getDateLabel(iso: string): string {
  const date = new Date(iso);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);

  const formatted = formatDate(iso);
  if (date.toDateString() === today.toDateString()) return `Today · ${formatted}`;
  if (date.toDateString() === yesterday.toDateString()) return `Yesterday · ${formatted}`;
  return formatted;
}

// Formats an ISO timestamp as a human-readable relative time (e.g. "2h ago", "Never").
export function formatLastFetched(timestamp: string | undefined): string {
  if (!timestamp) return "Never";
  const diff = Date.now() - new Date(timestamp).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "Just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

// Extracts the video ID from youtube.com and youtu.be URLs; returns null for non-YouTube URLs.
export function extractYouTubeId(url: string | null): string | null {
  if (!url) return null;
  try {
    const u = new URL(url);
    if (u.hostname === "www.youtube.com" || u.hostname === "youtube.com") {
      return u.searchParams.get("v");
    }
    if (u.hostname === "youtu.be") {
      return u.pathname.slice(1) || null;
    }
  } catch {
    // not a valid URL
  }
  return null;
}
