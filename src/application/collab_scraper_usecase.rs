use crate::domain::collab::CollabSnapshot;
use crate::domain::ports::scraper_port::{ScrapeRequest, ScraperError, ScraperPort};

pub struct CollabScraperUseCase<T: ScraperPort> {
    scraper_port: T,
}

impl<T: ScraperPort> CollabScraperUseCase<T> {
    pub fn new(scraper_port: T) -> Self {
        Self { scraper_port }
    }

    pub fn scrape(&self, moodle_session: &str, html: &str) -> Result<CollabSnapshot, ScraperError> {
        if moodle_session.trim().is_empty() {
            return Err(ScraperError::Unauthorized);
        }

        if html.trim().len() < 20 {
            return Err(ScraperError::InvalidInput(
                "HTML payload is empty or too short".to_string(),
            ));
        }

        self.scraper_port.scrape_collab_html(ScrapeRequest {
            moodle_session: moodle_session.to_string(),
            html: html.to_string(),
        })
    }
}

// Support for Box<dyn ScraperPort> - enables runtime polymorphic adapter selection
impl CollabScraperUseCase<Box<dyn ScraperPort>> {
    pub fn with_boxed(scraper_port: Box<dyn ScraperPort>) -> Self {
        Self { scraper_port }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::collab::{CollabSnapshot, CourseEntry, PlaybackEntry};

    struct MockScraper;

    impl ScraperPort for MockScraper {
        fn scrape_collab_html(&self, request: ScrapeRequest) -> Result<CollabSnapshot, ScraperError> {
            if request.html.contains("course-card") {
                return Ok(CollabSnapshot {
                    courses: vec![CourseEntry {
                        course_id: Some("101".to_string()),
                        title: "Rust 101".to_string(),
                        instructor: None,
                        schedule: None,
                    }],
                    playbacks: vec![PlaybackEntry {
                        course_title: Some("Rust 101".to_string()),
                        url: "https://eu.bbcollab.com/recording/1".to_string(),
                        label: Some("Kayit".to_string()),
                    }],
                });
            }

            Err(ScraperError::ParseError("No course markers".to_string()))
        }
    }

    #[test]
    fn rejects_empty_session() {
        let use_case = CollabScraperUseCase::new(MockScraper);
        let err = use_case
            .scrape("", "<div class=\"course-card\"></div>")
            .expect_err("empty session should fail");
        assert!(matches!(err, ScraperError::Unauthorized));
    }

    #[test]
    fn rejects_too_short_html() {
        let use_case = CollabScraperUseCase::new(MockScraper);
        let err = use_case
            .scrape("mdl-session", "<html></html>")
            .expect_err("short html should fail");
        assert!(matches!(err, ScraperError::InvalidInput(_)));
    }
}
