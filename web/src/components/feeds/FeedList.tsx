import { useState } from "react";
import { TbChecks, TbChevronDown, TbChevronRight, TbEdit, TbFolder, TbFolderOpen } from "react-icons/tb";
import { useFeeds } from "../../hooks/useFeeds";
import { useFolders, useUpdateFolder } from "../../hooks/useFolders";
import { useMarkRead } from "../../hooks/usePostMutations";
import { FeedItem } from "./FeedItem";
import { RowMenu } from "../common/RowMenu";
import { Link, useParams } from "react-router";
import { AddFeedForm } from "./AddFeedForm";
import { FolderForm } from "../folders/FolderForm";
import { renameErrorMessage } from "../../api/folders";
import type { FolderId } from "../../types/FolderId";

export function FeedList() {
  const { feedId, folderId } = useParams();
  const { data: feeds, isLoading } = useFeeds();
  const { data: folders } = useFolders();

  const [showAdd, setShowAdd] = useState(false);
  const [search, setSearch] = useState("");
  // Folders default to collapsed; expansion is session-only by design.
  const [expanded, setExpanded] = useState<Set<FolderId>>(new Set());
  // At most one folder renames at a time, so one shared mutation is enough.
  const [renaming, setRenaming] = useState<FolderId | null>(null);
  const update = useUpdateFolder();
  const markRead = useMarkRead();

  // Opens the active feed's folder whenever the active feed changes (e.g.
  // clicking the feed name in PostReader), so it can still be collapsed by hand.
  const activeFolderId = feeds?.find((f) => String(f.id) === feedId)?.folder_id;
  const [revealedFeedId, setRevealedFeedId] = useState<string | undefined>();
  if (feeds && feedId !== revealedFeedId) {
    setRevealedFeedId(feedId);
    if (activeFolderId !== undefined) setExpanded((prev) => new Set(prev).add(activeFolderId));
  }

  // Toggle the folder expansion (e.g. folder left click)
  function toggleExpanded(id: FolderId) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  const searching = search.trim() !== "";
  const filtered = (feeds ?? []).filter((f) =>
    f.title.toLowerCase().includes(search.toLowerCase())
  );

  const standalone = filtered.filter((f) => f.folder_id === undefined);

  // Folder groups in name order; groups whose feeds are all filtered out are
  // hidden. Unread sums cover the full membership, not just search matches.
  const groups = (folders ?? [])
    .map((folder) => ({
      folder,
      members: filtered.filter((f) => f.folder_id === folder.id),
      // Radar feeds don't surface an unread count anywhere in the sidebar.
      unread: (feeds ?? [])
        .filter((f) => f.folder_id === folder.id && f.display_mode !== "radar")
        .reduce((sum, f) => sum + f.unread_count, 0),
    }))
    .filter((g) => g.members.length > 0)
    .sort((a, b) => a.folder.name.localeCompare(b.folder.name));

  const renderFeed = (f: (typeof filtered)[number]) => (
    <FeedItem key={f.id} feed={f} active={feedId === String(f.id)} />
  );

  return (
    <>
      <div className="sidebar-feeds-header">
        <span>Feeds</span>
        <input
          className="sidebar-feeds-search"
          placeholder="Search…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      <div className="sidebar-feeds-scroll">
        {isLoading && <div className="pane-empty">Loading…</div>}
        {standalone.map(renderFeed)}
        {groups.map(({ folder, members, unread }) => {
          // While searching, groups with matches stay open so results show.
          const isOpen = searching || expanded.has(folder.id);
          return (
            <div key={folder.id} className="folder-group">
              <div
                className={`folder-group-header${folderId === String(folder.id) ? " active" : ""}`}
              >
                <button
                  className="folder-group-chevron"
                  onClick={() => toggleExpanded(folder.id)}
                  aria-expanded={isOpen}
                  aria-label={isOpen ? `Collapse ${folder.name}` : `Expand ${folder.name}`}
                >
                  {isOpen ? <TbChevronDown /> : <TbChevronRight />}
                </button>
                <Link to={`/folders/${folder.id}`} className="folder-group-toggle">
                  {isOpen ? <TbFolderOpen /> : <TbFolder />}
                  <span className="folder-group-name">{folder.name}</span>
                  {unread > 0 && (
                    <span className="folder-group-unread" title={`${unread} unread`}>
                      {unread > 999 ? "999+" : unread}
                    </span>
                  )}
                </Link>
                <RowMenu
                  label={`Actions for ${folder.name}`}
                  items={[
                    {
                      icon: <TbChecks />,
                      label: "Mark as read",
                      onSelect: () =>
                        markRead.mutate({ feed_id: null, folder_id: folder.id }),
                    },
                    {
                      icon: <TbEdit />,
                      label: "Rename",
                      onSelect: () => {
                        // Clear any error left over from a previous rename attempt.
                        update.reset();
                        setRenaming((id) => (id === folder.id ? null : folder.id));
                      },
                    },
                  ]}
                />
              </div>
              {renaming === folder.id && (
                <div className="folder-group-rename">
                  <FolderForm
                    initial={folder.name}
                    submitLabel="Save"
                    pendingLabel="Saving…"
                    isPending={update.isPending}
                    error={update.isError ? renameErrorMessage(update.error) : undefined}
                    onSubmit={(name) =>
                      update.mutate(
                        { id: folder.id, body: { name } },
                        { onSuccess: () => setRenaming(null) }
                      )
                    }
                    onCancel={() => setRenaming(null)}
                  />
                </div>
              )}
              {isOpen && <div className="folder-group-items">{members.map(renderFeed)}</div>}
            </div>
          );
        })}
        {filtered.length === 0 && !isLoading && (
          <div className="pane-empty">No feeds</div>
        )}
      </div>

      {showAdd ? (
        <AddFeedForm onClose={() => setShowAdd(false)} />
      ) : (
        <button className="add-feed-btn" onClick={() => setShowAdd(true)}>
          + Add Feed
        </button>
      )}

    </>);
}
