use serde::{Deserialize, Serialize};

use crate::analysis::coach::CoachReport;
use crate::settings::{load_settings, AppSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoachSummaryResult {
    pub markdown: String,
    pub model: String,
}

pub async fn generate_summary(report: &CoachReport) -> anyhow::Result<CoachSummaryResult> {
    let settings = load_settings();
    let prompt = build_prompt(report);
    let markdown = call_ollama(&settings, &prompt).await?;
    Ok(CoachSummaryResult {
        markdown,
        model: settings.ollama_model.clone(),
    })
}

fn build_prompt(report: &CoachReport) -> String {
    let insights: Vec<String> = report
        .insights
        .iter()
        .map(|i| format!("- {}: {}", i.title, i.detail))
        .collect();
    format!(
        "You are a concise sim racing coach. Based on this session data, give 3-5 actionable bullet points.\n\n\
         Valid laps: {}\n\
         Best lap: {:.3}s\n\
         Avg lap: {:.3}s\n\
         Consistency (std dev): {:.3}s\n\n\
         Insights:\n{}\n\n\
         Keep response under 200 words. Focus on what to fix next session.",
        report.summary.valid_lap_count,
        report.summary.best_lap_ms.unwrap_or(0.0) / 1000.0,
        report.summary.avg_lap_ms.unwrap_or(0.0) / 1000.0,
        report.summary.consistency_ms.unwrap_or(0.0) / 1000.0,
        insights.join("\n")
    )
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

async fn call_ollama(settings: &AppSettings, prompt: &str) -> anyhow::Result<String> {
    let url = format!(
        "{}/api/generate",
        settings.ollama_url.trim_end_matches('/')
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let body = OllamaRequest {
        model: settings.ollama_model.clone(),
        prompt: prompt.to_string(),
        stream: false,
    };
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama returned {status}: {text}");
    }
    let parsed: OllamaResponse = resp.json().await?;
    Ok(parsed.response.trim().to_string())
}
