import { useEffect, useState } from "react";
import {
  getImportStatus,
  importFolder,
  importIbt,
  onImportStatus,
  pickIbtFile,
} from "../../shared/api";
import { showToast } from "../../shared/toast";

/** Shared Import / Scan actions for the Analyze header and empty state. */
export function useImportActions() {
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    getImportStatus()
      .then((s) => setBusy(s.active))
      .catch(() => undefined);
    const unlisten = onImportStatus((s) => setBusy(s.active));
    return () => {
      unlisten.then((fn) => fn()).catch(() => undefined);
    };
  }, []);

  async function handleImport() {
    try {
      const path = await pickIbtFile();
      if (path) await importIbt(path);
    } catch (e) {
      console.error("Import failed", e);
      showToast(`Import failed: ${String(e)}`, "error");
    }
  }

  async function handleScan() {
    try {
      await importFolder();
    } catch (e) {
      console.error("Scan failed", e);
      showToast(`Scan failed: ${String(e)}`, "error");
    }
  }

  return { busy, handleImport, handleScan };
}
