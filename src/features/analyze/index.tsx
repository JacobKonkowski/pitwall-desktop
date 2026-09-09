import type { Feature } from "../registry";
import { AnalyzePage } from "./AnalyzePage";
import { useImportActions } from "./useImportActions";

/** Import / scan actions, rendered into the shell header slot. */
function AnalyzeHeaderActions() {
  const { busy, handleImport, handleScan } = useImportActions();

  return (
    <>
      <button className="btn" onClick={handleScan} disabled={busy}>
        Scan folder
      </button>
      <button className="btn btn-primary" onClick={handleImport} disabled={busy}>
        Import IBT
      </button>
    </>
  );
}

export const analyzeFeature: Feature = {
  id: "analyze",
  label: "Analyze",
  path: "/analyze",
  element: <AnalyzePage />,
  HeaderActions: AnalyzeHeaderActions,
};
