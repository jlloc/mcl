use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Config {
    pub db_path: PathBuf,
    pub cache_path: PathBuf,
    pub cache_expiry_days: u32,
}

impl Config {
    pub fn new() -> Self {
        let db_path = dirs::data_dir().unwrap().join("mc-installer");
        Self {
            db_path,
            cache_path: dirs::cache_dir().unwrap().join("mc-installer"),
            cache_expiry_days: 5,
        }
    }

    pub fn prepare_dirs(&self) -> anyhow::Result<()> {
        let dirs = vec![&self.db_path, &self.cache_path];
        for dir in dirs.into_iter() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}
