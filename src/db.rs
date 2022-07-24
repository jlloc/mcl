use crate::minecraft;
use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct Installation {
    pub name: String,
    pub version: String,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
    pub path: PathBuf,
    pub lib_dir: PathBuf,
}

impl Installation {
    pub fn new(version: &minecraft::Version, installation_dir: &Path) -> Self {
        let name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let path = installation_dir.join(&name);
        let lib_dir = path.join("libraries");

        Self {
            name,
            path,
            lib_dir,
            version: version.id.clone(),
            created_at: chrono::DateTime::from(SystemTime::now()),
            updated_at: None,
        }
    }

    pub fn ensure_dirs_exist(&self) -> anyhow::Result<()> {
        let dirs = vec![&self.path, &self.lib_dir];
        for dir in dirs.into_iter() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Db {
    pub installations: Vec<Installation>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            installations: Vec::new(),
        }
    }
}

#[allow(unused)]
pub struct JsonFileDb {
    pub db: Db,
    handle: File,
}

#[allow(unused)]
impl JsonFileDb {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        match File::open(path) {
            Ok(mut handle) => {
                let mut buf = String::new();
                handle.read_to_string(&mut buf)?;
                let db = serde_json::from_str(&buf)?;
                Ok(Self { handle, db })
            }
            Err(err) => {
                println!(
                    "Error opening database at {:?} ({}), creating instead...",
                    path, err
                );
                let mut handle = File::create(path)?;
                Ok(Self {
                    handle,
                    db: Db::new(),
                })
            }
        }
    }

   pub fn commit(&mut self) -> anyhow::Result<()> {
        let buf = serde_json::to_vec(&self.db)?;
        self.handle.write_all(&buf)?;
        Ok(())
    }
}
