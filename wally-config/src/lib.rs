use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Context;
use facet::Facet;
use facet_kdl as kdl;

#[derive(Facet)]
pub struct GeneralConfig {
    /// The directory to save wallpapers to
    pub output_dir: PathBuf,
}

#[derive(Facet)]
pub struct Config {
    #[facet(kdl::child)]
    pub general: GeneralConfig,
}

pub fn read_config(config_path: &Path) -> anyhow::Result<Config> {
    let mut file = File::open(config_path).context(format!(
        "config file does not exist at path {}",
        config_path.display()
    ))?;
    let mut content = String::new();
    File::read_to_string(&mut file, &mut content)?;
    Ok(facet_kdl::from_str(&content)?)
}
