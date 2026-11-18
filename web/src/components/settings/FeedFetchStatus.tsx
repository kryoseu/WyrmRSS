import { useRef, useState } from "react";
import { useDismiss } from "../../hooks/useDismiss";
import type { Feed } from "../../types/Feed";
import { formatLastFetched } from "../../utils/utils";

export function FeedFetchStatus({ feed }: { feed: Feed }) {
  const time = feed.last_fetched_at ? formatLastFetched(feed.last_fetched_at) : "Not fetched";

  if (feed.last_error) {
    return <FeedFetchError title={feed.title} error={feed.last_error} time={time} />;
  }

  const status = feed.last_fetched_at ? "healthy" : "never";
  return (
    <span className="feed-fetch-status settings-table-meta" aria-label={`${status === "healthy" ? "Healthy" : "Not fetched"}: ${time}`}>
      <span className={`feed-fetch-dot ${status}`} aria-hidden="true" />
      {time}
    </span>
  );
}

/** "Failed" label that toggles a small bubble with the error, anchored below it. */
function FeedFetchError({ title, error, time }: { title: string; error: string; time: string }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLSpanElement>(null);
  useDismiss(ref, open, () => setOpen(false));

  return (
    <span className="feed-fetch-wrap settings-table-meta" ref={ref}>
      <button
        type="button"
        className="feed-fetch-status feed-fetch-error"
        title={error}
        aria-label={`Fetch failed for ${title}: ${error}. Last success: ${time}`}
        aria-expanded={open}
        onClick={() => setOpen(!open)}
      >
        <span className="feed-fetch-dot error" aria-hidden="true" />
        Failed
      </button>
      {open && (
        <span className="feed-fetch-bubble" role="status">
          <b>Error:</b> {error}
          <br />
          Last success: {time}
        </span>
      )}
    </span>
  );
}
