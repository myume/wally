use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

const METADATA_FILE_NAME: &str = ".wally-meta";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub active_wallpaper: PathBuf,
}

impl Metadata {
    pub fn read(path: &Path) -> anyhow::Result<Metadata> {
        let Ok(mut file) = File::open(path.join(METADATA_FILE_NAME)) else {
            eprintln!("Metadata file not found, creating default");
            return Ok(Metadata::default());
        };
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .context("failed to read metadata file")?;

        Ok(serde_json::from_str(&file_content)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let mut file = File::create(path.join(METADATA_FILE_NAME))?;
        let file_content = serde_json::to_vec(self)?;
        file.write_all(&file_content)?;

        Ok(())
    }
}
