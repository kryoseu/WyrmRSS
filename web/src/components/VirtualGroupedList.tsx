import { useMemo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { groupByDate } from "../utils/posts";

type VRow<T> =
  | { kind: "header"; label: string; count: number }
  | { kind: "item"; item: T; isGroupFirst: boolean; isGroupLast: boolean }
  | { kind: "load-more" };

interface Props<T> {
  items: T[] | undefined;
  isLoading: boolean;
  hasNextPage: boolean | undefined;
  fetchNextPage: () => void;
  isFetchingNextPage: boolean;
  renderItem: (item: T) => React.ReactNode;
  // Date used for group headers; must be the field the list is sorted by.
  groupDate: (item: T) => string;
  emptyMessage: string;
}

export function VirtualGroupedList<T>({
  items,
  isLoading,
  hasNextPage,
  fetchNextPage,
  isFetchingNextPage,
  renderItem,
  groupDate,
  emptyMessage,
}: Props<T>) {
  const rows = useMemo<VRow<T>[]>(() => {
    const result: VRow<T>[] = [];
    for (const [label, groupItems] of items ? groupByDate(items, groupDate) : []) {
      result.push({ kind: "header", label, count: groupItems.length });
      for (const [i, item] of groupItems.entries()) {
        result.push({ kind: "item", item, isGroupFirst: i === 0, isGroupLast: i === groupItems.length - 1 });
      }
    }
    if (hasNextPage) result.push({ kind: "load-more" });
    return result;
  }, [items, hasNextPage, groupDate]);

  const scrollRef = useRef<HTMLDivElement>(null);
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 41,
    overscan: 10,
  });

  const isEmpty = !isLoading && rows.length === 0;

  return (
    <div className="pane-scroll" ref={scrollRef}>
      {isLoading && <div className="pane-empty">Loading…</div>}
      {isEmpty && <div className="pane-empty">{emptyMessage}</div>}
      {!isLoading && rows.length > 0 && (
        <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
          {virtualizer.getVirtualItems().map((vItem) => {
            const row = rows[vItem.index];
            return (
              <div
                key={vItem.key}
                data-index={vItem.index}
                ref={virtualizer.measureElement}
                className={
                  row.kind === "item"
                    ? `post-card-item${row.isGroupFirst ? " post-card-item--first" : ""}${row.isGroupLast ? " post-card-item--last" : ""}`
                    : undefined
                }
                style={{ position: "absolute", top: vItem.start, left: 0, right: 0 }}
              >
                {row.kind === "header" && (
                  <div className="posts-group">
                    <div className="posts-group-label">
                      <span>{row.label}</span>
                      <span className="posts-group-count">{row.count}</span>
                    </div>
                  </div>
                )}
                {row.kind === "item" && renderItem(row.item)}
                {row.kind === "load-more" && (
                  <div className="load-more">
                    <button
                      className="load-more-btn"
                      onClick={() => fetchNextPage()}
                      disabled={isFetchingNextPage}
                    >
                      {isFetchingNextPage ? "Loading…" : "Load more"}
                    </button>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
