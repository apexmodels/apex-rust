use crate::error::WebError;
use crate::scraper::PageScraper;
use crate::search::DuckDuckGoClient;
use apex_core::models::WebSource;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedResearchResult {
    pub sources: Vec<WebSource>,
    pub context_prompt: String,
}

pub struct GroundingEngine {
    search_client: DuckDuckGoClient,
    http_client: reqwest::Client,
}

impl Default for GroundingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GroundingEngine {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 ApexRust/1.0")
            .timeout(std::time::Duration::from_secs(6))
            .build()
            .unwrap_or_default();

        Self {
            search_client: DuckDuckGoClient::new(),
            http_client,
        }
    }

    /// Performs search and concurrent page scraping, assembling formatted grounding context.
    pub async fn research(
        &self,
        query: &str,
        max_sources: usize,
    ) -> Result<GroundedResearchResult, WebError> {
        let hits = self.search_client.search(query, max_sources).await?;
        if hits.is_empty() {
            return Ok(GroundedResearchResult {
                sources: Vec::new(),
                context_prompt: String::new(),
            });
        }

        let mut sources = Vec::new();
        let mut join_set = JoinSet::new();

        for (idx, hit) in hits.into_iter().enumerate() {
            let client = self.http_client.clone();
            let source_id = idx + 1;
            join_set.spawn(async move {
                let scraped = PageScraper::fetch_and_extract(&client, &hit.href, 1500).await;
                let text = match scraped {
                    Ok(t) if !t.is_empty() => t,
                    _ => hit.snippet.clone(),
                };
                (source_id, hit, text)
            });
        }

        let mut gathered = Vec::new();
        while let Some(res) = join_set.join_next().await {
            if let Ok(item) = res {
                gathered.push(item);
            }
        }
        gathered.sort_by_key(|(id, _, _)| *id);

        let mut context_builder =
            String::from("Here is live web search context to assist your response:\n\n");

        for (id, hit, text) in gathered {
            sources.push(WebSource {
                id,
                title: hit.title.clone(),
                url: hit.href.clone(),
                snippet: if text.len() > 300 {
                    format!("{}...", &text[..300])
                } else {
                    text.clone()
                },
            });

            context_builder.push_str(&format!(
                "[Source {}]: {}\nURL: {}\nContent:\n{}\n\n",
                id, hit.title, hit.href, text
            ));
        }

        Ok(GroundedResearchResult {
            sources,
            context_prompt: context_builder,
        })
    }
}
