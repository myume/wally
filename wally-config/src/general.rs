use std::path::PathBuf;

use facet::Facet;
use facet_kdl as kdl;

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct GeneralConfig {
    /// The directory to save wallpapers to
    #[facet(kdl::child)]
    pub output_dir: KdlPath,
}

#[derive(Facet)]
pub struct KdlPath {
    #[facet(kdl::argument)]
    pub value: PathBuf,
}
