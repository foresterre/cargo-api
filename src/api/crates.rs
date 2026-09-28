#[cfg(feature = "deprecated")]
pub mod authors;
pub mod crate_;
pub mod dependencies;
pub mod download;
pub mod downloads;
pub mod owners;
pub mod publish;
pub mod readme;
pub mod search;
pub mod versions;
pub mod yank;

#[cfg(feature = "deprecated")]
pub use authors::Authors;
pub use crate_::Crate;
pub use dependencies::{ReverseDependencies, VersionDependencies};
pub use download::Download;
pub use downloads::{CrateDownloads, Date, InvalidDate, VersionDownloads};
pub use owners::{AddOwners, Owners, RemoveOwners, TeamOwners, UserOwners};
pub use publish::{Dependency, DependencyKind, Publish, PublishError, PublishMetadata};
pub use readme::Readme;
pub use search::{Filter, Pagination, Search, Sort, UnknownSort};
pub use versions::{CrateVersion, UnknownVersionSort, VersionSort, Versions};
pub use yank::{Unyank, Yank};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Authentication, Endpoint};
    use http::Method;
    use semver::Version;
    use std::borrow::Cow;

    fn version() -> Cow<'static, Version> {
        Cow::Owned(Version::new(1, 0, 0))
    }

    #[yare::parameterized(
        crate_ = { &Crate::new("serde".into()), Method::GET, "v1/crates/serde", Authentication::None },
        search = { &Search::new(), Method::GET, "v1/crates", Authentication::None },
        search_following = { &Search::new().with_filter(Filter::Following), Method::GET, "v1/crates", Authentication::Required },
        publish = { &Publish::new(&PublishMetadata::new("serde".into(), Version::new(1, 0, 0)), b"").unwrap(), Method::PUT, "v1/crates/new", Authentication::Required },
        owners = { &Owners::new("serde".into()), Method::GET, "v1/crates/serde/owners", Authentication::None },
        add_owners = { &AddOwners::new("serde".into(), vec!["ghost".into()]), Method::PUT, "v1/crates/serde/owners", Authentication::Required },
        remove_owners = { &RemoveOwners::new("serde".into(), vec!["ghost".into()]), Method::DELETE, "v1/crates/serde/owners", Authentication::Required },
        yank = { &Yank::new("serde".into(), version()), Method::DELETE, "v1/crates/serde/1.0.0/yank", Authentication::Required },
        unyank = { &Unyank::new("serde".into(), version()), Method::PUT, "v1/crates/serde/1.0.0/unyank", Authentication::Required },
        download = { &Download::new("serde".into(), version()), Method::GET, "v1/crates/serde/1.0.0/download", Authentication::None },
        versions = { &Versions::new("serde".into()), Method::GET, "v1/crates/serde/versions", Authentication::None },
        crate_version = { &CrateVersion::new("serde".into(), version()), Method::GET, "v1/crates/serde/1.0.0", Authentication::None },
        version_dependencies = { &VersionDependencies::new("serde".into(), version()), Method::GET, "v1/crates/serde/1.0.0/dependencies", Authentication::None },
        reverse_dependencies = { &ReverseDependencies::new("serde".into()), Method::GET, "v1/crates/serde/reverse_dependencies", Authentication::None },
        crate_downloads = { &CrateDownloads::new("serde".into()), Method::GET, "v1/crates/serde/downloads", Authentication::None },
        version_downloads = { &VersionDownloads::new("serde".into(), version()), Method::GET, "v1/crates/serde/1.0.0/downloads", Authentication::None },
        readme = { &Readme::new("serde".into(), version()), Method::GET, "v1/crates/serde/1.0.0/readme", Authentication::None },
        user_owners = { &UserOwners::new("serde".into()), Method::GET, "v1/crates/serde/owner_user", Authentication::None },
        team_owners = { &TeamOwners::new("serde".into()), Method::GET, "v1/crates/serde/owner_team", Authentication::None },
    )]
    fn endpoint(
        endpoint: &dyn Endpoint,
        method: Method,
        path: &str,
        authentication: Authentication,
    ) {
        assert_eq!(endpoint.method(), method);
        assert_eq!(endpoint.endpoint(), path);
        assert_eq!(endpoint.authentication(), authentication);
    }
}
