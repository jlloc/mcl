use crate::minecraft::{Artifact, ReleaseType, Resource};
use crate::minecraft::{AssetIndex, Fetch, ResourceType, Resources};
use async_trait::async_trait;
use bytes::Buf;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct UnknownOSError;

impl fmt::Display for UnknownOSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown os literal")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RuleAction {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "disallow")]
    Disallow,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum OSName {
    #[serde(rename = "osx")]
    Osx,
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "windows")]
    Windows,
}

impl FromStr for OSName {
    type Err = UnknownOSError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "osx" | "macos" => Ok(OSName::Osx),
            "linux" => Ok(OSName::Linux),
            "windows" => Ok(OSName::Windows),
            _ => Err(UnknownOSError),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleOS {
    name: OSName,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    action: RuleAction,
    os: RuleOS,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LibraryExtract {
    pub exclude: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Library {
    pub name: String,
    pub downloads: LibraryDownloads,
    pub extract: Option<LibraryExtract>,
    pub natives: Option<HashMap<OSName, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetIndexArtifact {
    pub id: String,
    pub sha1: String,
    pub size: u32,
    pub url: String,

    #[serde(rename = "totalSize")]
    pub total_size: u32,
}

#[async_trait]
impl Fetch<AssetIndex> for AssetIndexArtifact {
    fn fetch(&self) -> anyhow::Result<AssetIndex> {
        let res = reqwest::blocking::get(&self.url)?;
        Ok(serde_json::from_reader(res)?)
    }

    async fn fetch_async(&self) -> anyhow::Result<AssetIndex> {
        let client = reqwest::Client::new();
        let res = client
            .get(&self.url)
            .header(USER_AGENT, "curl/7.79.1")
            .header(ACCEPT, "*/*")
            .send()
            .await?;

        // let res = reqwest::get(&self.url).await?;
        Ok(serde_json::from_reader(res.bytes().await?.reader())?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JavaVersion {
    pub component: String,

    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub id: String,

    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexArtifact,
    pub assets: String,

    #[serde(rename = "complianceLevel")]
    pub compliance_level: u32,
    pub downloads: HashMap<String, Artifact>,

    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,

    #[serde(rename = "mainClass")]
    pub main_class: String,

    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub time: String,

    #[serde(rename = "type")]
    pub release_type: ReleaseType,
}

impl Resources for Version {
    fn resources(&self) -> Vec<Resource> {
        let mut resources: Vec<Resource> = self
            .downloads
            .keys()
            .map(|id| {
                let artifact = &self.downloads[id];
                Resource {
                    resource_type: ResourceType::Library,
                    name: format!("{}.jar", id),
                    artifacts: vec![artifact.clone()],
                }
            })
            .collect();

        self.libraries
            .iter()
            .for_each(|lib| resources.push(Resource::from(lib.clone())));
        resources
    }
}
