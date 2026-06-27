import { useEffect, useState } from "react";
import { generateCoachSummary, getCoachReport } from "../lib/api";
import type { CoachInsight, CoachReport } from "../lib/types";

interface Props {
  sessionId: number;
  highlightedLaps: number[];
  onHighlightLaps: (lapNumbers: number[]) => void;
  onCompareLaps?: (lapNumbers: number[]) => void;
}

function severityClass(severity: string): string {
  switch (severity) {
    case "warn":
      return "coach-card coach-warn";
    case "good":
      return "coach-card coach-good";
    default:
      return "coach-card";
  }
}

const KIND_META: Record<string, { icon: string; label: string }> = {
  early_lift: { icon: "🔻", label: "Early lift" },
  late_brake: { icon: "🛑", label: "Late braking" },
  high_steering: { icon: "↩️", label: "Steering" },
  sector_weakness: { icon: "⏱️", label: "Sector" },
  consistency: { icon: "📊", label: "Consistency" },
  fuel: { icon: "⛽", label: "Fuel" },
  session_pace: { icon: "🏁", label: "Field pace" },
  traffic_pace: { icon: "🚗", label: "Traffic" },
};

function kindMeta(kind: string): { icon: string; label: string } {
  return KIND_META[kind] ?? { icon: "💡", label: "Tip" };
}

export function CoachPanel({
  sessionId,
  highlightedLaps,
  onHighlightLaps,
  onCompareLaps,
}: Props) {
  const [report, setReport] = useState<CoachReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [summary, setSummary] = useState<string | null>(null);
  const [summaryModel, setSummaryModel] = useState<string | null>(null);
  const [aiLoading, setAiLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    setError(null);
    setSummary(null);
    getCoachReport(sessionId)
      .then(setReport)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [sessionId]);

  const handleInsightClick = (insight: CoachInsight) => {
    if (insight.lapNumbers.length > 0) {
      onHighlightLaps(insight.lapNumbers);
    }
  };

  const handleGenerateSummary = async () => {
    setAiLoading(true);
    setError(null);
    try {
      const result = await generateCoachSummary(sessionId);
      setSummary(result.markdown);
      setSummaryModel(result.model);
    } catch (e) {
      setError(String(e));
    } finally {
      setAiLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="panel coach-panel">
        <p className="muted">Analyzing session…</p>
      </div>
    );
  }

  if (!report) {
    return (
      <div className="panel coach-panel">
        <p className="muted">{error ?? "No coaching data available."}</p>
      </div>
    );
  }

  return (
    <div className="panel coach-panel">
      <div className="panel-header">
        <h2>Coach</h2>
        <button onClick={handleGenerateSummary} disabled={aiLoading}>
          {aiLoading ? "Generating…" : "Generate AI summary"}
        </button>
      </div>

      <div className="coach-stats">
        <span>{report.summary.validLapCount} valid laps</span>
        {report.summary.bestLapMs != null && (
          <span>Best {formatSec(report.summary.bestLapMs)}</span>
        )}
        {report.summary.consistencyMs != null && (
          <span>σ {formatSec(report.summary.consistencyMs)}</span>
        )}
        {report.summary.weakestSector != null && (
          <span>
            Weakest S{report.summary.weakestSector}
            {report.summary.weakestSectorLossMs != null &&
              ` (+${formatSec(report.summary.weakestSectorLossMs)} avg)`}
          </span>
        )}
      </div>

      {error && <p className="live-error">{error}</p>}

      {summary && (
        <div className="coach-summary">
          <h3>AI summary {summaryModel ? `(${summaryModel})` : ""}</h3>
          <div
            className="coach-summary-md"
            dangerouslySetInnerHTML={{ __html: formatMarkdown(summary) }}
          />
        </div>
      )}

      {report.insights.length === 0 ? (
        <p className="muted">No insights yet — need more valid laps.</p>
      ) : (
        <div className="coach-cards">
          {report.insights.map((insight) => {
            const active =
              insight.lapNumbers.length > 0 &&
              insight.lapNumbers.every((n) => highlightedLaps.includes(n));
            const meta = kindMeta(insight.kind);
            return (
              <button
                key={`${insight.kind}-${insight.title}`}
                type="button"
                className={`${severityClass(insight.severity)}${active ? " coach-card-active" : ""}`}
                onClick={() => handleInsightClick(insight)}
              >
                <strong>
                  <span className="coach-card-icon" aria-hidden="true">
                    {meta.icon}
                  </span>
                  {insight.title}
                </strong>
                <p>{insight.detail}</p>
                {insight.lapNumbers.length > 0 && (
                  <span className="coach-card-actions">
                    <span className="muted small">Laps: {insight.lapNumbers.join(", ")}</span>
                    {onCompareLaps && (
                      <span
                        className="coach-compare-link"
                        role="button"
                        tabIndex={0}
                        onClick={(e) => {
                          e.stopPropagation();
                          onCompareLaps(insight.lapNumbers);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            e.stopPropagation();
                            onCompareLaps(insight.lapNumbers);
                          }
                        }}
                      >
                        Compare laps
                      </span>
                    )}
                  </span>
                )}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

function formatSec(ms: number): string {
  return `${(ms / 1000).toFixed(3)}s`;
}

function formatMarkdown(md: string): string {
  return md
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/^### (.+)$/gm, "<h4>$1</h4>")
    .replace(/^## (.+)$/gm, "<h3>$1</h3>")
    .replace(/^# (.+)$/gm, "<h2>$1</h2>")
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/^- (.+)$/gm, "<li>$1</li>")
    .replace(/(<li>.*<\/li>\n?)+/g, (m) => `<ul>${m}</ul>`)
    .replace(/\n\n/g, "</p><p>")
    .replace(/^(.+)$/gm, (line) =>
      line.startsWith("<h") || line.startsWith("<ul") || line.startsWith("<li") ? line : `<p>${line}</p>`,
    );
}
