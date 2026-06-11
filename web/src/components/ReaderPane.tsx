import { TbX } from "react-icons/tb";

interface Props {
  width: number;
  onClose: () => void;
  actions?: React.ReactNode;
  isLoading: boolean;
  notFound: boolean;
  notFoundMessage?: string;
  children: React.ReactNode;
}

export function ReaderPane({ width, onClose, actions, isLoading, notFound, notFoundMessage = "Not found", children }: Props) {
  return (
    <div className="pane pane-reader" style={{ width }}>
      <div className="pane-reader-header">
        {actions}
        <button className="pane-reader-close" onClick={onClose} aria-label="Close reader">
          <TbX />
        </button>
      </div>
      <div className="pane-scroll">
        {isLoading && <div className="pane-empty">Loading…</div>}
        {!isLoading && notFound && <div className="pane-empty">{notFoundMessage}</div>}
        {!isLoading && !notFound && children}
      </div>
    </div>
  );
}
