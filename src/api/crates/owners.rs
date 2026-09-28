use crate::api::{Authentication, Body, BodyError, Endpoint};
use std::borrow::Cow;

/// API to lists the owners of a crate.
#[derive(Clone, Debug)]
pub struct Owners<'a> {
    name: Cow<'a, str>,
}

impl<'a> Owners<'a> {
    pub fn new(name: Cow<'a, str>) -> Self {
        Self { name }
    }
}

impl<'a> Endpoint for Owners<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/owners", self.name))
    }
}

/// API to invites users or teams to become (co-)owners of a crate.
///
/// Owners are given by their Cargo login, e.g. `username`, `github:username`, or `github:org:team`.
#[derive(Clone, Debug)]
pub struct AddOwners<'a> {
    name: Cow<'a, str>,
    owners: Vec<Cow<'a, str>>,
}

impl<'a> AddOwners<'a> {
    pub fn new(name: Cow<'a, str>, owners: Vec<Cow<'a, str>>) -> Self {
        Self { name, owners }
    }
}

impl<'a> Endpoint for AddOwners<'a> {
    fn method(&self) -> http::Method {
        http::Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/owners", self.name))
    }

    fn body(&self) -> Result<Option<Body>, BodyError> {
        owners_body(&self.owners).map(Some)
    }

    fn authentication(&self) -> Authentication {
        Authentication::Required
    }
}

/// API which removes users or teams as owners of a crate.
///
/// Owners are given by their Cargo login, e.g. `username`, `github:username`, or `github:org:team`.
#[derive(Clone, Debug)]
pub struct RemoveOwners<'a> {
    name: Cow<'a, str>,
    owners: Vec<Cow<'a, str>>,
}

impl<'a> RemoveOwners<'a> {
    pub fn new(name: Cow<'a, str>, owners: Vec<Cow<'a, str>>) -> Self {
        Self { name, owners }
    }
}

impl<'a> Endpoint for RemoveOwners<'a> {
    fn method(&self) -> http::Method {
        http::Method::DELETE
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/owners", self.name))
    }

    fn body(&self) -> Result<Option<Body>, BodyError> {
        owners_body(&self.owners).map(Some)
    }

    fn authentication(&self) -> Authentication {
        Authentication::Required
    }
}

fn owners_body(owners: &[Cow<'_, str>]) -> Result<Body, BodyError> {
    #[derive(serde::Serialize)]
    struct OwnersBody<'b> {
        owners: &'b [Cow<'b, str>],
    }

    Body::json(&OwnersBody { owners })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body() {
        let endpoint = AddOwners::new(
            "serde".into(),
            vec!["ghost".into(), "github:org:team".into()],
        );
        let body = endpoint.body().unwrap().unwrap();

        assert_eq!(body.content_type(), "application/json");
        assert_eq!(
            body.into_bytes(),
            br#"{"owners":["ghost","github:org:team"]}"#
        );
    }
}
