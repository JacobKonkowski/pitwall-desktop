import type { IracingConfigCheck, ImportStatus } from "../lib/types";



interface Props {

  config: IracingConfigCheck | null;

  importStatus: ImportStatus;

  onStartLive?: () => void;

}



export function ConfigBanner({ config, importStatus, onStartLive }: Props) {

  if (!config) return null;



  const showMemCta = config.memEnabled && onStartLive;



  if (config.warnings.length === 0 && !showMemCta) return null;



  return (

    <div className="banner banner-warn">

      <strong>iRacing setup</strong>

      {config.warnings.length > 0 && (

        <ul>

          {config.warnings.map((w) => (

            <li key={w}>{w}</li>

          ))}

        </ul>

      )}

      {showMemCta && (

        <div className="banner-cta">

          <span>Shared memory telemetry is enabled.</span>

          <button type="button" onClick={onStartLive}>

            Start live monitor

          </button>

        </div>

      )}

      {importStatus.active && (

        <p className="import-active">Importing: {importStatus.currentFile ?? importStatus.message}</p>

      )}

    </div>

  );

}

