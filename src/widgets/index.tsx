import type { LiveSnapshot, WidgetKind } from "../lib/types";
import { CoachWidget } from "./CoachWidget";
import { RadarWidget } from "./RadarWidget";
import { RelativeWidget } from "./RelativeWidget";
import { StandingsWidget } from "./StandingsWidget";
import "./widgets.css";

export { CoachWidget, StandingsWidget, RelativeWidget, RadarWidget };

interface WidgetProps {
  kind: WidgetKind;
  snap: LiveSnapshot;
  fieldPaceMode: string;
}

/** Renders the widget for a kind, shared by the desktop overlay and previews. */
export function Widget({ kind, snap, fieldPaceMode }: WidgetProps) {
  switch (kind) {
    case "standings":
      return <StandingsWidget snap={snap} />;
    case "relative":
      return <RelativeWidget snap={snap} />;
    case "radar":
      return <RadarWidget snap={snap} />;
    case "coach":
    default:
      return <CoachWidget snap={snap} fieldPaceMode={fieldPaceMode} />;
  }
}
