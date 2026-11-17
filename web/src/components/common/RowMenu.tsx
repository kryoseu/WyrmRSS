import { useRef, useState } from "react";
import { TbDotsVertical } from "react-icons/tb";
import { useDismiss } from "../../hooks/useDismiss";

export interface RowMenuItem {
  icon: React.ReactNode;
  label: string;
  /** When set, show confirmation prompt. */
  confirmLabel?: string;
  /** Styles the item as destructive (red hover). */
  danger?: boolean;
  disabled?: boolean;
  onSelect: () => void;
}

interface Props {
  label: string;
  items: RowMenuItem[];
}

/**
 * Kebab menu for feeds and folders. The trigger is displayed on hover.
 */
export function RowMenu({ label, items }: Props) {
  const [open, setOpen] = useState(false);
  const [confirming, setConfirming] = useState<number | null>(null);
  const ref = useRef<HTMLDivElement>(null);

  useDismiss(ref, open, () => {
    setOpen(false);
    setConfirming(null);
  });

  function handleTrigger(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    setOpen(!open);
    setConfirming(null);
  }

  function handleItem(e: React.MouseEvent, index: number) {
    e.preventDefault();
    e.stopPropagation();
    const item = items[index];
    if (item.confirmLabel && confirming !== index) {
      setConfirming(index);
      return;
    }
    setOpen(false);
    setConfirming(null);
    item.onSelect();
  }

  return (
    <div className="row-menu" ref={ref}>
      <button
        className={`row-menu-trigger${open ? " open" : ""}`}
        onClick={handleTrigger}
        title={label}
        aria-label={label}
        aria-haspopup="menu"
        aria-expanded={open}
      >
        <TbDotsVertical />
      </button>
      {open && (
        <div className="row-menu-dropdown" role="menu">
          {items.map((item, i) => (
            <button
              key={item.label}
              role="menuitem"
              className={`row-menu-item${item.danger ? " danger" : ""}`}
              disabled={item.disabled}
              onClick={(e) => handleItem(e, i)}
            >
              {item.icon}
              <span>
                {confirming === i && item.confirmLabel ? item.confirmLabel : item.label}
              </span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
