use crate::api::crates::Pagination;
use crate::api::{Endpoint, QueryParams};
use semver::Version;
use std::borrow::Cow;
use std::num::NonZeroU32;

/// API to list the dependencies of a crate version.
#[derive(Clone, Debug)]
pub struct VersionDependencies<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, Version>,
}

impl<'a> VersionDependencies<'a> {
    pub fn new(name: Cow<'a, str>, version: Cow<'a, Version>) -> Self {
        Self { name, version }
    }
}

impl<'a> Endpoint for VersionDependencies<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "v1/crates/{}/{}/dependencies",
            self.name, self.version
        ))
    }
}

/// API to list the reverse dependencies of a crate.
#[derive(Clone, Debug)]
pub struct ReverseDependencies<'a> {
    name: Cow<'a, str>,
    pagination: Option<Pagination<'a>>,
    per_page: Option<NonZeroU32>,
}

impl<'a> ReverseDependencies<'a> {
    pub fn new(name: Cow<'a, str>) -> Self {
        Self {
            name,
            pagination: None,
            per_page: None,
        }
    }

    /// Set the pagination
    pub fn with_pagination(mut self, pagination: Pagination<'a>) -> Self {
        self.pagination = Some(pagination);
        self
    }

    /// Set the amount of results to be requested per page
    pub fn with_per_page(mut self, per_page: NonZeroU32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

impl<'a> Endpoint for ReverseDependencies<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/reverse_dependencies", self.name))
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        if let Some(pagination) = &self.pagination {
            pagination.push_query_params(&mut params);
        }
        if let Some(per_page) = self.per_page {
            params.push("per_page", per_page.to_string());
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters() {
        let reverse_dependencies = ReverseDependencies::new("serde".into())
            .with_pagination(Pagination::Seek("abc".into()))
            .with_per_page(NonZeroU32::new(10).unwrap());

        let params = reverse_dependencies.parameters();

        assert_eq!(
            params.iter_tuples().collect::<Vec<_>>(),
            [("seek", "abc"), ("per_page", "10")]
        );
    }
}
