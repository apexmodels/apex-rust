pub mod error;
pub mod grounding;
pub mod scraper;
pub mod search;

pub use error::WebError;
pub use grounding::{GroundedResearchResult, GroundingEngine};
pub use scraper::PageScraper;
pub use search::{DuckDuckGoClient, SearchHit};
