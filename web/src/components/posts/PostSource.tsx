import { useRef, useState } from "react";
import { TbCode, TbX } from "react-icons/tb";
import hljs from "highlight.js/lib/core";
import xml from "highlight.js/lib/languages/xml";
import { html as beautifyHtml } from "js-beautify";
import { SanitizedHtml } from "../common/SanitizedHtml";

hljs.registerLanguage("xml", xml);


interface Props {
  html: string;
}

/** The post's stored HTML as text, before sanitizing. */
export function PostSource({ html }: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [highlighted, setHighlighted] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  // Format on open, not on every post the reader shows.
  function show() {
    const formatted = beautifyHtml(html, {
      indent_size: 2,
      wrap_line_length: 100,
      wrap_attributes: "force-expand-multiline",
    });

    setHighlighted(hljs.highlight(formatted, { language: "xml" }).value);
    dialogRef.current?.showModal();
    dialogRef.current?.focus();
  }

  function copy() {
    navigator.clipboard.writeText(html).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }

  // Clicks outside the box land on the dialog element itself.
  function closeOnBackdropClick(e: React.MouseEvent<HTMLDialogElement>) {
    if (e.target === e.currentTarget) e.currentTarget.close();
  }

  return (
    <>
      <button className="pane-reader-source" onClick={show} aria-label="View source" title="View source">
        <TbCode />
      </button>
      <dialog className="shortcuts-help post-source" ref={dialogRef} tabIndex={-1} onClick={closeOnBackdropClick}>
        <div className="shortcuts-help-content">
          <div className="shortcuts-help-header">
            <h2 className="settings-section-title">Source</h2>
            <div className="post-source-actions">
              <button className="segmented-btn" onClick={copy}>{copied ? "Copied" : "Copy"}</button>
              <button className="pane-reader-close" onClick={() => dialogRef.current?.close()} aria-label="Close">
                <TbX />
              </button>
            </div>
          </div>
          {highlighted !== null && <SanitizedHtml className="post-source-text" html={highlighted} />}
        </div>
      </dialog>
    </>
  );
}
