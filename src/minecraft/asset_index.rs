use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::minecraft::Artifact;
use crate::minecraft::{Resource, ResourceType, Resources};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub hash: String,
    pub size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetIndex {
    pub objects: HashMap<String, Asset>,
}

impl Resources for AssetIndex {
    fn resources(&self) -> Vec<Resource> {
        self.objects
            .keys()
            .into_iter()
            .map(|asset_name| {
                let asset = &self.objects[asset_name];
                let asset_artifact = Artifact::from(asset.clone());
                Resource {
                    resource_type: ResourceType::Asset,
                    name: asset_name.clone(),
                    artifacts: vec![asset_artifact],
                }
            })
            .collect()
    }
}
