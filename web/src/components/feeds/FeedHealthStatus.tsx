import { useId, useState } from "react";
import type { FeedView } from "../../types/FeedView";

function timestamp(value?: string) {
  return value ? new Date(value).toLocaleString() : "Never";
}

/** Diagnostics expand below the row, available to touch and keyboard users. */
export function FeedHealthStatus({ feed }: { feed: FeedView }) {
  const [expanded, setExpanded] = useState(false);
  const detailsId = useId();
  const health = feed.health;
  const error = health?.last_error;
  const status = feed.is_paused ? "Paused" : error ? "Refresh failed" :
    health?.last_success_at ? "Healthy" : "Not checked yet";
  const due = health?.next_check_at;
  const nextCheck = feed.is_paused ? "Paused" : !due
    ? "Due on the next polling cycle" : `After ${timestamp(due)} (next polling cycle)`;

  return (
    <>
      <button
        type="button"
        className={`feed-health${error ? " feed-health--error" : ""}`}
        aria-label={`Feed health for ${feed.title}: ${status}`}
        aria-expanded={expanded}
        aria-controls={expanded ? detailsId : undefined}
        onClick={() => setExpanded((value) => !value)}
      >
        <span aria-hidden="true">{expanded ? "▾" : "▸"}</span> {status}
      </button>
      {expanded && (
        <dl id={detailsId} className="feed-health-details">
          <dt>Last checked</dt><dd>{timestamp(health?.last_checked_at)}</dd>
          <dt>Last successful check</dt><dd>{timestamp(health?.last_success_at ?? feed.last_fetched_at)}</dd>
          <dt>Next eligible check</dt><dd>{nextCheck}</dd>
          {error && <><dt>Latest error</dt><dd>{error}</dd></>}
        </dl>
      )}
    </>
  );
}
