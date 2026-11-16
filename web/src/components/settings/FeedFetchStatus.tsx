import { useId } from "react";
import type { Feed } from "../../types/Feed";
import { formatLastFetched } from "../../utils/utils";

export function FeedFetchStatus({ feed }: { feed: Feed }) {
  const id = useId();
  const status = feed.last_error ? "error" : feed.last_fetched_at ? "healthy" : "never";
  const time = feed.last_fetched_at ? formatLastFetched(feed.last_fetched_at) : "Not fetched";
  const content = <><span className={`feed-fetch-dot ${status}`} aria-hidden="true" />{time}</>;

  if (!feed.last_error) {
    return <span className="feed-fetch-status settings-table-meta" aria-label={`${status === "healthy" ? "Healthy" : "Not fetched"}: ${time}`}>{content}</span>;
  }

  return <>
    <button type="button" className="feed-fetch-status feed-fetch-error settings-table-meta" popoverTarget={id} aria-haspopup="dialog" aria-label={`Fetch failed for ${feed.title}. Last fetched: ${time}. Show error`}>
      {content}
    </button>
    <div id={id} popover="auto" role="dialog" aria-label={`Fetch error for ${feed.title}`} className="feed-fetch-popover">
      <strong>{feed.title}</strong>
      <p>{feed.last_error}</p>
      <p>Last successful fetch: {feed.last_fetched_at ? new Date(feed.last_fetched_at).toLocaleString() : "Not fetched"}</p>
      <button type="button" className="btn btn-ghost" popoverTarget={id} popoverTargetAction="hide">Close</button>
    </div>
  </>;
}
