//! Membership checks for profile-local effect and cloth identities.
use super::ClothSystem;
use crate::model::Error;
impl ClothSystem<'_> {
    /// Checks links whose targets belong to this M3 profile rather than shared arenas.
    pub fn validate(&self) -> Result<(), Error> {
        for (index, cloth) in self.cloths.iter().enumerate() {
            for proxy in &cloth.proxies {
                if proxy.cloth.index() >= self.cloths.len() {
                    return Err(Error::new(
                        format!("m3.cloths[{index}].proxies"),
                        "cloth target does not belong to this profile",
                    ));
                }
            }
            if cloth.anchors.len() != cloth.sim_enabled.len() {
                return Err(Error::new(
                    format!("m3.cloths[{index}].vertices"),
                    "cloth anchor and particle-control cardinalities differ",
                ));
            }
        }
        Ok(())
    }
}
