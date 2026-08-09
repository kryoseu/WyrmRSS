import { Link } from "react-router";
import { useFeeds, useFeedIcon } from "../hooks/useFeeds";
import { usePosts } from "../hooks/usePosts";
import { useUnreadOnly } from "../hooks/useUnreadOnly";
import { initials } from "../utils/posts";
import { plainTextSnippet } from "../utils/radar";
import type { FeedView } from "../types/FeedView";

// Feeds clearing this bar get a full card; the rest become compact rows.
// Capped so the featured tier can't sprawl even if many feeds qualify.
const FEATURED_MIN_UNREAD = 10;
const FEATURED_MAX_CARDS = 8;

// One card per featured Radar feed. A card's snippet reuses the same
// per-feed post query (and cache key) PostList uses for /feeds/:feedId
// -- unread_only mirrors PostList's own translation of the toggle -- so if
// you've visited that feed recently it renders from cache instead of
// refetching.
function RadarCard({ feed }: { feed: FeedView }) {
  const { unreadOnly } = useUnreadOnly();
  const iconUrl = useFeedIcon({ id: feed.id, name: feed.title, hasIcon: feed.has_icon });
  const { data } = usePosts({
    feed_id: feed.id,
    unread_only: unreadOnly ? undefined : false,
  });
  const latest = data?.pages[0]?.items[0];

  return (
    <Link to={`/feeds/${feed.id}`} className="radar-card">
      <div className="radar-card-head">
        {iconUrl ? (
          <img className="radar-card-icon" src={iconUrl} alt="" loading="lazy" />
        ) : (
          <span className="radar-card-avatar">{initials(feed.title)}</span>
        )}
        <span className="radar-card-title">{feed.title}</span>
        <span className="radar-card-count">
          {feed.unread_count > 999 ? "999+" : feed.unread_count} new
        </span>
      </div>
      {latest && (
        <p className="radar-card-snippet">
          <strong>{latest.title ?? "Untitled"}</strong>
          {latest.description && ` — ${plainTextSnippet(latest.description)}`}
        </p>
      )}
    </Link>
  );
}

// Compact row for everything past the featured tier: icon, title, count.
function RadarRow({ feed }: { feed: FeedView }) {
  const iconUrl = useFeedIcon({ id: feed.id, name: feed.title, hasIcon: feed.has_icon });

  return (
    <Link to={`/feeds/${feed.id}`} className="radar-row">
      {iconUrl ? (
        <img className="radar-row-icon" src={iconUrl} alt="" loading="lazy" />
      ) : (
        <span className="radar-row-avatar">{initials(feed.title)}</span>
      )}
      <span className="radar-row-title">{feed.title}</span>
      <span className="radar-row-count">{feed.unread_count > 999 ? "999+" : feed.unread_count}</span>
    </Link>
  );
}

export function RadarBoard() {
  const { data: feeds, isLoading } = useFeeds();

  const cards = (feeds ?? [])
    .filter((f) => f.display_mode === "radar" && f.unread_count > 0)
    .sort((a, b) => b.unread_count - a.unread_count || a.title.localeCompare(b.title));

  // cards is sorted by unread desc, and the threshold is monotone over that
  // order, so qualifying feeds are always a prefix -- rest is just whatever
  // wasn't featured.
  const qualifying = cards.filter((f) => f.unread_count >= FEATURED_MIN_UNREAD);
  const featured = qualifying.slice(0, FEATURED_MAX_CARDS);
  const rest = cards.slice(featured.length);

  return (
    <div className="pane pane-radar">
      <div className="radar-header">
        <span>Radar</span>
      </div>
      {isLoading && <div className="pane-empty">Loading…</div>}
      {!isLoading && cards.length === 0 && (
        <div className="pane-empty">Nothing on the radar</div>
      )}
      {featured.length > 0 && (
        <>
          {rest.length > 0 && <div className="radar-section-label">Featured</div>}
          <div className="radar-grid">
            {featured.map((feed) => (
              <RadarCard key={feed.id} feed={feed} />
            ))}
          </div>
        </>
      )}
      {rest.length > 0 && (
        <>
          {featured.length > 0 && <div className="radar-section-label">Everything else</div>}
          <div className="radar-list">
            {rest.map((feed) => (
              <RadarRow key={feed.id} feed={feed} />
            ))}
          </div>
        </>
      )}
    </div>
  );
}
