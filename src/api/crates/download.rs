use crate::api::Endpoint;
use semver::Version;
use std::borrow::Cow;

/// API to get the path of the `.crate` file for a given crate version.
///
/// Response consists of the URL of the file (iso the crate itself).
#[derive(Clone, Debug)]
pub struct Download<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, Version>,
}

impl<'a> Download<'a> {
    pub fn new(name: Cow<'a, str>, version: Cow<'a, Version>) -> Self {
        Self { name, version }
    }
}

impl<'a> Endpoint for Download<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/{}/download", self.name, self.version))
    }
}
