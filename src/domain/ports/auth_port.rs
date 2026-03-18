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

#[derive(Debug, Clone)]
pub enum AuthError {
    InvalidCredentials,
    CasServerError(String),
    NetworkError(String),
    InvalidSession,
    ParsingError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "Invalid credentials"),
            Self::CasServerError(msg) => write!(f, "CAS server error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::InvalidSession => write!(f, "Session invalid or expired"),
            Self::ParsingError(msg) => write!(f, "HTML parsing error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}
