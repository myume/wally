use facet::Facet;
use facet_kdl as kdl;

use crate::util::KdlBool;

#[derive(Facet)]
#[facet(deny_unknown_fields)]
pub struct KonachanConfig {
    #[facet(kdl::child)]
    pub explicit: KdlBool,
}
