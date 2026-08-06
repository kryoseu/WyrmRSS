import { useMemo } from "react";
import { isTauri } from "@tauri-apps/api/core";
import DOMPurify from "dompurify";

interface Props {
  html: string;
  className?: string;
}

// Feed content is untrusted, so sanitize it before injecting. This is the only
// place in the app allowed to use dangerouslySetInnerHTML.
export function SanitizedHtml({ html, className }: Props) {
  const clean = useMemo(
    // target="_blank" (feed content may already carry it) bypasses the
    // desktop link handler; strip it there.
    () => DOMPurify.sanitize(html, isTauri() ? { FORBID_ATTR: ["target"] } : undefined),
    [html],
  );
  return <div className={className} dangerouslySetInnerHTML={{ __html: clean }} />;
}
