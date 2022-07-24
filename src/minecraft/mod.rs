mod asset_index;
mod resource;
mod version;
mod version_manifest;

pub use asset_index::*;
pub use resource::*;
pub use version::*;
pub use version_manifest::*;

use async_trait::async_trait;

#[async_trait]
pub trait Fetch<T> {
    fn fetch(&self) -> anyhow::Result<T>;
    async fn fetch_async(&self) -> anyhow::Result<T>;
}
