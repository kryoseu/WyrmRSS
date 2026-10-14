import { useMemo } from "react";
import { isTauri } from "@tauri-apps/api/core";
import DOMPurify from "dompurify";

interface Props {
  html: string;
  className?: string;
}

// Some feeds (e.g. WordPress sites like nasa.gov) ship their page chrome:
// icon SVGs, breadcrumbs, menu buttons. Drop those along with their contents.
const PAGE_CHROME = ["svg", "nav", "button", "form", "input"];

// Feed content is untrusted, so sanitize it before injecting. This is the only
// place in the app allowed to use dangerouslySetInnerHTML.
export function SanitizedHtml({ html, className }: Props) {
  const clean = useMemo(
    () =>
      DOMPurify.sanitize(html, {
        FORBID_TAGS: PAGE_CHROME,
        ADD_FORBID_CONTENTS: PAGE_CHROME,
        // Inline styles assume the source site's layout. target="_blank"
        // bypasses the desktop link handler.
        FORBID_ATTR: isTauri() ? ["style", "target"] : ["style"],
      }),
    [html],
  );
  return <div className={className} dangerouslySetInnerHTML={{ __html: clean }} />;
}
