use facet::Facet;
use facet_kdl as kdl;

use crate::util::{KdlPath, Kdlu32};

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct GeneralConfig {
    /// The directory to save wallpapers to
    #[facet(kdl::child)]
    pub output_dir: KdlPath,

    /// command to set wallpaper
    #[facet(kdl::child)]
    pub set_command: Command,

    /// maximum number of wallpapers to keep downloaded in the output dir
    #[facet(kdl::child)]
    pub max_downloaded: Kdlu32,
}

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct Command {
    #[facet(kdl::argument)]
    pub command: String,
}
