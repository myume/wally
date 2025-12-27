use facet::Facet;
use facet_kdl as kdl;

use crate::util::KdlBool;

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct WallhavenConfig {
    #[facet(kdl::child)]
    pub categories: WallhavenCategories,
}

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct WallhavenCategories {
    #[facet(kdl::child)]
    pub general: KdlBool,

    #[facet(kdl::child)]
    pub anime: KdlBool,

    #[facet(kdl::child)]
    pub people: KdlBool,
}
