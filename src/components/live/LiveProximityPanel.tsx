import type { LiveSnapshot } from "../../lib/types";
import { RadarWidget } from "../../widgets/RadarWidget";
import { RelativeWidget } from "../../widgets/RelativeWidget";
import "../../widgets/widgets.css";

interface Props {
  snap: LiveSnapshot;
}

export function LiveProximityPanel({ snap }: Props) {
  return (
    <div className="panel live-proximity">
      <h3>Proximity</h3>
      <div className="live-proximity-grid">
        <div className="live-proximity-widget">
          <h4>Radar (±3s)</h4>
          <div className="live-widget-frame">
            <RadarWidget snap={snap} />
          </div>
        </div>
        <div className="live-proximity-widget">
          <h4>Relative (±8s)</h4>
          <div className="live-widget-frame live-widget-relative">
            <RelativeWidget snap={snap} />
          </div>
        </div>
      </div>
    </div>
  );
}
