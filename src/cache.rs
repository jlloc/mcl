use std::fs::File;
use std::io::Write;
use crate::minecraft::{VersionManifest};
use crate::Config;
use std::time::{Duration, SystemTime};

pub async fn read_manifest(config: &Config) -> anyhow::Result<VersionManifest> {
    let manifest_path = config.cache_path.join("version_manifest.json");
    let manifest = match File::open(&manifest_path) {
        Ok(mut f) => {
            let md = f.metadata()?;
            let expiry_secs = u64::from(&config.cache_expiry_days * 86400);
            if SystemTime::now().duration_since(md.modified()?)? > Duration::from_secs(expiry_secs) {
                println!(
                    "Local version manifest older than {} days, refreshing...",
                    &config.cache_expiry_days
                );
                let version_manifest = VersionManifest::fetch_async().await?;
                f.write_all(serde_json::to_vec(&version_manifest)?.as_slice())?;
                version_manifest
            } else {
                serde_json::from_slice(std::fs::read(&manifest_path)?.as_slice())?
            }
        },
        Err(err) => {
            println!("Error reading local version manifest ({}). Fetching...", err);
            let m = VersionManifest::fetch_async().await?;
            write_manifest(config, &m)?;
            m
        }
    };
    Ok(manifest)
}

pub fn write_manifest(config: &Config, manifest: &VersionManifest) -> anyhow::Result<()> {
    let manifest_path = config.cache_path.join("version_manifest.json");
    let buf = serde_json::to_vec(manifest)?;
    Ok(std::fs::write(manifest_path, buf)?)
}
