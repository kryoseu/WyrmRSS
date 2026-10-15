import { TbChevronLeft, TbX } from "react-icons/tb";
import { useIsMobile } from "../../hooks/useIsMobile";

interface Props {
  /** Omitted on phones, where the reader covers the whole screen. */
  width?: number;
  onClose: () => void;
  actions?: React.ReactNode;
  isLoading: boolean;
  notFound: boolean;
  notFoundMessage?: string;
  children: React.ReactNode;
}

export function ReaderPane({ width, onClose, actions, isLoading, notFound, notFoundMessage = "Not found", children }: Props) {
  const isMobile = useIsMobile();

  const content = (
    <div className="pane-scroll">
      {isLoading && <div className="pane-empty">Loading…</div>}
      {!isLoading && notFound && <div className="pane-empty">{notFoundMessage}</div>}
      {!isLoading && !notFound && children}
    </div>
  );

  // Phones get a back button on top and the actions in a bottom bar, within
  // thumb reach.
  if (isMobile) {
    return (
      <div className="pane pane-reader">
        <div className="pane-reader-topbar">
          <button className="pane-reader-back" onClick={onClose}>
            <TbChevronLeft />
            <span>Back</span>
          </button>
        </div>
        {content}
        {actions && <div className="pane-reader-actions">{actions}</div>}
      </div>
    );
  }

  return (
    <div className="pane pane-reader" style={{ width }}>
      <div className="pane-reader-header">
        {actions}
        <button className="pane-reader-close" onClick={onClose} aria-label="Close reader">
          <TbX />
        </button>
      </div>
      {content}
    </div>
  );
}
