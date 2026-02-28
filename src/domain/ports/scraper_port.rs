use crate::domain::collab::CollabSnapshot;

pub trait ScraperPort {
    fn scrape_collab_html(&self, request: ScrapeRequest) -> Result<CollabSnapshot, ScraperError>;
}

#[derive(Debug, Clone)]
pub struct ScrapeRequest {
    pub moodle_session: String,
    pub html: String,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ScraperError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Unauthorized")]
    Unauthorized,
}
