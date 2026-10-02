use crate::api::Endpoint;
use semver::Version;
use std::borrow::Cow;

/// API to get the authors of a crate version.
///
/// Deprecated by crates.io (RFC 3052), the response always has an empty list of authors.
#[derive(Clone, Debug)]
pub struct Authors<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, Version>,
}

impl<'a> Authors<'a> {
    pub fn new(name: Cow<'a, str>, version: Cow<'a, Version>) -> Self {
        Self { name, version }
    }
}

impl<'a> Endpoint for Authors<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/{}/authors", self.name, self.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint() {
        let authors = Authors::new("serde".into(), Cow::Owned(Version::new(1, 0, 0)));

        assert_eq!(authors.method(), http::Method::GET);
        assert_eq!(authors.endpoint(), "v1/crates/serde/1.0.0/authors");
    }
}
