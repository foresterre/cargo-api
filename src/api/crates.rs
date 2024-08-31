use crate::types::{Crate, Version};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CrateGet {
    pub categories: Vec<String>,
    #[serde(rename = "crate")]
    pub crate_: Crate,
    pub keywords: Vec<String>,
    pub versions: Vec<Version>,
}
