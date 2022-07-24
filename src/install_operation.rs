use crate::minecraft;
use crate::minecraft::Resources;
use futures::future::try_join_all;
use std::path::{Path, PathBuf};

#[allow(unused)]
pub struct InstallOperation {
    version: minecraft::Version,
    asset_index: minecraft::AssetIndex,
    dir: PathBuf,
}

#[allow(unused)]
impl InstallOperation {
    pub fn new(ver: minecraft::Version, asset_index: minecraft::AssetIndex, dir: PathBuf) -> Self {
        Self {
            version: ver,
            asset_index,
            dir,
        }
    }

    async fn install_libs(&self, dst: &Path) -> anyhow::Result<()> {
        let resources = self.version.resources();
        try_join_all(resources.iter().map(|resource| resource.install_to(dst))).await?;
        Ok(())
    }

    async fn install_assets(&self, dst: &Path) -> anyhow::Result<()> {
        let resources = self.asset_index.resources();
        try_join_all(resources.iter().map(|resource| resource.install_to(dst))).await?;
        Ok(())
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        let dst = PathBuf::from(&self.dir).join(&self.version.id);
        let lib_dir = dst.join("libraries");
        std::fs::create_dir_all(&lib_dir).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });

        println!("Installing libraries to {:?}", &lib_dir);
        self.install_libs(&lib_dir).await?;

        let asset_dir = dst.join("assets");
        std::fs::create_dir_all(&asset_dir).unwrap_or_else(|why| println!("! {:?}", why.kind()));

        println!("Installing assets to {:?}", &asset_dir);
        self.install_assets(&asset_dir).await?;

        Ok(())
    }
}
