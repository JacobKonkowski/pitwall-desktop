import type { Feature } from "../features/registry";

interface Props {
  features: Feature[];
  activeId: string;
  onSelect: (id: string) => void;
}

/**
 * Tab bar built from the feature registry. Hidden while only one feature is
 * registered — Analyze is then the whole app. Adding a second feature makes the
 * nav appear with no change to the shell.
 */
export function FeatureNav({ features, activeId, onSelect }: Props) {
  if (features.length < 2) return null;
  return (
    <nav className="feature-nav">
      {features.map((f) => (
        <button
          key={f.id}
          className={f.id === activeId ? "active" : ""}
          onClick={() => onSelect(f.id)}
        >
          {f.label}
        </button>
      ))}
    </nav>
  );
}
