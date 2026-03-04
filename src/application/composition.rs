// Application: Composition Root - Dependency Wiring
// Hexagonal Architecture: Single point for adapter selection

use crate::domain::ports::auth_port::AuthPort;
#[cfg(test)]
use crate::domain::ports::auth_port::{Session, AuthError};
use crate::domain::ports::scraper_port::ScraperPort;
#[cfg(test)]
use crate::domain::ports::scraper_port::{ScrapeRequest, ScraperError};
#[cfg(test)]
use crate::domain::collab::CollabSnapshot;
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
                user = user.with_full_name("Test User".to_string())
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

        // Default behavior: parse HTML for course cards
        if request.html.contains("course-card") {
            Ok(CollabSnapshot {
                courses: vec![CourseEntry {
                    course_id: Some("test101".to_string()),
                    title: "Test Course".to_string(),
                    instructor: None,
                    schedule: None,
                }],
                playbacks: vec![],
            })
        } else {
            Err(ScraperError::ParseError("No course markers found".to_string()))
        }
    }
}

// Re-export for convenience
#[cfg(test)]
use crate::domain::collab::CourseEntry;

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
