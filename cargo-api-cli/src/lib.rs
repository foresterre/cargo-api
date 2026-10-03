use anyhow::Context;
#[cfg(feature = "deprecated")]
use cargo_api_core::api::crates::Authors;
use cargo_api_core::api::crates::{
    AddOwners, Crate, CrateDownloads, CrateVersion, Date, Download, Owners, Pagination, Publish,
    PublishMetadata, Readme, RemoveOwners, ReverseDependencies, Search, Sort, TeamOwners, Unyank,
    UserOwners, VersionDependencies, VersionDownloads, VersionSort, Versions, Yank,
};
use cargo_api_core::api::{Json, Query};
use cargo_api_core::client::{ReqwestClient, Token};
use clap::{Args, Parser};
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::path::PathBuf;

pub fn run() -> anyhow::Result<()> {
    let CargoCli::Api(args) = CargoCli::parse();

    let mut client = ReqwestClient::new(args.user_agent.as_str());

    if let Some(token) = args.token.as_deref() {
        client = client.with_token(Token::new(token)?);
    }

    let value = match args.subcommand {
        Subcommand::Crate(opts) => {
            Json::new(Crate::new(Cow::Borrowed(opts.name.as_str()))).query(&client)?
        }
        Subcommand::Search(opts) => Json::new(opts.endpoint()).query(&client)?,
        Subcommand::Publish(opts) => Json::new(opts.endpoint()?).query(&client)?,
        Subcommand::Owner(OwnerCommand::List(opts)) => {
            Json::new(Owners::new(Cow::Borrowed(opts.name.as_str()))).query(&client)?
        }
        Subcommand::Owner(OwnerCommand::Users(opts)) => {
            Json::new(UserOwners::new(Cow::Borrowed(opts.name.as_str()))).query(&client)?
        }
        Subcommand::Owner(OwnerCommand::Teams(opts)) => {
            Json::new(TeamOwners::new(Cow::Borrowed(opts.name.as_str()))).query(&client)?
        }
        Subcommand::Owner(OwnerCommand::Add(opts)) => {
            Json::new(AddOwners::new(opts.name(), opts.owners())).query(&client)?
        }
        Subcommand::Owner(OwnerCommand::Remove(opts)) => {
            Json::new(RemoveOwners::new(opts.name(), opts.owners())).query(&client)?
        }
        Subcommand::Yank(opts) => {
            Json::new(Yank::new(opts.name(), opts.version())).query(&client)?
        }
        Subcommand::Unyank(opts) => {
            Json::new(Unyank::new(opts.name(), opts.version())).query(&client)?
        }
        Subcommand::Download(opts) => {
            Json::new(Download::new(opts.name(), opts.version())).query(&client)?
        }
        Subcommand::Versions(opts) => Json::new(opts.endpoint()).query(&client)?,
        Subcommand::Version(opts) => {
            Json::new(CrateVersion::new(opts.name(), opts.version())).query(&client)?
        }
        Subcommand::Dependencies(opts) => {
            Json::new(VersionDependencies::new(opts.name(), opts.version())).query(&client)?
        }
        Subcommand::ReverseDependencies(opts) => Json::new(opts.endpoint()).query(&client)?,
        Subcommand::Downloads(opts) => Json::new(opts.endpoint()).query(&client)?,
        Subcommand::VersionDownloads(opts) => Json::new(opts.endpoint()).query(&client)?,
        Subcommand::Readme(opts) => {
            Json::new(Readme::new(opts.name(), opts.version())).query(&client)?
        }
        #[cfg(feature = "deprecated")]
        Subcommand::Authors(opts) => {
            Json::new(Authors::new(opts.name(), opts.version())).query(&client)?
        }
    };

    println!("{}", value);

    Ok(())
}

#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
enum CargoCli {
    Api(ApiArgs),
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct ApiArgs {
    #[command(flatten)]
    manifest: clap_cargo::Manifest,

    #[arg(long)]
    user_agent: String,

    /// API token for endpoints which require authentication.
    ///
    /// Prefer the environment variable, so the token does not end up in your shell history.
    #[arg(long, env = "CARGO_REGISTRY_TOKEN", hide_env_values = true)]
    token: Option<String>,

    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
#[command(propagate_version = true)]
pub enum Subcommand {
    /// Print details for a specific crate.
    ///
    /// Endpoint: /api/v1/crates/:name
    Crate(CrateOpts),
    /// Search for crates.
    ///
    /// Endpoint: /api/v1/crates
    Search(SearchOpts),
    /// Publish a crate version. Requires an API token.
    ///
    /// Endpoint: /api/v1/crates/new
    Publish(PublishOpts),
    /// List, add, or remove the owners of a crate.
    ///
    /// Endpoint: /api/v1/crates/:name/owners
    #[command(subcommand)]
    Owner(OwnerCommand),
    /// Yank a crate version. Requires an API token.
    ///
    /// Endpoint: /api/v1/crates/:name/:version/yank
    Yank(VersionOpts),
    /// Undo the yank of a crate version. Requires an API token.
    ///
    /// Endpoint: /api/v1/crates/:name/:version/unyank
    Unyank(VersionOpts),
    /// Print the download URL of a crate version.
    ///
    /// Endpoint: /api/v1/crates/:name/:version/download
    Download(VersionOpts),
    /// Print the published versions of a crate.
    ///
    /// Endpoint: /api/v1/crates/:name/versions
    Versions(VersionsOpts),
    /// Print info of a specific crate version.
    ///
    /// Endpoint: /api/v1/crates/:name/:version
    Version(VersionOpts),
    /// Print the dependencies of a crate version.
    ///
    /// Endpoint: /api/v1/crates/:name/:version/dependencies
    Dependencies(VersionOpts),
    /// Print the reverse deps of a crate.
    ///
    /// Endpoint: /api/v1/crates/:name/reverse_dependencies
    ReverseDependencies(ReverseDependenciesOpts),
    /// Print the download counts of a crate.
    ///
    /// Endpoint: /api/v1/crates/:name/downloads
    Downloads(DownloadsOpts),
    /// Print the download counts of a crate version.
    ///
    /// Endpoint: /api/v1/crates/:name/:version/downloads
    VersionDownloads(VersionDownloadsOpts),
    /// Print the README URL of a crate version.
    ///
    /// Endpoint: /api/v1/crates/:name/:version/readme
    Readme(VersionOpts),
    /// Print the authors of a crate version. Deprecated, always empty.
    ///
    /// Requires the `deprecated` feature.
    ///
    /// Endpoint: /api/v1/crates/:name/:version/authors
    #[cfg(feature = "deprecated")]
    Authors(VersionOpts),
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Crate options")]
pub struct CrateOpts {
    #[arg(value_name = "name")]
    name: String,
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Search options")]
pub struct SearchOpts {
    #[arg(value_name = "query")]
    query: Option<String>,

    /// One of: alphabetical, relevance, downloads, recent-downloads, recent-updates, or new.
    #[arg(long)]
    sort: Option<Sort>,

    #[arg(long)]
    page: Option<NonZeroU32>,

    #[arg(long)]
    per_page: Option<NonZeroU32>,
}

impl SearchOpts {
    fn endpoint(&self) -> Search<'_> {
        let mut search = Search::new();

        if let Some(query) = &self.query {
            search = search.with_query(Cow::Borrowed(query.as_str()));
        }
        if let Some(sort) = self.sort {
            search = search.with_sort(sort);
        }
        if let Some(page) = self.page {
            search = search.with_pagination(Pagination::Page(page));
        }
        if let Some(per_page) = self.per_page {
            search = search.with_per_page(per_page);
        }

        search
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Publish options")]
pub struct PublishOpts {
    /// Path to a JSON file with the metadata of the crate version.
    #[arg(long, value_name = "path")]
    metadata: PathBuf,

    /// Path to the `.crate` file, as created by `cargo package`.
    #[arg(value_name = "crate-file")]
    crate_file: PathBuf,
}

impl PublishOpts {
    fn endpoint(&self) -> anyhow::Result<Publish> {
        let metadata = std::fs::read(&self.metadata).with_context(|| {
            format!(
                "Unable to read the metadata file '{}'",
                self.metadata.display()
            )
        })?;
        let metadata: PublishMetadata = serde_json::from_slice(&metadata).with_context(|| {
            format!(
                "Unable to parse the metadata file '{}'",
                self.metadata.display()
            )
        })?;
        let crate_file = std::fs::read(&self.crate_file).with_context(|| {
            format!(
                "Unable to read the crate file '{}'",
                self.crate_file.display()
            )
        })?;

        Ok(Publish::new(&metadata, &crate_file)?)
    }
}

#[derive(clap::Subcommand)]
pub enum OwnerCommand {
    /// List the owners of a crate.
    List(CrateOpts),
    /// List the users which own a crate.
    ///
    /// Endpoint: /api/v1/crates/:name/owner_user
    Users(CrateOpts),
    /// List the teams which own a crate.
    ///
    /// Endpoint: /api/v1/crates/:name/owner_team
    Teams(CrateOpts),
    /// Invite users or teams to become owners of a crate. Requires an API token.
    Add(OwnersOpts),
    /// Remove users or teams as owners of a crate. Requires an API token.
    Remove(OwnersOpts),
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Owner options")]
pub struct OwnersOpts {
    #[arg(value_name = "name")]
    name: String,

    /// Login of a user or team, e.g. `username`, `github:username`, or `github:org:team`.
    #[arg(value_name = "owner", required = true)]
    owners: Vec<String>,
}

impl OwnersOpts {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.name.as_str())
    }

    fn owners(&self) -> Vec<Cow<'_, str>> {
        self.owners
            .iter()
            .map(|owner| Cow::Borrowed(owner.as_str()))
            .collect()
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Version options")]
pub struct VersionOpts {
    #[arg(value_name = "name")]
    name: String,

    // Not named `version`, because that clashes with the `--version` flag.
    #[arg(value_name = "version")]
    crate_version: semver::Version,
}

impl VersionOpts {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.name.as_str())
    }

    fn version(&self) -> Cow<'_, semver::Version> {
        Cow::Borrowed(&self.crate_version)
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Versions options")]
pub struct VersionsOpts {
    #[arg(value_name = "name")]
    name: String,

    /// One of: date, or semver.
    #[arg(long)]
    sort: Option<VersionSort>,

    #[arg(long)]
    page: Option<NonZeroU32>,

    #[arg(long)]
    per_page: Option<NonZeroU32>,
}

impl VersionsOpts {
    fn endpoint(&self) -> Versions<'_> {
        let mut versions = Versions::new(Cow::Borrowed(self.name.as_str()));

        if let Some(sort) = self.sort {
            versions = versions.with_sort(sort);
        }
        if let Some(page) = self.page {
            versions = versions.with_pagination(Pagination::Page(page));
        }
        if let Some(per_page) = self.per_page {
            versions = versions.with_per_page(per_page);
        }

        versions
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Reverse dependencies options")]
pub struct ReverseDependenciesOpts {
    #[arg(value_name = "name")]
    name: String,

    #[arg(long)]
    page: Option<NonZeroU32>,

    #[arg(long)]
    per_page: Option<NonZeroU32>,
}

impl ReverseDependenciesOpts {
    fn endpoint(&self) -> ReverseDependencies<'_> {
        let mut reverse_dependencies = ReverseDependencies::new(Cow::Borrowed(self.name.as_str()));

        if let Some(page) = self.page {
            reverse_dependencies = reverse_dependencies.with_pagination(Pagination::Page(page));
        }
        if let Some(per_page) = self.per_page {
            reverse_dependencies = reverse_dependencies.with_per_page(per_page);
        }

        reverse_dependencies
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Downloads options")]
pub struct DownloadsOpts {
    #[arg(value_name = "name")]
    name: String,

    /// Include the metadata of the versions.
    #[arg(long)]
    include_versions: bool,
}

impl DownloadsOpts {
    fn endpoint(&self) -> CrateDownloads<'_> {
        let downloads = CrateDownloads::new(Cow::Borrowed(self.name.as_str()));

        if self.include_versions {
            downloads.with_versions()
        } else {
            downloads
        }
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Version downloads options")]
pub struct VersionDownloadsOpts {
    #[command(flatten)]
    version: VersionOpts,

    /// Only count downloads before this date (YYYY-MM-DD).
    #[arg(long)]
    before_date: Option<Date>,
}

impl VersionDownloadsOpts {
    fn endpoint(&self) -> VersionDownloads<'_> {
        let downloads = VersionDownloads::new(self.version.name(), self.version.version());

        match self.before_date {
            Some(before_date) => downloads.with_before_date(before_date),
            None => downloads,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        CargoCli::command().debug_assert();
    }
}
