// Infrastructure: CAS Adapter (mock CAS authentication for Story 2.1 baseline)

use chrono::Utc;

use crate::domain::ports::auth_port::{AuthPort, Session, AuthError};
use crate::domain::user::User;

pub struct CasAdapter {
    cas_base_url: String,
    service_url: String,
}

impl CasAdapter {
    pub fn new(cas_base_url: String, service_url: String) -> Self {
        Self {
            cas_base_url,
            service_url,
        }
    }
}

impl AuthPort for CasAdapter {
    fn authenticate(&self, username: &str, password: &str) -> Result<Session, AuthError> {
        // Story 2.1 baseline: deterministic mock flow.
        // Real CAS TGT/ST flow will replace this implementation in the next DS cycle.
        if username.is_empty() || password.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }

        // Mock successful authentication
        Ok(Session {
            moodle_session: format!("mock_session_{}_{}", self.cas_base_url.len(), username),
            user: User::new(username.to_string())
                .with_full_name(format!("{} {}", username, "User"))
                .with_email(format!(
                    "{}@firat.edu.tr",
                    if self.service_url.contains("debsis") { username } else { "unknown" }
                )),
            expires_at: Utc::now().checked_add_signed(chrono::Duration::hours(24))
                .expect("Invalid expiration time"),
        })
    }

    fn validate_session(&self, cookie: &str) -> Result<User, AuthError> {
        if cookie.starts_with("mock_session_") {
            let username = cookie.strip_prefix("mock_session_").unwrap_or("");
            Ok(User::new(username.to_string())
                .with_full_name(format!("{} {}", username, "User"))
                .with_email(format!("{}@firat.edu.tr", username)))
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    fn logout(&self, cookie: &str) -> Result<(), AuthError> {
        if cookie.starts_with("mock_session_") {
            Ok(())
        } else {
            Err(AuthError::InvalidSession)
        }
    }
}
