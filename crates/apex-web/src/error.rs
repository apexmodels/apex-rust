use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebError {
    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    InvalidUrl(String),
    #[error("HTML selector error: {0}")]
    Selector(String),
    #[error("Network timeout fetching {0}")]
    Timeout(String),
}
