use crate::domain::collab::CollabSnapshot;

pub trait ScraperPort {
    fn scrape_collab_html(&self, request: ScrapeRequest) -> Result<CollabSnapshot, ScraperError>;
}

#[derive(Debug, Clone)]
pub struct ScrapeRequest {
    pub moodle_session: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScraperError {
    InvalidInput(String),
    ParseError(String),
    UnsupportedFormat(String),
    Unauthorized,
}

impl std::fmt::Display for ScraperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            Self::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl std::error::Error for ScraperError {}
