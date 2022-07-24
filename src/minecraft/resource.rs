use crate::checksum::{Checksum, ChecksumVerificationError};
use crate::minecraft::{Asset, Library, OSName};
use crate::Fetch;
use anyhow::ensure;
use async_trait::async_trait;
use bytes::Bytes;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const RESOURCE_URL: &str = "https://resources.download.minecraft.net/";
const RESOURCE_PATH: &str = ".minecraft/assets/objects/";

#[derive(Debug)]
pub enum ResourceType {
    Asset,
    Library,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artifact {
    pub sha1: String,
    pub size: u32,
    pub url: String,
    pub path: Option<PathBuf>,
}

impl From<Asset> for Artifact {
    fn from(asset: Asset) -> Self {
        let mut resource_url = url::Url::parse(RESOURCE_URL).unwrap();
        resource_url
            .path_segments_mut()
            .unwrap()
            .push(&asset.hash[..2])
            .push(&asset.hash);

        let path = PathBuf::from_str(RESOURCE_PATH)
            .unwrap()
            .join(&asset.hash[..2])
            .join(&asset.hash);

        Self {
            sha1: asset.hash,
            size: asset.size,
            url: resource_url.to_string(),
            path: Some(path),
        }
    }
}

#[async_trait]
impl Fetch<Bytes> for Artifact {
    #[allow(unused)]
    fn fetch(&self) -> anyhow::Result<Bytes> {
        Ok(reqwest::blocking::get(&self.url)?.bytes()?)
    }

    async fn fetch_async(&self) -> anyhow::Result<Bytes> {
        let resp = reqwest::get(&self.url).await?;
        let status = resp.status();
        if !status.is_success() {
            let msg = format!("Received non-success HTTP status code ({})", status);
            return Err(anyhow::Error::msg(msg));
        }
        Ok(resp.bytes().await?)
    }
}

impl Checksum for Artifact {
    fn verify_checksum(&self, v: &Bytes) -> anyhow::Result<()> {
        let mut hasher = Sha1::new();
        hasher.update(v);
        let res = hasher.finalize();
        let sha1_hex = hex::decode(&self.sha1)?;
        ensure!(res[..].eq(&sha1_hex), ChecksumVerificationError);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub name: String,
    pub artifacts: Vec<Artifact>,
}

impl From<Library> for Resource {
    fn from(lib: Library) -> Self {
        let mut artifacts = Vec::new();

        if let Some(main_artifact) = lib.downloads.artifact {
            artifacts.push(main_artifact)
        }

        if let Some(natives) = lib.natives {
            let os_name = OSName::from_str(env::consts::OS).unwrap();
            if let Some(classifier) = natives.get(&os_name) {
                let dep_lib = &lib.downloads.classifiers.unwrap()[classifier];
                artifacts.push(dep_lib.clone())
            }
        }

        Self {
            resource_type: ResourceType::Library,
            name: lib.name,
            artifacts,
        }
    }
}

impl Resource {
    pub async fn install_to(&self, dst: &Path) -> anyhow::Result<()> {
        println!("=> {}", &self.name);
        for artifact in self.artifacts.iter() {
            let p = if let Some(p) = &artifact.path {
                p.clone()
            } else {
                PathBuf::from(&self.name)
            };
            println!("  -> {:?}", &p);
            let dst_path = dst.join(p);
            std::fs::create_dir_all(dst_path.parent().unwrap())?;
            std::fs::write(dst_path, artifact.fetch_async().await?)?;
        }
        Ok(())
    }
}

impl From<Asset> for Resource {
    fn from(asset: Asset) -> Self {
        Self {
            name: asset.hash.clone(),
            resource_type: ResourceType::Asset,
            artifacts: vec![Artifact::from(asset)],
        }
    }
}

pub trait Resources {
    fn resources(&self) -> Vec<Resource>;
}
