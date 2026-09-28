use crate::api::{Authentication, Endpoint};
use semver::Version;
use std::borrow::Cow;

/// API to yank a crate version
#[derive(Clone, Debug)]
pub struct Yank<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, Version>,
}

impl<'a> Yank<'a> {
    pub fn new(name: Cow<'a, str>, version: Cow<'a, Version>) -> Self {
        Self { name, version }
    }
}

impl<'a> Endpoint for Yank<'a> {
    fn method(&self) -> http::Method {
        http::Method::DELETE
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/{}/yank", self.name, self.version))
    }

    fn authentication(&self) -> Authentication {
        Authentication::Required
    }
}

/// API to unyank a crate version
#[derive(Clone, Debug)]
pub struct Unyank<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, Version>,
}

impl<'a> Unyank<'a> {
    pub fn new(name: Cow<'a, str>, version: Cow<'a, Version>) -> Self {
        Self { name, version }
    }
}

impl<'a> Endpoint for Unyank<'a> {
    fn method(&self) -> http::Method {
        http::Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/{}/unyank", self.name, self.version))
    }

    fn authentication(&self) -> Authentication {
        Authentication::Required
    }
}
