/**
 * Shown when the IBT lacks the `LapDeltaTo*_OK` channels, so there is no
 * automatic pace-eligible best. The user must pick a reference lap manually.
 */
export function ConfigBanner() {
  return (
    <div className="banner">
      <span>ⓘ</span>
      <div>
        <div className="banner-title">No pace validity in this file</div>
        <div>
          This IBT does not contain iRacing's <code>LapDeltaToBestLap_OK</code>{" "}
          channels, so no automatic best lap can be chosen. Pick a reference lap
          manually in the compare panel below.
        </div>
      </div>
    </div>
  );
}
