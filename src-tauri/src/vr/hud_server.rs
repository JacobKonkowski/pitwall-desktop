//! Local HTTP HUD for in-headset use via OpenKneeboard (or any Web Dashboard tab).
//! Works with iRacing OpenXR — no SteamVR required.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::live::LiveService;
use crate::vr::VrOverlayService;

pub const HUD_PORT: u16 = 17342;

pub fn hud_url() -> String {
    format!("http://127.0.0.1:{HUD_PORT}/vr")
}

pub fn run_hud_server(
    service: Arc<VrOverlayService>,
    live: Arc<LiveService>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{HUD_PORT}");
    let listener = TcpListener::bind(&addr)?;
    service.status.lock().message = format!("HUD ready at {}", hud_url());
    service.status.lock().runtime = "OpenXR (Web HUD)".into();
    service.status.lock().active = true;

    listener.set_nonblocking(true)?;

    while !cancel.is_cancelled() {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let live = Arc::clone(&live);
                thread::spawn(move || {
                    let _ = handle_connection(&mut stream, &live);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(e.into()),
        }
    }

    service.status.lock().active = false;
    service.status.lock().message = "In-headset HUD stopped".into();
    Ok(())
}

fn handle_connection(stream: &mut std::net::TcpStream, live: &LiveService) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let (status, content_type, body) = match path {
        "/api/live" => {
            let snap = live.snapshot.lock().clone();
            let json = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".into());
            ("200 OK", "application/json", json)
        }
        "/vr" | "/" => ("200 OK", "text/html; charset=utf-8", VR_HUD_HTML.to_string()),
        _ => (
            "404 Not Found",
            "text/plain",
            "Not found".to_string(),
        ),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

const VR_HUD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>PitWall VR HUD</title>
  <style>
    * { box-sizing: border-box; margin: 0; }
    body {
      font-family: "Segoe UI", system-ui, sans-serif;
      background: rgba(12, 16, 24, 0.92);
      color: #e8eef8;
      padding: 12px 14px;
      min-width: 280px;
    }
    .track { font-size: 13px; font-weight: 600; margin-bottom: 4px; }
    .lap { font-size: 22px; font-weight: 700; margin: 4px 0; }
    .delta { font-size: 14px; margin-bottom: 6px; }
    .delta.slow { color: #ef5350; }
    .delta.fast { color: #66bb6a; }
    .fuel { color: #aab4c4; font-size: 13px; margin-bottom: 8px; }
    .sectors { display: flex; flex-direction: column; gap: 4px; }
    .sector { display: grid; grid-template-columns: 24px 1fr; gap: 6px; align-items: center; font-size: 12px; }
    .bar { height: 6px; background: rgba(255,255,255,0.12); border-radius: 3px; overflow: hidden; }
    .fill { height: 100%; background: #4fc3f7; border-radius: 3px; transition: width 0.1s linear; }
    .wait { color: #888; font-size: 14px; }
  </style>
</head>
<body>
  <div id="root"><p class="wait">Waiting for PitWall live telemetry…</p></div>
  <script>
    function fmt(ms) {
      if (ms == null || ms <= 0) return "—";
      const s = ms / 1000;
      const m = Math.floor(s / 60);
      const sec = s - m * 60;
      return m > 0 ? m + ":" + sec.toFixed(3).padStart(6, "0") : sec.toFixed(3);
    }
    function fmtDelta(ms) {
      if (ms == null) return "—";
      return (ms >= 0 ? "+" : "") + (ms / 1000).toFixed(3);
    }
    function render(s) {
      if (!s || !s.track) {
        document.getElementById("root").innerHTML = '<p class="wait">Waiting for iRacing session…</p>';
        return;
      }
      const deltaClass = s.deltaToBestMs != null && s.deltaToBestMs > 0 ? "slow" : "fast";
      const sectors = [1,2,3].map(n => {
        const sec = (s.sectors || []).find(x => x.sectorNum === n);
        const done = sec && sec.completed;
        const active = s.currentSector === n;
        const pct = done ? 100 : active ? Math.min(100, (s.lapDistPct || 0) * 100) : 0;
        return `<div class="sector"><span>S${n}</span><div class="bar"><div class="fill" style="width:${pct}%"></div></div></div>`;
      }).join("");
      document.getElementById("root").innerHTML = `
        <div class="track">${s.track} · ${s.sessionType || ""}</div>
        <div class="lap">Lap ${s.lap} · ${fmt(s.lapTimeMs)}</div>
        <div class="delta ${deltaClass}">Δ best ${fmtDelta(s.deltaToBestMs)}</div>
        <div class="fuel">Fuel ${(s.fuelLevel || 0).toFixed(1)} L · ${(s.speed || 0).toFixed(0)} km/h</div>
        <div class="sectors">${sectors}</div>`;
    }
    async function poll() {
      try {
        const r = await fetch("/api/live");
        render(await r.json());
      } catch (_) {}
    }
    poll();
    setInterval(poll, 100);
  </script>
</body>
</html>
"#;
