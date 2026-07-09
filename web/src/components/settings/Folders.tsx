import { useState } from "react";
import { TbEdit, TbPlus, TbTrash } from "react-icons/tb";
import { useCreateFolder, useDeleteFolder, useFolders, useUpdateFolder } from "../../hooks/useFolders";
import { useFeeds } from "../../hooks/useFeeds";
import { FolderForm } from "../FolderForm";
import { renameErrorMessage } from "../../api/folders";
import type { Folder } from "../../types/Folder";
import type { FolderId } from "../../types/FolderId";

function FolderRow({ folder, feedCount }: { folder: Folder; feedCount: number }) {
  const [editing, setEditing] = useState(false);
  const update = useUpdateFolder();
  const { mutate: deleteFolder } = useDeleteFolder();

  return (
    <>
      <div className={`settings-table-row${editing ? " editing" : ""}`}>
        <span className="settings-table-title">{folder.name}</span>
        <span className="settings-table-meta">
          {feedCount === 1 ? "1 feed" : `${feedCount} feeds`}
        </span>
        <div className="settings-table-actions">
          <button
            className={`settings-table-btn${editing ? " active" : ""}`}
            onClick={() => {
              update.reset();
              setEditing((e) => !e);
            }}
            title="Rename"
          >
            <TbEdit />
          </button>
          <button
            className="settings-table-btn settings-table-btn-delete"
            onClick={() => deleteFolder(folder.id)}
            title="Delete (feeds become standalone)"
          >
            <TbTrash />
          </button>
        </div>
      </div>
      {editing && (
        <div className="settings-table-edit">
          <FolderForm
            initial={folder.name}
            submitLabel="Save"
            pendingLabel="Saving…"
            isPending={update.isPending}
            error={update.isError ? renameErrorMessage(update.error) : undefined}
            onSubmit={(name) =>
              update.mutate(
                { id: folder.id, body: { name } },
                { onSuccess: () => setEditing(false) }
              )
            }
            onCancel={() => setEditing(false)}
          />
        </div>
      )}
    </>
  );
}

export function Folders() {
  const { data: folders, isLoading } = useFolders();
  const { data: feeds } = useFeeds();
  const create = useCreateFolder();
  const [search, setSearch] = useState("");
  const [adding, setAdding] = useState(false);

  if (isLoading) return <div className="pane-empty">Loading…</div>;

  const feedCounts = new Map<FolderId, number>();
  for (const f of feeds ?? []) {
    if (f.folder_id !== undefined) {
      feedCounts.set(f.folder_id, (feedCounts.get(f.folder_id) ?? 0) + 1);
    }
  }

  const filtered = (folders ?? [])
    .filter((f) => f.name.toLowerCase().includes(search.toLowerCase()))
    .sort((a, b) => a.name.localeCompare(b.name));

  return (
    <div className="settings-table-wrap">
      <div className="settings-table-toolbar">
        <input
          className="settings-table-search"
          placeholder="Search folders…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <button
          className={`btn ${adding ? "btn-ghost" : "btn-primary"} settings-table-add-btn`}
          onClick={() => setAdding((a) => !a)}
        >
          <TbPlus /> {adding ? "Cancel" : "Add folder"}
        </button>
      </div>
      {adding && (
        <div className="settings-table-edit">
          <FolderForm
            submitLabel="Add"
            pendingLabel="Adding…"
            isPending={create.isPending}
            // POST resolves-or-creates: a case-variant of an existing folder
            // returns the existing row, so this never errors on duplicates.
            onSubmit={(name) =>
              create.mutate({ name }, { onSuccess: () => setAdding(false) })
            }
            onCancel={() => setAdding(false)}
          />
        </div>
      )}
      {!folders?.length ? (
        <div className="settings-empty">No folders yet.</div>
      ) : (
        <div className="settings-table folders-table">
          <div className="settings-table-header">
            <span className="settings-table-title">Name</span>
            <span className="settings-table-meta">Feeds</span>
            <span className="settings-table-actions" />
          </div>
          {filtered.map((folder) => (
            <FolderRow
              key={folder.id}
              folder={folder}
              feedCount={feedCounts.get(folder.id) ?? 0}
            />
          ))}
          {filtered.length === 0 && <div className="pane-empty">No folders match.</div>}
        </div>
      )}
    </div>
  );
}
