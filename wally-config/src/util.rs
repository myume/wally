use std::path::PathBuf;

use facet::Facet;
use facet_kdl as kdl;

#[derive(Facet)]
pub struct KdlPath {
    #[facet(kdl::argument)]
    pub value: PathBuf,
}

#[derive(Facet)]
pub struct KdlBool {
    #[facet(kdl::argument)]
    pub value: bool,
}
