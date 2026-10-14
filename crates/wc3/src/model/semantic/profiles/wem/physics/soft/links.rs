use super::*;
use crate::model::semantic::{Error, Link, MeshId, ValidateLinks};
fn finite(values: impl IntoIterator<Item = f32>, visitor: &mut dyn FnMut(Link)) {
    if values.into_iter().any(|value| !value.is_finite()) {
        visitor(Link::InvalidValue);
    }
}
impl ValidateLinks for SurfacePoint {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.mesh.visit_links(visitor);
        finite(self.weights.into_iter().chain([self.radius]), visitor);
    }
}
impl ValidateLinks for SoftPart {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.mesh.visit_links(visitor);
        self.material.visit_links(visitor);
    }
}
impl ValidateLinks for Inflate {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        finite([self.round, self.rate, self.depth], visitor);
    }
}
impl ValidateLinks for Bulge<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for node in self.chain.iter() {
            node.visit_links(visitor);
        }
        finite(
            [self.rise, self.fall]
                .into_iter()
                .chain(self.line.iter().flatten()),
            visitor,
        );
    }
}
impl ValidateLinks for Impulse {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.place.visit_links(visitor);
        finite(
            self.direction.into_iter().flatten().chain([self.strength]),
            visitor,
        );
    }
}
impl ValidateLinks for Jiggle<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        finite(
            [
                self.looseness,
                self.bounce,
                self.weight,
                self.hold_shape,
                self.keep_volume,
                self.limit,
            ],
            visitor,
        );
        let mut omitted = false;
        for impulse in self.impulses.iter() {
            impulse.visit_links(visitor);
            if impulse.direction.is_none() {
                omitted = true;
            } else if omitted {
                visitor(Link::InvalidValue);
            }
        }
    }
}
impl ValidateLinks for Wobble {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.axis_bone.visit_links(visitor);
        finite([self.speed, self.size, self.overlap], visitor);
        if self.axis == Axis::Bone && self.axis_bone.is_none() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Blobify {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        finite([self.roundness, self.swell], visitor);
    }
}
impl ValidateLinks for DripSite {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.place.visit_links(visitor);
        finite([self.thickness], visitor);
    }
}
impl ValidateLinks for Strand {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.nodes.visit_links(visitor);
        self.drop.visit_links(visitor);
        finite(self.position.into_iter().chain([self.width]), visitor);
    }
}
impl ValidateLinks for Drip<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.material.visit_links(visitor);
        finite(
            [
                self.overhang,
                self.spacing,
                self.length,
                self.speed,
                self.thickness,
                self.neck_width,
                self.tip,
                self.stagger,
                self.sway,
                self.pull,
                self.pinch_every,
            ],
            visitor,
        );
        for site in self.authored_sites.iter() {
            site.visit_links(visitor);
        }
        for strand in self.strands.iter() {
            strand.visit_links(visitor);
        }
    }
}
impl ValidateLinks for Squash {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.axis_bone.visit_links(visitor);
        finite(
            [
                self.stretch,
                self.squash,
                self.turning,
                self.drag,
                self.wind_up,
                self.smoothing,
            ],
            visitor,
        );
        if self.axis == Axis::Bone && self.axis_bone.is_none() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Melt {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        finite(
            [self.solidity, self.floor, self.puddle, self.uneven],
            visitor,
        );
    }
}
impl ValidateLinks for Ripple<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        finite(
            [
                self.wavelength,
                self.speed,
                self.envelope,
                self.fade_distance,
                self.fade_time,
            ],
            visitor,
        );
    }
}
impl ValidateLinks for Settings<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.inflate.visit_links(visitor);
        self.bulge.visit_links(visitor);
        self.jiggle.visit_links(visitor);
        self.wobble.visit_links(visitor);
        self.blobify.visit_links(visitor);
        self.drip.visit_links(visitor);
        self.squash.visit_links(visitor);
        self.melt.visit_links(visitor);
        self.ripple.visit_links(visitor);
    }
}
impl ValidateLinks for SoftEffect<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.place.visit_links(visitor);
        self.settings.visit_links(visitor);
        finite([self.amount], visitor);
    }
}
impl ValidateLinks for SoftLump {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.node.visit_links(visitor);
        self.carrier.visit_links(visitor);
        finite(
            self.anchor
                .into_iter()
                .chain(self.rest)
                .chain([self.depth, self.clear]),
            visitor,
        );
    }
}
impl SoftBody<'_> {
    /// Checks that pivots refer only to actual effects and riders in this definition.
    pub fn validate(&self) -> Result<(), Error> {
        for (_, lump) in self.lumps() {
            if lump.effect.is_some_and(|id| self.effect(id).is_none())
                || lump.rider.is_some_and(|id| self.rider(id).is_none())
            {
                return Err(Error::new(
                    "wem.soft.lump",
                    "pivot association belongs to another soft body",
                ));
            }
        }
        Ok(())
    }
}
impl ValidateLinks for SoftBody<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.validate().is_err() {
            visitor(Link::InvalidValue);
        }
        for mesh in self.meshes.iter() {
            mesh.visit_links(visitor);
        }
        self.drips.visit_links(visitor);
        for part in self.inside.iter() {
            part.visit_links(visitor);
        }
        for (_, part) in self.riders() {
            part.visit_links(visitor);
        }
        for (_, lump) in self.lumps() {
            lump.visit_links(visitor);
        }
        for (_, effect) in self.effects() {
            effect.visit_links(visitor);
        }
        finite([self.blend], visitor);
    }
}
impl SoftBody<'_> {
    /// Checks that every surface attachment addresses a polygon with a usable first triangle.
    ///
    /// The lookup belongs to the containing model's geometry arena, so deformation
    /// definitions retain actual mesh handles without borrowing another owner.
    pub fn validate_mesh_faces(
        &self,
        mut usable: impl FnMut(MeshId, u32) -> bool,
    ) -> Result<(), Error> {
        let mut check = |place: &SurfacePoint| {
            if usable(place.mesh, place.face) {
                Ok(())
            } else {
                Err(Error::new(
                    "wem.soft.attachment",
                    "surface point does not address a usable polygon",
                ))
            }
        };
        for (_, effect) in self.effects() {
            if let Some(place) = &effect.place {
                check(place)?;
            }
            if let Some(jiggle) = &effect.settings.jiggle {
                for impulse in jiggle.impulses.iter() {
                    check(&impulse.place)?;
                }
            }
            if let Some(drip) = &effect.settings.drip {
                for site in drip.authored_sites.iter() {
                    check(&site.place)?;
                }
            }
        }
        Ok(())
    }
}
