import { useState, type ReactNode } from "react";

interface Props {
  url?: string;
  /** Shown while the image loads, and instead of it when missing or broken. */
  fallback: ReactNode;
  /** Small corner badge over a loaded image, e.g. the feed icon. */
  badge?: ReactNode;
}

/** Card-mode thumbnail tile. Key it by `url` so a new image resets its state. */
export function PostThumbnail({ url, fallback, badge }: Props) {
  const [loaded, setLoaded] = useState(false);
  const [failed, setFailed] = useState(false);

  return (
    <span className="post-item-thumb">
      {fallback}
      {url && !failed && (
        <img
          className={`post-item-thumb-image${loaded ? " loaded" : ""}`}
          src={url}
          alt=""
          loading="lazy"
          referrerPolicy="no-referrer"
          onLoad={() => setLoaded(true)}
          onError={() => setFailed(true)}
        />
      )}
      {loaded && !failed && badge && <span className="post-item-thumb-badge">{badge}</span>}
    </span>
  );
}
