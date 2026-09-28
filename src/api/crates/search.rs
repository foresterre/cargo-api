use crate::api::{Authentication, Endpoint, QueryParams};
use std::borrow::Cow;
use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

/// API to searches and lists crates.
///
/// If no params are give, then all crates are listed alphabetically (by page).
#[derive(Clone, Debug, Default)]
pub struct Search<'a> {
    query: Option<Cow<'a, str>>,
    sort: Option<Sort>,
    include_yanked: Option<bool>,
    category: Option<Cow<'a, str>>,
    filter: Option<Filter<'a>>,
    pagination: Option<Pagination<'a>>,
    per_page: Option<NonZeroU32>,
}

impl<'a> Search<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// The query
    pub fn with_query(mut self, query: Cow<'a, str>) -> Self {
        self.query = Some(query);
        self
    }

    /// Defaults to [`Sort::Relevance`] with a query, and to [`Sort::Alphabetical`] without.
    pub fn with_sort(mut self, sort: Sort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Whether to include yanked crates
    pub fn with_include_yanked(mut self, include_yanked: bool) -> Self {
        self.include_yanked = Some(include_yanked);
        self
    }

    /// Only list crates in the given category, or in one of its subcategories.
    pub fn with_category(mut self, category: Cow<'a, str>) -> Self {
        self.category = Some(category);
        self
    }

    /// Set the filter
    pub fn with_filter(mut self, filter: Filter<'a>) -> Self {
        self.filter = Some(filter);
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

impl<'a> Endpoint for Search<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Borrowed("v1/crates")
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        if let Some(query) = &self.query {
            params.push("q", query.to_string());
        }
        if let Some(sort) = self.sort {
            params.push("sort", sort.as_str());
        }
        if let Some(include_yanked) = self.include_yanked {
            params.push("include_yanked", if include_yanked { "yes" } else { "no" });
        }
        if let Some(category) = &self.category {
            params.push("category", category.to_string());
        }
        if let Some(filter) = &self.filter {
            filter.push_to(&mut params);
        }
        match &self.pagination {
            Some(Pagination::Page(page)) => {
                params.push("page", page.to_string());
            }
            Some(Pagination::Seek(seek)) => {
                params.push("seek", seek.to_string());
            }
            None => {}
        }
        if let Some(per_page) = self.per_page {
            params.push("per_page", per_page.to_string());
        }

        params
    }

    fn authentication(&self) -> Authentication {
        match self.filter {
            Some(Filter::Following) => Authentication::Required,
            _ => Authentication::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Alphabetical,
    Relevance,
    Downloads,
    RecentDownloads,
    RecentUpdates,
    New,
}

impl Sort {
    const ALL: [Sort; 6] = [
        Sort::Alphabetical,
        Sort::Relevance,
        Sort::Downloads,
        Sort::RecentDownloads,
        Sort::RecentUpdates,
        Sort::New,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Sort::Alphabetical => "alphabetical",
            Sort::Relevance => "relevance",
            Sort::Downloads => "downloads",
            Sort::RecentDownloads => "recent-downloads",
            Sort::RecentUpdates => "recent-updates",
            Sort::New => "new",
        }
    }
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Sort {
    type Err = UnknownSort;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Sort::ALL
            .into_iter()
            .find(|sort| sort.as_str() == s)
            .ok_or_else(|| UnknownSort {
                value: s.to_string(),
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "Unknown sort order '{}', expected one of: alphabetical, relevance, downloads, recent-downloads, recent-updates, or new",
    value
)]
pub struct UnknownSort {
    pub value: String,
}

/// Filters listed crates
///
/// crates.io applies at most one of these filters, so only one can be set at the same time.
#[derive(Clone, Debug)]
pub enum Filter<'a> {
    /// Only crates which have all the given keywords.
    AllKeywords(Vec<Cow<'a, str>>),
    Keyword(Cow<'a, str>),
    /// Only crates whose name starts with the given letter.
    Letter(char),
    /// Only crates owned by the crates.io user with the given id.
    UserId(i32),
    /// Only crates owned by the crates.io team with the given id.
    TeamId(i32),
    /// Only crates followed by the authenticated user, requires an API token.
    Following,
    /// Only the crates with the given names.
    Ids(Vec<Cow<'a, str>>),
}

impl<'a> Filter<'a> {
    fn push_to(&self, params: &mut QueryParams) {
        match self {
            Filter::AllKeywords(keywords) => {
                params.push("all_keywords", keywords.join(" "));
            }
            Filter::Keyword(keyword) => {
                params.push("keyword", keyword.to_string());
            }
            Filter::Letter(letter) => {
                params.push("letter", letter.to_string());
            }
            Filter::UserId(id) => {
                params.push("user_id", id.to_string());
            }
            Filter::TeamId(id) => {
                params.push("team_id", id.to_string());
            }
            // crates.io ignores the value, only the presence of the parameter matters
            Filter::Following => {
                params.push("following", "1");
            }
            Filter::Ids(ids) => {
                for id in ids {
                    params.push("ids[]", id.to_string());
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Pagination<'a> {
    Page(NonZeroU32),
    /// The seek key from the `meta.next_page` or `meta.prev_page` field of a previous response.
    Seek(Cow<'a, str>),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(search: Search<'_>) -> Vec<(String, String)> {
        search
            .parameters()
            .iter_tuples()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
        expected
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn nz(n: u32) -> NonZeroU32 {
        NonZeroU32::new(n).unwrap()
    }

    #[yare::parameterized(
        none = { Search::new(), &[] },
        query = { Search::new().with_query("serde".into()).with_per_page(nz(10)), &[("q", "serde"), ("per_page", "10")] },
        sort = { Search::new().with_sort(Sort::RecentDownloads), &[("sort", "recent-downloads")] },
        include_yanked = { Search::new().with_include_yanked(true), &[("include_yanked", "yes")] },
        exclude_yanked = { Search::new().with_include_yanked(false), &[("include_yanked", "no")] },
        category = { Search::new().with_category("parsing".into()), &[("category", "parsing")] },
        all_keywords = { Search::new().with_filter(Filter::AllKeywords(vec!["a".into(), "b".into()])), &[("all_keywords", "a b")] },
        keyword = { Search::new().with_filter(Filter::Keyword("cli".into())), &[("keyword", "cli")] },
        letter = { Search::new().with_filter(Filter::Letter('s')), &[("letter", "s")] },
        user_id = { Search::new().with_filter(Filter::UserId(1)), &[("user_id", "1")] },
        team_id = { Search::new().with_filter(Filter::TeamId(2)), &[("team_id", "2")] },
        following = { Search::new().with_filter(Filter::Following), &[("following", "1")] },
        ids = { Search::new().with_filter(Filter::Ids(vec!["serde".into(), "rand".into()])), &[("ids[]", "serde"), ("ids[]", "rand")] },
        page = { Search::new().with_pagination(Pagination::Page(nz(3))), &[("page", "3")] },
        seek = { Search::new().with_pagination(Pagination::Seek("abc".into())), &[("seek", "abc")] },
    )]
    fn parameters(search: Search<'static>, expected: &[(&str, &str)]) {
        assert_eq!(params(search), pairs(expected));
    }

    #[test]
    fn sort_round_trips() {
        for sort in Sort::ALL {
            assert_eq!(sort.as_str().parse::<Sort>(), Ok(sort));
        }
    }

    #[test]
    fn unknown_sort() {
        assert_eq!(
            "popular".parse::<Sort>(),
            Err(UnknownSort {
                value: "popular".into()
            })
        );
    }
}
