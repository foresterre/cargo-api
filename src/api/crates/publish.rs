use crate::api::{Authentication, Body, BodyError, Endpoint};
use semver::Version;
use std::borrow::Cow;
use std::collections::BTreeMap;

/// API to publish a new crate, or update it by publish a new version of an existing crate.
#[derive(Clone, Debug)]
pub struct Publish {
    body: Vec<u8>,
}

impl Publish {
    /// `crate_file` is the `.crate` file (a gzipped tarball), as created by `cargo package`.
    pub fn new(metadata: &PublishMetadata, crate_file: &[u8]) -> Result<Self, PublishError> {
        let metadata =
            serde_json::to_vec(metadata).map_err(|error| PublishError::Metadata { error })?;
        let metadata_len = length_prefix(metadata.len()).ok_or(PublishError::MetadataTooLarge {
            len: metadata.len(),
        })?;
        let crate_file_len =
            length_prefix(crate_file.len()).ok_or(PublishError::CrateFileTooLarge {
                len: crate_file.len(),
            })?;

        let mut body = Vec::with_capacity(4 + metadata.len() + 4 + crate_file.len());
        body.extend_from_slice(&metadata_len);
        body.extend_from_slice(&metadata);
        body.extend_from_slice(&crate_file_len);
        body.extend_from_slice(crate_file);

        Ok(Self { body })
    }
}

impl Endpoint for Publish {
    fn method(&self) -> http::Method {
        http::Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        Cow::Borrowed("v1/crates/new")
    }

    fn body(&self) -> Result<Option<Body>, BodyError> {
        Ok(Some(Body::new(
            "application/octet-stream",
            self.body.clone(),
        )))
    }

    fn authentication(&self) -> Authentication {
        Authentication::Required
    }
}

/// Metadata of crate to publish
///
/// See <https://doc.rust-lang.org/cargo/reference/registry-web-api.html#publish>.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PublishMetadata {
    pub name: String,
    pub vers: Version,
    #[serde(default)]
    pub deps: Vec<Dependency>,
    #[serde(default)]
    pub features: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    /// The content of the readme file.
    pub readme: Option<String>,
    /// The path to the readme file, relative to the root of the package.
    pub readme_file: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    /// An SPDX 2.3 license expression.
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    /// Deprecated by cargo, which always sends an empty map.
    #[serde(default)]
    pub badges: BTreeMap<String, BTreeMap<String, String>>,
    /// The name of the native library the package links to.
    pub links: Option<String>,
    pub rust_version: Option<String>,
}

impl PublishMetadata {
    pub fn new(name: String, vers: Version) -> Self {
        Self {
            name,
            vers,
            deps: Vec::new(),
            features: BTreeMap::new(),
            authors: Vec::new(),
            description: None,
            documentation: None,
            homepage: None,
            readme: None,
            readme_file: None,
            keywords: Vec::new(),
            categories: Vec::new(),
            license: None,
            license_file: None,
            repository: None,
            badges: BTreeMap::new(),
            links: None,
            rust_version: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    /// The name of the package
    ///
    /// Differs from the name in `Cargo.toml` when the dependency is renamed
    ///
    /// see also: `explicit_name_in_toml`.
    pub name: String,
    pub version_req: String,
    #[serde(default)]
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    /// The target platform, e.g. `cfg(windows)` or `x86_64-pc-windows-msvc`.
    pub target: Option<String>,
    pub kind: DependencyKind,
    /// The URL of the index of the registry
    pub registry: Option<String>,
    pub explicit_name_in_toml: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    Normal,
    Dev,
    Build,
}

/// The body consists of the metadata and the `.crate` file, each prefixed with its
/// length as a 32-bit little endian integer.
fn length_prefix(len: usize) -> Option<[u8; 4]> {
    u32::try_from(len).ok().map(u32::to_le_bytes)
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PublishError {
    #[error("Unable to serialize the publish metadata: {}", error)]
    Metadata { error: serde_json::Error },
    #[error(
        "The publish metadata is {} bytes, which exceeds the maximum of 4 GiB",
        len
    )]
    MetadataTooLarge { len: usize },
    #[error("The crate file is {} bytes, which exceeds the maximum of 4 GiB", len)]
    CrateFileTooLarge { len: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_layout() {
        let metadata = PublishMetadata::new("serde".into(), Version::new(1, 0, 0));
        let json = serde_json::to_vec(&metadata).unwrap();
        let crate_file = b"crate file";

        let body = Publish::new(&metadata, crate_file)
            .unwrap()
            .body()
            .unwrap()
            .unwrap()
            .into_bytes();

        let (json_len, rest) = body.split_at(4);
        let json_len = u32::from_le_bytes(json_len.try_into().unwrap()) as usize;
        let (actual_json, rest) = rest.split_at(json_len);
        let (crate_len, actual_crate_file) = rest.split_at(4);

        assert_eq!(actual_json, json);
        assert_eq!(u32::from_le_bytes(crate_len.try_into().unwrap()), 10);
        assert_eq!(actual_crate_file, crate_file);
    }

    #[yare::parameterized(
        zero = { 0, Some([0, 0, 0, 0]) },
        little_endian = { 0x0102_0304, Some([4, 3, 2, 1]) },
        max = { u32::MAX as usize, Some([0xff, 0xff, 0xff, 0xff]) },
    )]
    fn length_prefix_fits(len: usize, expected: Option<[u8; 4]>) {
        assert_eq!(length_prefix(len), expected);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn length_prefix_too_large() {
        assert_eq!(length_prefix(u32::MAX as usize + 1), None);
    }

    #[test]
    fn metadata_matches_cargo_format() {
        let mut metadata = PublishMetadata::new("foo".into(), Version::new(0, 1, 0));
        metadata.deps.push(Dependency {
            name: "rand".into(),
            version_req: "^0.6".into(),
            features: vec!["i128_support".into()],
            optional: false,
            default_features: true,
            target: None,
            kind: DependencyKind::Normal,
            registry: None,
            explicit_name_in_toml: None,
        });

        let value = serde_json::to_value(&metadata).unwrap();

        assert_eq!(value["vers"], "0.1.0");
        assert_eq!(value["deps"][0]["kind"], "normal");
        assert_eq!(value["license"], serde_json::Value::Null);
        assert_eq!(value["badges"], serde_json::json!({}));
    }
}
