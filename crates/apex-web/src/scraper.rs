use scraper::{Html, Selector};

pub struct PageScraper;

impl PageScraper {
    /// Extracts clean readable text paragraphs and headers from HTML content,
    /// filtering out navigation, footer, scripts, styles, and empty elements.
    pub fn extract_readable_text(html_content: &str, max_chars: usize) -> String {
        let document = Html::parse_document(html_content);
        
        // Target informative content blocks
        let content_selectors = ["article", "main", "#content", ".content", "body"];
        let mut target_elem = None;

        for sel_str in &content_selectors {
            if let Ok(sel) = Selector::parse(sel_str) {
                if let Some(elem) = document.select(&sel).next() {
                    target_elem = Some(elem);
                    break;
                }
            }
        }

        let p_sel = Selector::parse("p, h1, h2, h3, h4, li, pre code").unwrap();
        let ignore_sel = Selector::parse("script, style, noscript, nav, footer, header, aside, .nav, .footer, .menu").unwrap();

        let mut extracted_parts = Vec::new();
        let mut total_len = 0;

        let root_elem = target_elem.unwrap_or_else(|| document.root_element());

        for el in root_elem.select(&p_sel) {
            // Check if this element is inside an ignored container
            let mut is_ignored = false;
            let mut current = el.parent();
            while let Some(parent) = current {
                if let Some(parent_elem) = scraper::ElementRef::wrap(parent) {
                    if ignore_sel.matches(&parent_elem) {
                        is_ignored = true;
                        break;
                    }
                }
                current = parent.parent();
            }

            if is_ignored {
                continue;
            }

            let text = el.text().collect::<Vec<_>>().join(" ");
            let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");

            if clean.len() >= 25 {
                total_len += clean.len() + 1;
                extracted_parts.push(clean);
                if total_len >= max_chars {
                    break;
                }
            }
        }

        extracted_parts.join("\n\n")
    }

    /// Fetches a URL with a timeout and extracts its readable text.
    pub async fn fetch_and_extract(client: &reqwest::Client, url: &str, max_chars: usize) -> Result<String, crate::error::WebError> {
        let resp = client.get(url).send().await?;
        let body = resp.text().await?;
        Ok(Self::extract_readable_text(&body, max_chars))
    }
}
