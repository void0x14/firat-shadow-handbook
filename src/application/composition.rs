// Application: Composition Root - Dependency Wiring
// Hexagonal Architecture: Single point for adapter selection

#[cfg(test)]
use crate::domain::collab::{CollabSnapshot, CourseEntry, CourseSchedule, PlaybackEntry};
use crate::domain::ports::auth_port::AuthPort;
#[cfg(test)]
use crate::domain::ports::auth_port::{AuthError, Session};
use crate::domain::ports::scraper_port::ScraperPort;
#[cfg(test)]
use crate::domain::ports::scraper_port::{ScrapeRequest, ScraperError};
use crate::domain::ports::websocket_port::WebSocketPort;
#[cfg(test)]
use crate::domain::user::User;

/// Environment configuration for adapter selection
#[derive(Debug, Clone)]
pub enum AdapterConfig {
    /// Production adapters (real CAS and scraper)
    Production,
    /// Test adapters (test doubles) - only used in tests
    #[cfg(test)]
    Test,
}

/// Composition Root - Single point for dependency wiring
/// Following hexagonal architecture: this is the ONLY place where
/// concrete adapters are instantiated
pub struct CompositionRoot {
    config: AdapterConfig,
}

// Safety: CompositionRoot only contains AdapterConfig which is Sync
// This is required for OnceLock singleton pattern
unsafe impl Sync for CompositionRoot {}

impl CompositionRoot {
    pub fn new(config: AdapterConfig) -> Self {
        Self { config }
    }

    /// Creates LoginUseCase with the appropriate AuthPort adapter
    /// based on the current configuration
    /// Returns Box<dyn AuthPort> for runtime polymorphic adapter selection
    pub fn create_auth_adapter(&self) -> Box<dyn AuthPort> {
        match self.config {
            AdapterConfig::Production => {
                Box::new(crate::infrastructure::cas_adapter::CasAdapter::new(
                    "https://jasig.firat.edu.tr/cas".to_string(),
                    "https://debsis.firat.edu.tr/login/index.php?authCAS=CAS".to_string(),
                ))
            }
            #[cfg(test)]
            AdapterConfig::Test => Box::new(FakeAuthPort::new()),
        }
    }

    /// Creates CollabScraperUseCase with the appropriate ScraperPort adapter
    /// based on the current configuration
    /// Returns Box<dyn ScraperPort> for runtime polymorphic adapter selection
    pub fn create_scraper_adapter(&self) -> Box<dyn ScraperPort> {
        match self.config {
            AdapterConfig::Production => {
                Box::new(crate::infrastructure::collab_scraper_adapter::CollabScraperAdapter::new())
            }
            #[cfg(test)]
            AdapterConfig::Test => Box::new(FakeScraperPort::new()),
        }
    }

    /// Creates WebSocketAdapter for WebSocket operations
    /// Returns Box<dyn WebSocketPort> for runtime polymorphic adapter selection
    pub fn create_websocket_adapter(&self) -> Box<dyn WebSocketPort<Stream = std::net::TcpStream>> {
        match self.config {
            AdapterConfig::Production => {
                Box::new(crate::infrastructure::websocket_adapter::WebSocketAdapter::new())
            }
            #[cfg(test)]
            AdapterConfig::Test => {
                Box::new(crate::infrastructure::websocket_adapter::WebSocketAdapter::new())
            }
        }
    }

    /// Returns the current adapter configuration
    /// Note: Available for introspection/debugging if needed
    #[allow(dead_code)]
    pub fn config(&self) -> &AdapterConfig {
        &self.config
    }
}

// ============================================================================
// Test Doubles - Fake Implementations for Port Testing
// ============================================================================

/// Fake AuthPort for testing - simulates both success and failure scenarios
#[derive(Debug, Clone)]
#[cfg(test)]
pub struct FakeAuthPort {
    /// If Some, always return this session (for success tests)
    forced_session: Option<Session>,
    /// If Some, always return this error (for failure tests)
    forced_error: Option<AuthError>,
}

#[cfg(test)]
impl FakeAuthPort {
    pub fn new() -> Self {
        Self {
            forced_session: None,
            forced_error: None,
        }
    }

    /// Configure for successful authentication
    #[cfg(test)]
    pub fn with_success(mut self, username: &str) -> Self {
        self.forced_session = Some(Session {
            moodle_session: format!("fake_session_{}", username),
            user: User::new(username.to_string())
                .with_full_name(format!("Test {}", username))
                .with_email(format!("{}@test.com", username)),
        });
        self
    }

    /// Configure for authentication failure
    #[cfg(test)]
    pub fn with_failure(mut self, error: AuthError) -> Self {
        self.forced_error = Some(error);
        self
    }
}

#[cfg(test)]
impl Default for FakeAuthPort {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl AuthPort for FakeAuthPort {
    fn authenticate(&self, username: &str, password: &str) -> Result<Session, AuthError> {
        if let Some(ref error) = self.forced_error {
            return Err(error.clone());
        }

        if let Some(ref session) = self.forced_session {
            return Ok(session.clone());
        }

        // Default behavior: success for any non-empty credentials
        if !username.is_empty() && !password.is_empty() {
            Ok(Session {
                moodle_session: format!("session_{}", username),
                user: User::new(username.to_string()),
            })
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    fn validate_session(&self, cookie: &str) -> Result<User, AuthError> {
        if let Some(ref error) = self.forced_error {
            return Err(error.clone());
        }

        if (!cookie.is_empty() && cookie.starts_with("session_")) || cookie == "test_session" {
            let mut user = User::new("testuser".to_string());
            #[cfg(test)]
            {
                user = user
                    .with_full_name("Test User".to_string())
                    .with_email("test@example.com".to_string());
            }
            Ok(user)
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    fn logout(&self, cookie: &str) -> Result<(), AuthError> {
        if let Some(ref error) = self.forced_error {
            return Err(error.clone());
        }

        if !cookie.is_empty() {
            Ok(())
        } else {
            Err(AuthError::InvalidSession)
        }
    }
}

/// Fake ScraperPort for testing - simulates both success and failure scenarios
#[derive(Debug, Clone)]
#[cfg(test)]
pub struct FakeScraperPort {
    /// Forced result for scrape operations
    forced_result: Option<Result<CollabSnapshot, ScraperError>>,
}

#[cfg(test)]
impl FakeScraperPort {
    pub fn new() -> Self {
        Self {
            forced_result: None,
        }
    }
}

#[cfg(test)]
impl Default for FakeScraperPort {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl ScraperPort for FakeScraperPort {
    fn scrape_collab_html(&self, request: ScrapeRequest) -> Result<CollabSnapshot, ScraperError> {
        if let Some(ref result) = self.forced_result {
            return result.clone();
        }

        let html = &request.html;
        let mut courses = Vec::new();
        let mut playbacks = Vec::new();

        if html.contains("course-card") || html.contains("data-course-id") {
            for token in html.split("<div") {
                let has_data_course = token.contains("data-course-id");
                let has_course_card = token.contains("course-card");

                if has_data_course {
                    let course_id = extract_data_attr(token, "data-course-id");
                    let title = extract_data_attr(token, "data-course-title")
                        .unwrap_or("Unknown Course".to_string());
                    let schedule_str = extract_data_attr(token, "data-schedule");

                    let schedule = schedule_str.and_then(|s| {
                        let parts: Vec<&str> = s.split('|').collect();
                        if parts.len() >= 3 {
                            Some(CourseSchedule {
                                start_iso: Some(parts[0].to_string()),
                                end_iso: Some(parts[1].to_string()),
                                timezone: Some(parts[2].to_string()),
                            })
                        } else {
                            None
                        }
                    });

                    courses.push(CourseEntry {
                        course_id,
                        title,
                        instructor: None,
                        schedule,
                    });
                } else if has_course_card {
                    courses.push(CourseEntry {
                        course_id: Some("test101".to_string()),
                        title: "Test Course".to_string(),
                        instructor: None,
                        schedule: None,
                    });
                }
            }
        }

        for token in html.split("<a") {
            if token.contains("playback-link") || token.contains("data-playback") {
                if let Some(url) = extract_href_from_tag(&format!("<a{}", token)) {
                    if url.contains("eu.bbcollab.com/recording") {
                        let label = extract_data_attr(token, "data-label");
                        let course_title = extract_data_attr(
                            token
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .as_str(),
                            "data-course-title",
                        );
                        playbacks.push(PlaybackEntry {
                            course_title,
                            url,
                            label,
                        });
                    }
                }
            }
        }

        if courses.is_empty() && playbacks.is_empty() {
            Err(ScraperError::ParseError(
                "No course markers found".to_string(),
            ))
        } else {
            Ok(CollabSnapshot { courses, playbacks })
        }
    }
}

#[cfg(test)]
fn extract_data_attr(s: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = s.find(&pattern)?;
    let rest = &s[start + pattern.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
fn extract_href_from_tag(tag: &str) -> Option<String> {
    let pattern = "href=\"";
    let start = tag.find(pattern)? + pattern.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

// Re-export for convenience
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_auth_success() {
        let fake = FakeAuthPort::new().with_success("testuser");
        let result = fake.authenticate("testuser", "password");

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.moodle_session, "fake_session_testuser");
    }

    #[test]
    fn test_fake_auth_failure() {
        let fake = FakeAuthPort::new().with_failure(AuthError::InvalidCredentials);
        let result = fake.authenticate("testuser", "wrongpass");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    }

    #[test]
    fn test_fake_scraper_success() {
        let fake = FakeScraperPort::new();
        let result = fake.scrape_collab_html(ScrapeRequest {
            moodle_session: "test".to_string(),
            html: r#"<div class="course-card">Test</div>"#.to_string(),
        });

        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert!(!snapshot.courses.is_empty());
    }

    #[test]
    fn test_fake_scraper_failure() {
        let fake = FakeScraperPort::new();
        let result = fake.scrape_collab_html(ScrapeRequest {
            moodle_session: "test".to_string(),
            html: "<html>No content</html>".to_string(),
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_composition_root_production() {
        let root = CompositionRoot::new(AdapterConfig::Production);

        // Should return production adapters
        let _auth = root.create_auth_adapter();
        let _scraper = root.create_scraper_adapter();

        assert!(matches!(root.config(), AdapterConfig::Production));
    }

    #[test]
    fn test_composition_root_test() {
        let root = CompositionRoot::new(AdapterConfig::Test);

        // Should return test doubles
        let auth = root.create_auth_adapter();
        let scraper = root.create_scraper_adapter();

        // Verify they are our fake implementations
        let auth_result = auth.authenticate("test", "test");
        assert!(auth_result.is_ok());

        let scraper_result = scraper.scrape_collab_html(ScrapeRequest {
            moodle_session: "test".to_string(),
            html: "<div class='course-card'>Test</div>".to_string(),
        });
        assert!(scraper_result.is_ok());
    }
}
