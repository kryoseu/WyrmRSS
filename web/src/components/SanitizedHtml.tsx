import { useMemo } from "react";
import DOMPurify from "dompurify";

interface Props {
  html: string;
  className?: string;
}

// Feed content is untrusted, so sanitize it before injecting. This is the only
// place in the app allowed to use dangerouslySetInnerHTML.
export function SanitizedHtml({ html, className }: Props) {
  const clean = useMemo(() => DOMPurify.sanitize(html), [html]);
  return <div className={className} dangerouslySetInnerHTML={{ __html: clean }} />;
}
