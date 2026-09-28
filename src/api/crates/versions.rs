use crate::api::crates::Pagination;
use crate::api::{Endpoint, QueryParams};
use semver::Version;
use std::borrow::Cow;
use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

/// API to list the versions of a crate.
#[derive(Clone, Debug)]
pub struct Versions<'a> {
    name: Cow<'a, str>,
    sort: Option<VersionSort>,
    include_release_tracks: bool,
    nums: Vec<Cow<'a, Version>>,
    pagination: Option<Pagination<'a>>,
    per_page: Option<NonZeroU32>,
}

impl<'a> Versions<'a> {
    pub fn new(name: Cow<'a, str>) -> Self {
        Self {
            name,
            sort: None,
            include_release_tracks: false,
            nums: Vec::new(),
            pagination: None,
            per_page: None,
        }
    }

    /// Defaults to [`VersionSort::Semver`].
    pub fn with_sort(mut self, sort: VersionSort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Include the latest version of each release track (e.g. `1.x`) in the response
    pub fn with_release_tracks(mut self) -> Self {
        self.include_release_tracks = true;
        self
    }

    /// Only list the given versions
    pub fn with_nums(mut self, nums: Vec<Cow<'a, Version>>) -> Self {
        self.nums = nums;
        self
    }

    /// Set the pagination opts
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

impl<'a> Endpoint for Versions<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/versions", self.name))
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        if let Some(sort) = self.sort {
            params.push("sort", sort.as_str());
        }
        if self.include_release_tracks {
            params.push("include", "release_tracks");
        }
        for num in &self.nums {
            params.push("nums[]", num.to_string());
        }
        if let Some(pagination) = &self.pagination {
            pagination.push_query_params(&mut params);
        }
        if let Some(per_page) = self.per_page {
            params.push("per_page", per_page.to_string());
        }

        params
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionSort {
    Date,
    Semver,
}

impl VersionSort {
    pub fn as_str(self) -> &'static str {
        match self {
            VersionSort::Date => "date",
            VersionSort::Semver => "semver",
        }
    }
}

impl fmt::Display for VersionSort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VersionSort {
    type Err = UnknownVersionSort;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        [VersionSort::Date, VersionSort::Semver]
            .into_iter()
            .find(|sort| sort.as_str() == s)
            .ok_or_else(|| UnknownVersionSort {
                value: s.to_string(),
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("Unknown sort order '{}', expected one of: date, or semver", value)]
pub struct UnknownVersionSort {
    pub value: String,
}

/// API to get the metadata of a crate version.
#[derive(Clone, Debug)]
pub struct CrateVersion<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, Version>,
}

impl<'a> CrateVersion<'a> {
    pub fn new(name: Cow<'a, str>, version: Cow<'a, Version>) -> Self {
        Self { name, version }
    }
}

impl<'a> Endpoint for CrateVersion<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/{}", self.name, self.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters() {
        let versions = Versions::new("serde".into())
            .with_sort(VersionSort::Date)
            .with_release_tracks()
            .with_nums(vec![
                Cow::Owned(Version::new(1, 0, 0)),
                Cow::Owned(Version::new(1, 0, 1)),
            ])
            .with_pagination(Pagination::Page(NonZeroU32::new(2).unwrap()))
            .with_per_page(NonZeroU32::new(10).unwrap());

        let params = versions.parameters();

        assert_eq!(
            params.iter_tuples().collect::<Vec<_>>(),
            [
                ("sort", "date"),
                ("include", "release_tracks"),
                ("nums[]", "1.0.0"),
                ("nums[]", "1.0.1"),
                ("page", "2"),
                ("per_page", "10"),
            ]
        );
    }

    #[yare::parameterized(
        date = { "date", Ok(VersionSort::Date) },
        semver = { "semver", Ok(VersionSort::Semver) },
        unknown = { "newest", Err(UnknownVersionSort { value: "newest".into() }) },
    )]
    fn parse_sort(input: &str, expected: Result<VersionSort, UnknownVersionSort>) {
        assert_eq!(input.parse::<VersionSort>(), expected);
    }
}
