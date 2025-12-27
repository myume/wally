use std::path::PathBuf;

use facet::Facet;
use facet_kdl as kdl;

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct GeneralConfig {
    /// The directory to save wallpapers to
    #[facet(kdl::child)]
    pub output_dir: KdlPath,

    /// command to set wallpaper
    #[facet(kdl::child)]
    pub set_command: Command,
}

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct Command {
    #[facet(kdl::argument)]
    pub command: String,
}

#[derive(Facet)]
pub struct KdlPath {
    #[facet(kdl::argument)]
    pub value: PathBuf,
}
