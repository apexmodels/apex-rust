use crate::error::WebError;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub title: String,
    pub href: String,
    pub snippet: String,
}

pub struct DuckDuckGoClient {
    client: reqwest::Client,
}

impl Default for DuckDuckGoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DuckDuckGoClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 ApexRust/1.0")
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Search DuckDuckGo HTML endpoint and extract top hits.
    pub async fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchHit>, WebError> {
        let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding(query));
        let response = self.client.get(&url).send().await?;
        let html_body = response.text().await?;

        Ok(Self::parse_html_hits(&html_body, max_results))
    }

    /// Parses search result items from raw HTML.
    pub fn parse_html_hits(html_str: &str, max_results: usize) -> Vec<SearchHit> {
        let document = Html::parse_document(html_str);
        let result_sel = Selector::parse("div.result").unwrap();
        let title_sel = Selector::parse("a.result__a").unwrap();
        let snippet_sel = Selector::parse("a.result__snippet").unwrap();

        let mut hits = Vec::new();

        for element in document.select(&result_sel) {
            if hits.len() >= max_results {
                break;
            }

            let title_elem = element.select(&title_sel).next();
            let snippet_elem = element.select(&snippet_sel).next();

            if let Some(t) = title_elem {
                let title = t.text().collect::<Vec<_>>().join(" ").trim().to_string();
                let mut href = t.value().attr("href").unwrap_or("").to_string();

                // Extract redirect if uddg= parameter is present
                if let Some(pos) = href.find("uddg=") {
                    let after = &href[pos + 5..];
                    let end_pos = after.find('&').unwrap_or(after.len());
                    let encoded = &after[..end_pos];
                    href = urldecoding(encoded);
                }

                let snippet = if let Some(s) = snippet_elem {
                    s.text().collect::<Vec<_>>().join(" ").trim().to_string()
                } else {
                    String::new()
                };

                if !title.is_empty()
                    && (href.starts_with("http://") || href.starts_with("https://"))
                {
                    hits.push(SearchHit {
                        title,
                        href,
                        snippet,
                    });
                }
            }
        }

        hits
    }
}

fn urlencoding(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

fn urldecoding(input: &str) -> String {
    let mut result = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                result.push(val);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}
