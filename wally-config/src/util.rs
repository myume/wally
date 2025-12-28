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

#[derive(Facet)]
pub struct Kdlu32 {
    #[facet(kdl::argument)]
    pub value: u32,
}
