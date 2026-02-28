// Domain: User entity

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            username,
            full_name: None,
            email: None,
        }
    }

    pub fn with_full_name(mut self, full_name: String) -> Self {
        self.full_name = Some(full_name);
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
}
