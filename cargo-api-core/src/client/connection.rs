use crate::api::{Connection, MissingToken};
use crate::client::Token;
use http::request::Builder;
use url::Url;

pub const CRATES_IO_API: &str = "https://crates.io/api/";

#[derive(Clone, Debug)]
pub struct CratesIoConnection {
    // Doesn't include the version and beyond.
    // A full path would be: https://crates.io/api/v1/crates/cargo-api
    url: Url,
    token: Option<Token>,
}

impl CratesIoConnection {
    pub fn new(url: Url) -> Self {
        Self {
            url: with_trailing_slash(url),
            token: None,
        }
    }

    /// API token for endpoints which require authentication.
    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }
}

impl Default for CratesIoConnection {
    fn default() -> Self {
        Self::new(Url::parse(CRATES_IO_API).unwrap())
    }
}

impl Connection for CratesIoConnection {
    fn base_url(&self) -> &Url {
        &self.url
    }

    fn authorize(&self, request_builder: Builder) -> Result<Builder, MissingToken> {
        match &self.token {
            Some(token) => Ok(request_builder
                .header(http::header::AUTHORIZATION, token.as_header_value().clone())),
            None => Err(MissingToken),
        }
    }
}

fn with_trailing_slash(mut url: Url) -> Url {
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }

    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[yare::parameterized(
        with_slash = { "https://crates.test/api/" },
        without_slash = { "https://crates.test/api" },
    )]
    fn joins_endpoints_below_the_base_url(url: &str) {
        let connection = CratesIoConnection::new(Url::parse(url).unwrap());

        assert_eq!(
            connection.base_url().join("v1/crates").unwrap().as_str(),
            "https://crates.test/api/v1/crates"
        );
    }

    #[test]
    fn defaults_to_crates_io() {
        assert_eq!(
            CratesIoConnection::default().base_url().as_str(),
            CRATES_IO_API
        );
    }

    #[test]
    fn authorizes_with_token() {
        let connection =
            CratesIoConnection::default().with_token(Token::new("cio_secret").unwrap());

        let request = connection
            .authorize(http::Request::builder())
            .unwrap()
            .body(())
            .unwrap();

        let value = &request.headers()[http::header::AUTHORIZATION];
        assert_eq!(value, "cio_secret");
        assert!(value.is_sensitive());
    }

    #[test]
    fn missing_token() {
        let result = CratesIoConnection::default().authorize(http::Request::builder());

        assert_eq!(result.err(), Some(MissingToken));
    }
}
