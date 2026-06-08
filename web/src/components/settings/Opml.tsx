import { useState } from "react";
import { useImportOpml } from "../../hooks/useSettings";
import { ENDPOINTS } from "../../utils/api";

export function Opml() {
  const { mutate, isPending, isSuccess, isError } = useImportOpml();
  const [dragging, setDragging] = useState(false);

  function handleFile(file: File) {
    mutate(file);
  }


  function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (file) handleFile(file);
    e.target.value = "";
  }

  function handleDragOver(e: React.DragEvent) {
    e.preventDefault();
    setDragging(true);
  }

  function handleDragLeave() {
    setDragging(false);
  }

  function handleDrop(e: React.DragEvent) {
    e.preventDefault();
    setDragging(false);
    const file = e.dataTransfer.files?.[0];
    if (file) handleFile(file);
  }

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">Import</h2>
      <label
        className={`opml-dropzone${dragging ? " dragging" : ""}`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <span>Drop .opml file here, or click to browse</span>
        <input type="file" accept=".opml,.xml" onChange={handleChange} style={{ display: "none" }} />
      </label>
      <p className="settings-hint">
        Tags are inferred from OPML folders. Feeds with duplicate URLs are skipped.
      </p>
      {isPending && <p className="settings-status">Importing...</p>}
      {isSuccess && <p className="settings-status settings-status-ok">Import complete.</p>}
      {isError && <p className="settings-status settings-status-err">Import failed.</p>}

      <h2 className="settings-section-title">Export</h2>
      <a href={ENDPOINTS.settings.export()} download="wyrm.opml" className="btn btn-ghost settings-export-btn">
        ↓ Download OPML
      </a>
    </div>
  );
}
