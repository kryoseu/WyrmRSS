import { useEffect, useRef } from "react";
import type Hls from "hls.js";

interface Props {
  src: string;
  poster?: string;
}

// Prefers hls.js, since native HLS on Linux WebKitGTK is unreliable.
export function HlsVideo({ src, poster }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    let hls: Hls | undefined;
    let cancelled = false;
    import("hls.js").then(({ default: HlsPlayer }) => {
      if (cancelled) return;
      if (HlsPlayer.isSupported()) {
        hls = new HlsPlayer({ capLevelToPlayerSize: true });
        hls.loadSource(src);
        hls.attachMedia(video);
      } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
        video.src = src;
      }
    });
    return () => {
      cancelled = true;
      hls?.destroy();
    };
  }, [src]);

  return <video ref={videoRef} className="post-reader-video" controls playsInline preload="metadata" poster={poster} />;
}
