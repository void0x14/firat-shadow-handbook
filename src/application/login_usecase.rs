// Application: Login Use Case

use crate::domain::ports::auth_port::{AuthError, AuthPort, Session};
use crate::domain::user::User;

pub struct LoginUseCase<T: AuthPort> {
    auth_port: T,
}

impl<T: AuthPort> LoginUseCase<T> {
    /// Creates a new LoginUseCase with the given AuthPort
    /// Note: Currently using with_boxed() in main.rs for runtime polymorphism
    #[allow(dead_code)]
    pub fn new(auth_port: T) -> Self {
        Self { auth_port }
    }

    /// Executes the login use case
    pub fn login(&self, username: &str, password: &str) -> Result<Session, AuthError> {
        // Validate input
        if !is_valid_username(username) || !is_valid_password(password) {
            return Err(AuthError::InvalidCredentials);
        }

        // Delegate to port
        self.auth_port.authenticate(username, password)
    }

    /// Validates an existing session
    pub fn validate_session(&self, cookie: &str) -> Result<User, AuthError> {
        if cookie.is_empty() {
            return Err(AuthError::InvalidSession);
        }

        self.auth_port.validate_session(cookie)
    }

    /// Logs out the user
    pub fn logout(&self, cookie: &str) -> Result<(), AuthError> {
        if cookie.is_empty() {
            return Err(AuthError::InvalidSession);
        }

        self.auth_port.logout(cookie)
    }
}

// Support for Box<dyn AuthPort> - enables runtime polymorphic adapter selection
impl LoginUseCase<Box<dyn AuthPort>> {
    pub fn with_boxed(auth_port: Box<dyn AuthPort>) -> Self {
        Self { auth_port }
    }
}

fn is_valid_username(username: &str) -> bool {
    if username.is_empty() || username.len() > 64 {
        return false;
    }
    username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '@'))
}

fn is_valid_password(password: &str) -> bool {
    !password.is_empty() && password.len() <= 128
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::auth_port::{AuthError, AuthPort, Session};
    use crate::domain::user::User;
    use std::cell::RefCell;

    struct MockAuthPort;

    impl AuthPort for MockAuthPort {
        fn authenticate(&self, username: &str, password: &str) -> Result<Session, AuthError> {
            if username == "testuser" && password == "testpass" {
                Ok(Session {
                    moodle_session: "test_session".to_string(),
                    user: User::new("testuser".to_string())
                        .with_full_name("Test User".to_string())
                        .with_email("test@example.com".to_string()),
                })
            } else {
                Err(AuthError::InvalidCredentials)
            }
        }

        fn validate_session(&self, cookie: &str) -> Result<User, AuthError> {
            if cookie == "test_session" {
                Ok(User::new("testuser".to_string())
                    .with_full_name("Test User".to_string())
                    .with_email("test@example.com".to_string()))
            } else {
                Err(AuthError::InvalidSession)
            }
        }

        fn logout(&self, cookie: &str) -> Result<(), AuthError> {
            if cookie == "test_session" {
                Ok(())
            } else {
                Err(AuthError::InvalidSession)
            }
        }
    }

    struct StatefulAuthPort {
        active_session: RefCell<Option<String>>,
    }

    impl StatefulAuthPort {
        fn new() -> Self {
            Self {
                active_session: RefCell::new(None),
            }
        }
    }

    impl AuthPort for StatefulAuthPort {
        fn authenticate(&self, username: &str, password: &str) -> Result<Session, AuthError> {
            if username != "testuser" || password != "testpass" {
                return Err(AuthError::InvalidCredentials);
            }
            let session_id = "stateful_session".to_string();
            *self.active_session.borrow_mut() = Some(session_id.clone());
            Ok(Session {
                moodle_session: session_id,
                user: User::new("testuser".to_string()),
            })
        }

        fn validate_session(&self, cookie: &str) -> Result<User, AuthError> {
            match self.active_session.borrow().as_deref() {
                Some(active) if active == cookie => Ok(User::new("testuser".to_string())),
                _ => Err(AuthError::InvalidSession),
            }
        }

        fn logout(&self, cookie: &str) -> Result<(), AuthError> {
            let is_active = self
                .active_session
                .borrow()
                .as_deref()
                .map(|active| active == cookie)
                .unwrap_or(false);

            if is_active {
                *self.active_session.borrow_mut() = None;
                Ok(())
            } else {
                Err(AuthError::InvalidSession)
            }
        }
    }

    #[test]
    fn test_login_success() {
        let use_case = LoginUseCase::new(MockAuthPort);
        let result = use_case.login("testuser", "testpass");

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.moodle_session, "test_session");
        assert_eq!(session.user.username, "testuser");
        assert_eq!(session.user.full_name, Some("Test User".to_string()));
        assert_eq!(session.user.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_login_invalid_credentials() {
        let use_case = LoginUseCase::new(MockAuthPort);
        let result = use_case.login("invalid", "credentials");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    }

    #[test]
    fn test_login_empty_credentials() {
        let use_case = LoginUseCase::new(MockAuthPort);

        let result1 = use_case.login("", "password");
        assert!(result1.is_err());
        assert!(matches!(
            result1.unwrap_err(),
            AuthError::InvalidCredentials
        ));

        let result2 = use_case.login("username", "");
        assert!(result2.is_err());
        assert!(matches!(
            result2.unwrap_err(),
            AuthError::InvalidCredentials
        ));
    }

    #[test]
    fn test_login_rejects_illegal_username_chars() {
        let use_case = LoginUseCase::new(MockAuthPort);
        let result = use_case.login("bad user!", "testpass");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
    }

    #[test]
    fn test_validate_session_success() {
        let use_case = LoginUseCase::new(MockAuthPort);
        let result = use_case.validate_session("test_session");

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
    }

    #[test]
    fn test_validate_session_invalid() {
        let use_case = LoginUseCase::new(MockAuthPort);
        let result = use_case.validate_session("invalid_session");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidSession));
    }

    #[test]
    fn test_logout_success() {
        let use_case = LoginUseCase::new(MockAuthPort);
        let result = use_case.logout("test_session");

        assert!(result.is_ok());
    }

    #[test]
    fn test_logout_invalid() {
        let use_case = LoginUseCase::new(MockAuthPort);
        let result = use_case.logout("invalid_session");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidSession));
    }

    #[test]
    fn test_auth_lifecycle_login_validate_logout_validate_invalid() {
        let use_case = LoginUseCase::new(StatefulAuthPort::new());

        let session = use_case
            .login("testuser", "testpass")
            .expect("login should succeed");
        use_case
            .validate_session(&session.moodle_session)
            .expect("session should validate right after login");
        use_case
            .logout(&session.moodle_session)
            .expect("logout should succeed");
        let after_logout = use_case.validate_session(&session.moodle_session);
        assert!(
            matches!(after_logout, Err(AuthError::InvalidSession)),
            "session must be invalid after logout"
        );
    }
}
