// Domain: User entity

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UserRole {
    Student,
    Teacher,
    Admin,
    #[default]
    Unknown,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Student => "student",
            Self::Teacher => "teacher",
            Self::Admin => "admin",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "teacher" => Self::Teacher,
            "student" => Self::Student,
            _ => Self::Unknown,
        }
    }

    pub fn from_moodle_role_names(role_names: &[String]) -> Self {
        let normalized: Vec<String> = role_names
            .iter()
            .map(|role| role.trim().to_ascii_lowercase())
            .collect();

        if normalized.iter().any(|role| role == "admin") {
            return Self::Admin;
        }

        if normalized.iter().any(|role| {
            matches!(
                role.as_str(),
                "teacher" | "editingteacher" | "manager" | "coursecreator"
            )
        }) {
            return Self::Teacher;
        }

        if normalized.iter().any(|role| role == "student") {
            return Self::Student;
        }

        Self::Unknown
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub role: UserRole,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            username,
            full_name: None,
            email: None,
            role: UserRole::Unknown,
        }
    }

    /// Builder pattern for setting full name (used in tests)
    #[cfg(test)]
    pub fn with_full_name(mut self, full_name: String) -> Self {
        self.full_name = Some(full_name);
        self
    }

    /// Builder pattern for setting email (used in tests)
    #[cfg(test)]
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Builder pattern for setting role (used in tests)
    #[cfg(test)]
    pub fn with_role(mut self, role: UserRole) -> Self {
        self.role = role;
        self
    }
}
