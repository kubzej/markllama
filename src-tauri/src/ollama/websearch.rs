use serde::{Deserialize, Serialize};
use std::time::Duration;

const WEB_SEARCH_URL: &str = "https://ollama.com/api/web_search";
/// Keeps each result's contribution to the prompt small — inference stays local and fast, the
/// web results are meant as reference snippets, not full page dumps.
const MAX_CONTENT_CHARS: usize = 500;
/// A real network round-trip to an external service — more generous than the local status
/// calls, but still bounded so a slow/unreachable ollama.com can't hang generation forever.
const WEB_SEARCH_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Serialize)]
struct WebSearchRequest<'a> {
    query: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct WebSearchResult {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct WebSearchResponse {
    #[serde(default)]
    results: Vec<WebSearchResult>,
}

pub async fn web_search(
    http: &reqwest::Client,
    api_key: &str,
    query: &str,
) -> Result<Vec<WebSearchResult>, String> {
    let response = http
        .post(WEB_SEARCH_URL)
        .bearer_auth(api_key)
        .json(&WebSearchRequest { query })
        .timeout(WEB_SEARCH_TIMEOUT)
        .send()
        .await
        .map_err(|err| format!("Could not reach Ollama web search: {err}"))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Ollama rejected the web search API key — check it in Settings.".to_string());
    }
    if !response.status().is_success() {
        return Err(format!("Ollama web search returned {}", response.status()));
    }

    let parsed: WebSearchResponse = response
        .json()
        .await
        .map_err(|err| format!("Unexpected response from Ollama web search: {err}"))?;

    Ok(parsed.results)
}

/// Renders results as a compact block for the prompt's user message — title, URL, and a
/// truncated content snippet per result.
pub fn format_results_for_prompt(results: &[WebSearchResult]) -> String {
    results
        .iter()
        .map(|result| {
            let snippet: String = result.content.chars().take(MAX_CONTENT_CHARS).collect();
            format!("- {} ({})\n  {snippet}", result.title, result.url)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_content_and_keeps_title_and_url() {
        let results = vec![WebSearchResult {
            title: "Docs".to_string(),
            url: "https://docs.test".to_string(),
            content: "z".repeat(1000),
        }];

        let formatted = format_results_for_prompt(&results);
        assert!(formatted.starts_with("- Docs (https://docs.test)"));
        assert_eq!(formatted.matches('z').count(), MAX_CONTENT_CHARS);
    }
}
