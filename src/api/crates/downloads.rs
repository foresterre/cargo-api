use crate::api::{Endpoint, QueryParams};
use semver::Version;
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

/// API to get the download counts of a crate
///
/// Returns daily count for the last 90 days.
#[derive(Clone, Debug)]
pub struct CrateDownloads<'a> {
    name: Cow<'a, str>,
    include_versions: bool,
}

impl<'a> CrateDownloads<'a> {
    pub fn new(name: Cow<'a, str>) -> Self {
        Self {
            name,
            include_versions: false,
        }
    }

    /// Request thar the metadata of the versions is included in the response
    pub fn with_versions(mut self) -> Self {
        self.include_versions = true;
        self
    }
}

impl<'a> Endpoint for CrateDownloads<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!("v1/crates/{}/downloads", self.name))
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        if self.include_versions {
            params.push("include", "versions");
        }

        params
    }
}

/// API to get the download counts of a crate version
#[derive(Clone, Debug)]
pub struct VersionDownloads<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, Version>,
    before_date: Option<Date>,
}

impl<'a> VersionDownloads<'a> {
    pub fn new(name: Cow<'a, str>, version: Cow<'a, Version>) -> Self {
        Self {
            name,
            version,
            before_date: None,
        }
    }

    /// Only count downloads before the given date
    pub fn with_before_date(mut self, before_date: Date) -> Self {
        self.before_date = Some(before_date);
        self
    }
}

impl<'a> Endpoint for VersionDownloads<'a> {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "v1/crates/{}/{}/downloads",
            self.name, self.version
        ))
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        if let Some(before_date) = self.before_date {
            params.push("before_date", before_date.to_string());
        }

        params
    }
}

/// A calendar date, formatted as `YYYY-MM-DD`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, InvalidDate> {
        let days_in_month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if is_leap_year(year) => 29,
            2 => 28,
            _ => return Err(InvalidDate::Month { month }),
        };

        if day == 0 || day > days_in_month {
            return Err(InvalidDate::Day { year, month, day });
        }

        Ok(Self { year, month, day })
    }
}

// TODO(foresterre): this is probably to naive?
fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for Date {
    type Err = InvalidDate;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('-').collect::<Vec<_>>();
        let [year, month, day] = parts.as_slice() else {
            return Err(InvalidDate::format(s));
        };
        if year.len() != 4 || month.len() != 2 || day.len() != 2 {
            return Err(InvalidDate::format(s));
        }

        // `parse` would  accept a leading `+`, which is not valifd for dates
        let number = |part: &str| {
            part.bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| part.parse().ok())
                .flatten()
                .ok_or_else(|| InvalidDate::format(s))
        };

        Date::new(number(year)?, number(month)? as u8, number(day)? as u8)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum InvalidDate {
    #[error("Invalid date '{}', expected the format YYYY-MM-DD", value)]
    Format { value: String },
    #[error(
        "Invalid month '{}', expected a month from 1 up to and including 12",
        month
    )]
    Month { month: u8 },
    #[error("Invalid day '{}' for month '{}' of year '{}'", day, month, year)]
    Day { year: u16, month: u8, day: u8 },
}

impl InvalidDate {
    fn format(value: impl ToString) -> Self {
        Self::Format {value: value.to_string()}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters() {
        let downloads = VersionDownloads::new("serde".into(), Cow::Owned(Version::new(1, 0, 0)))
            .with_before_date(Date::new(2024, 2, 29).unwrap());

        let params = downloads.parameters();

        assert_eq!(
            params.iter_tuples().collect::<Vec<_>>(),
            [("before_date", "2024-02-29")]
        );
    }

    #[test]
    fn include_versions() {
        let params = CrateDownloads::new("serde".into())
            .with_versions()
            .parameters();

        assert_eq!(
            params.iter_tuples().collect::<Vec<_>>(),
            [("include", "versions")]
        );
    }

    #[yare::parameterized(
        regular = { "2024-01-31", Date::new(2024, 1, 31) },
        leap_day = { "2024-02-29", Date::new(2024, 2, 29) },
        leap_day_400 = { "2000-02-29", Date::new(2000, 2, 29) },
        no_leap_day_100 = { "1900-02-29", Err(InvalidDate::Day { year: 1900, month: 2, day: 29 }) },
        no_leap_day = { "2023-02-29", Err(InvalidDate::Day { year: 2023, month: 2, day: 29 }) },
        day_zero = { "2024-01-00", Err(InvalidDate::Day { year: 2024, month: 1, day: 0 }) },
        day_31_in_30_day_month = { "2024-04-31", Err(InvalidDate::Day { year: 2024, month: 4, day: 31 }) },
        month_zero = { "2024-00-01", Err(InvalidDate::Month { month: 0 }) },
        month_13 = { "2024-13-01", Err(InvalidDate::Month { month: 13 }) },
        no_padding = { "2024-1-01", Err(InvalidDate::Format { value: "2024-1-01".into() }) },
        plus_sign = { "2024-+1-01", Err(InvalidDate::Format { value: "2024-+1-01".into() }) },
        too_many_parts = { "2024-01-01-01", Err(InvalidDate::Format { value: "2024-01-01-01".into() }) },
        empty = { "", Err(InvalidDate::Format { value: "".into() }) },
    )]
    fn parse_date(input: &str, expected: Result<Date, InvalidDate>) {
        assert_eq!(input.parse::<Date>(), expected);
    }

    #[test]
    fn display_round_trips() {
        assert_eq!(Date::new(987, 3, 4).unwrap().to_string(), "0987-03-04");
        assert_eq!("0987-03-04".parse::<Date>(), Date::new(987, 3, 4));
    }
}
