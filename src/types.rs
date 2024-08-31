use jiff::civil::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Crate {
    pub badges: Vec<String>,
    pub categories: Vec<String>,
    pub created_at: DateTime,
    pub description: String,
    pub documentation: Option<String>,
    pub downloads: u32,
    pub exact_match: bool,
    pub homepage: Option<String>,
    pub id: String,
    pub keywords: Vec<String>,
    pub links: CrateLinks,
    pub max_stable_version: String,
    pub max_version: String,
    pub name: String,
    pub newest_version: String,
    pub recent_downloads: u32,
    pub repository: Option<String>,
    pub updated_at: DateTime,
    pub versions: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrateLinks {
    pub owner_team: String,
    pub owner_user: String,
    pub owners: String,
    pub reverse_dependencies: String,
    pub version_downloads: String,
    pub versions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub audit_actions: Vec<AuditAction>,
    pub bin_names: Vec<String>,
    pub checksum: String,
    pub crate_name: String,
    pub crate_size: u32,
    pub created_at: DateTime,
    pub dl_path: String,
    pub downloads: u32,
    pub features: HashMap<String, Vec<String>>,
    pub has_lib: bool,
    pub id: u32,
    pub lib_links: Option<String>,
    pub license: String,
    pub links: VersionLinks,
    pub num: String,
    pub published_by: User,
    pub readme_path: String,
    pub rust_version: Option<String>,
    pub updated_at: DateTime,
    pub yanked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditAction {
    pub action: String,
    pub time: DateTime,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub avatar: String,
    pub id: u32,
    pub login: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionLinks {
    pub authors: String,
    pub dependencies: String,
    pub version_downloads: String,
}
