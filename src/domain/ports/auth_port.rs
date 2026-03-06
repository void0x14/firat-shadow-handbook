// Domain: Authentication Port (Hexagonal Architecture)

use crate::domain::user::User;

pub trait AuthPort {
    /// Authenticates user with CAS and returns session cookie
    fn authenticate(&self, username: &str, password: &str) -> Result<Session, AuthError>;

    /// Validates an existing session cookie
    fn validate_session(&self, cookie: &str) -> Result<User, AuthError>;

    /// Logs out user by invalidating the session
    fn logout(&self, cookie: &str) -> Result<(), AuthError>;
}

#[derive(Debug, Clone)]
pub struct Session {
    pub moodle_session: String,
    pub user: User,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("CAS server error: {0}")]
    CasServerError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Session invalid or expired")]
    InvalidSession,

    #[error("HTML parsing error: {0}")]
    ParsingError(String),
}
