use crate::minecraft::{Fetch, Version};
use async_trait::async_trait;
use bytes::Buf;
use serde::{Deserialize, Serialize};

const VERISON_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Serialize, Deserialize, Debug)]
pub enum ReleaseType {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "old_beta")]
    OldBeta,
    #[serde(rename = "old_alpha")]
    OldAlpha,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub release_type: ReleaseType,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: i32,
}

#[async_trait]
impl Fetch<Version> for VersionInfo {
    #[allow(unused)]
    fn fetch(&self) -> anyhow::Result<Version> {
        let res = reqwest::blocking::get(&self.url)?;
        let v = serde_json::from_reader(res)?;
        Ok(v)
    }

    #[allow(unused)]
    async fn fetch_async(&self) -> anyhow::Result<Version> {
        let res = reqwest::get(&self.url).await?;
        let v = serde_json::from_reader(res.bytes().await?.reader())?;
        Ok(v)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionLatestInfo {
    release: String,
    snapshot: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionManifest {
    pub latest: VersionLatestInfo,
    pub versions: Vec<VersionInfo>,
}

impl VersionManifest {
    #[allow(unused)]
    pub fn fetch() -> anyhow::Result<Self> {
        let res = reqwest::blocking::get(VERISON_MANIFEST_URL)?;
        Ok(serde_json::from_reader(res)?)
    }

    #[allow(unused)]
    pub async fn fetch_async() -> anyhow::Result<Self> {
        let res = reqwest::get(VERISON_MANIFEST_URL).await?.bytes().await?;
        Ok(serde_json::from_reader(res.reader())?)
    }
}
