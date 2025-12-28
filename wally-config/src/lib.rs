use std::{fs::File, io::Read, path::Path};

use facet::Facet;
use facet_kdl as kdl;
use miette::{IntoDiagnostic, WrapErr};

pub mod general;
pub mod konachan;
pub mod util;
pub mod wallhaven;

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct Config {
    #[facet(kdl::child)]
    pub general: general::GeneralConfig,

    #[facet(kdl::child)]
    pub wallhaven: wallhaven::WallhavenConfig,

    #[facet(kdl::child)]
    pub konachan: konachan::KonachanConfig,
}

pub fn read_config(config_path: &Path) -> miette::Result<Config> {
    let mut file = File::open(config_path).into_diagnostic().wrap_err(format!(
        "config file does not exist at path {}",
        config_path.display()
    ))?;
    let mut content = String::new();
    File::read_to_string(&mut file, &mut content)
        .into_diagnostic()
        .wrap_err("could not read config file")?;
    facet_kdl::from_str(&content).into_diagnostic()
}
