interface HlsHost {
  pattern: RegExp;
  playlist: (match: RegExpMatchArray) => string;
  // The post's first <img> is this host's video preview: show it as the
  // player's poster and remove it from the post body.
  posterIsFirstImage?: boolean;
}

// Hosts that link to a video page instead of its HLS playlist.
const HLS_HOSTS: HlsHost[] = [
  {
    pattern: /https:\/\/v\.redd\.it\/([a-z0-9]+)/i,
    playlist: (m) => `https://v.redd.it/${m[1]}/HLSPlaylist.m3u8`,
    posterIsFirstImage: true,
  },
];

// Finds an HLS stream in post HTML: any .m3u8 src/href, else a known video host.
function findHlsSource(html: string): { src: string; posterIsFirstImage: boolean } | null {
  const direct = html.match(/(?:src|href)="(https?:\/\/[^"]+?\.m3u8(?:\?[^"]*)?)"/i);
  if (direct) return { src: direct[1].replaceAll("&amp;", "&"), posterIsFirstImage: false };
  for (const { pattern, playlist, posterIsFirstImage } of HLS_HOSTS) {
    const match = html.match(pattern);
    if (match) return { src: playlist(match), posterIsFirstImage: posterIsFirstImage ?? false };
  }
  return null;
}

export interface HlsVideoMatch {
  src: string;
  poster?: string;
  // Post HTML to render, minus the image used as the poster.
  html: string;
}

// Finds an HLS video in post HTML, moving the host's preview image into the
// poster. DOMParser documents are inert, so nothing loads or runs here.
export function extractHlsVideo(html: string): HlsVideoMatch | null {
  const source = findHlsSource(html);
  if (!source) return null;
  if (!source.posterIsFirstImage) return { src: source.src, html };
  const doc = new DOMParser().parseFromString(html, "text/html");
  const img = doc.querySelector("img");
  img?.remove();
  return { src: source.src, poster: img?.getAttribute("src") ?? undefined, html: doc.body.innerHTML };
}
