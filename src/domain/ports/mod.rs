// Domain ports module

pub mod auth_port;
pub mod scraper_port;

// Blanket implementations for Box<dyn Trait>
// This enables runtime polymorphic adapter selection via CompositionRoot

impl<T: auth_port::AuthPort + ?Sized> auth_port::AuthPort for Box<T> {
    fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<auth_port::Session, auth_port::AuthError> {
        (**self).authenticate(username, password)
    }

    fn validate_session(
        &self,
        cookie: &str,
    ) -> Result<crate::domain::user::User, auth_port::AuthError> {
        (**self).validate_session(cookie)
    }

    fn logout(&self, cookie: &str) -> Result<(), auth_port::AuthError> {
        (**self).logout(cookie)
    }
}

impl<T: scraper_port::ScraperPort + ?Sized> scraper_port::ScraperPort for Box<T> {
    fn scrape_collab_html(
        &self,
        request: scraper_port::ScrapeRequest,
    ) -> Result<crate::domain::collab::CollabSnapshot, scraper_port::ScraperError> {
        (**self).scrape_collab_html(request)
    }
}
