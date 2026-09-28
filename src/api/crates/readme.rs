use crate::api::Endpoint;
use semver::Version;
use std::borrow::Cow;

/// API to get the path of the readme of a crate version.
///
/// Response consists of the URL of the readme (iso the readme itself).
#[derive(Clone, Debug)]
pub struct Readme<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, Version>,
}

impl<'a> Readme<'a> {
    pub fn new(name: Cow<'a, str>, version: Cow<'a, Version>) -> Self {
        Self { name, version }
    }
}

impl<'a> Endpoint for Readme<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/{}/readme", self.name, self.version))
    }
}
