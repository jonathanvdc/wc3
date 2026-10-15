//! Relationship traversal for world-model authoring data.
use super::{Material, Node, WorldSystem};
use crate::model::{FeatureKind, Link, ValidateLinks};
impl ValidateLinks for Material<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.state.visit_links(visitor);
        self.textures.visit_links(visitor);
        if self
            .uv_scroll
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for WorldSystem<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.validate().is_err() {
            visitor(Link::InvalidValue);
        }
        for group in &self.groups {
            for instance in &group.instances {
                instance.visit_links(visitor);
            }
        }
        for fog in &self.fogs {
            fog.doodad_set.visit_links(visitor);
        }
        for ambient in &self.ambient_volumes {
            ambient.doodad_set.visit_links(visitor);
        }
        for set in &self.doodad_sets {
            for id in &set.members {
                visitor(Link::TypedFeature {
                    id: *id,
                    kind: FeatureKind::WmoDoodad,
                });
            }
        }
    }
}
impl Node<'_> {
    pub(crate) fn visit_profile_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Node::Attachment(doodad) => {
                if doodad
                    .appearance
                    .color_multiplier
                    .is_some_and(|value| !value.is_finite())
                {
                    visitor(Link::InvalidValue);
                }
                if let super::DoodadTint::Lit { light: id, .. } = doodad.appearance.tint {
                    visitor(Link::TypedFeature {
                        id,
                        kind: FeatureKind::WmoLight,
                    });
                }
            }
            Node::PointLight(light) => {
                if !light.intensity.is_finite()
                    || !light.scalar_control.is_finite()
                    || light.attenuation.iter().any(|value| !value.is_finite())
                {
                    visitor(Link::InvalidValue);
                }
            }
            Node::LightProgram(program) => {
                visitor(Link::TypedFeature {
                    id: program.light,
                    kind: FeatureKind::WmoPointLight,
                });
                if let Some(texture) = program.cookie {
                    visitor(Link::Texture(texture));
                }
                if !program.intensity.is_finite()
                    || program
                        .attenuation
                        .iter()
                        .chain(&program.orientation)
                        .chain(&program.flicker)
                        .any(|value| !value.is_finite())
                {
                    visitor(Link::InvalidValue);
                }
            }
            Node::Light(light) => {
                if light.attenuation.iter().any(|value| !value.is_finite()) {
                    visitor(Link::InvalidValue);
                }
                if light.directional_terms.as_ref().is_some_and(|terms| {
                    terms.iter().any(|(direction, scalar)| {
                        !scalar.is_finite() || direction.iter().any(|value| !value.is_finite())
                    })
                }) {
                    visitor(Link::InvalidValue);
                }
                light.color.visit_links(visitor);
                light.intensity.visit_links(visitor);
                if let Some(modern) = &light.modern {
                    modern.cookie.visit_links(visitor);
                    modern.doodad_set.visit_links(visitor);
                }
            }

            Node::Camera(unavailable)
            | Node::Particle(unavailable)
            | Node::Ribbon(unavailable)
            | Node::Event(unavailable)
            | Node::GeometryAnimation(unavailable)
            | Node::CollisionShape(unavailable)
            | Node::ParticleProgram(unavailable)
            | Node::FaceActor(unavailable)
            | Node::Extent(unavailable)
            | Node::ParticleCopy(unavailable)
            | Node::Projector(unavailable)
            | Node::Force(unavailable)
            | Node::Warp(unavailable)
            | Node::HitTest(unavailable)
            | Node::AttachmentVolume(unavailable)
            | Node::Billboard(unavailable)
            | Node::Turret(unavailable)
            | Node::IkJoint(unavailable)
            | Node::IkTwoJoint(unavailable)
            | Node::IkCcd(unavailable)
            | Node::OneBoneSolver(unavailable)
            | Node::ShadowBox(unavailable)
            | Node::ViewVolume(unavailable)
            | Node::AtlasParticles(unavailable) => match *unavailable {},
        }
    }
}

impl ValidateLinks for super::RegionControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.implicit_liquid.is_some() && !self.liquids.is_empty() {
            visitor(Link::InvalidValue);
        }
        if self
            .terrain_cut_planes
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        self.parent_or_first_child.visit_links(visitor);
        self.next_child.visit_links(visitor);
        for id in self
            .point_lights
            .iter()
            .chain(self.point_light_sets.iter().flatten())
        {
            visitor(Link::TypedFeature {
                id: *id,
                kind: FeatureKind::WmoPointLight,
            });
        }
        for id in self
            .point_programs
            .iter()
            .chain(self.point_program_sets.iter().flatten())
        {
            visitor(Link::TypedFeature {
                id: *id,
                kind: FeatureKind::WmoPointProgram,
            });
        }
        for id in &self.doodads {
            visitor(Link::TypedFeature {
                id: *id,
                kind: FeatureKind::WmoDoodad,
            });
        }
        for id in &self.lights {
            visitor(Link::TypedFeature {
                id: *id,
                kind: FeatureKind::WmoLight,
            });
        }
        for connection in &self.portals {
            connection.portal.visit_links(visitor);
            connection.neighbor.visit_links(visitor);
        }
        self.fogs.visit_links(visitor);
        self.ambients.visit_links(visitor);
        self.lightmap_images.visit_links(visitor);
        if self
            .auxiliary_geometry
            .as_ref()
            .is_some_and(|geometry| !geometry.values.iter().flatten().all(f32::is_finite))
        {
            visitor(Link::InvalidValue);
        }
        for liquid in &self.liquids {
            liquid.material.visit_links(visitor);
            let [x, y] = liquid.dimensions;
            if u64::from(x) * u64::from(y) != liquid.vertices.len() as u64
                || u64::from(x.saturating_sub(1)) * u64::from(y.saturating_sub(1))
                    != liquid.tiles.len() as u64
                || liquid
                    .vertices
                    .iter()
                    .any(|vertex| !vertex.height().is_finite())
            {
                visitor(Link::InvalidValue);
            }
        }
    }
}
macro_rules! member {
    ($type:ty,$variant:ident) => {
        impl ValidateLinks for $type {
            fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
                visitor(Link::SystemMember {
                    system: self.system,
                    member: crate::model::SystemMember::$variant(self.index),
                });
            }
        }
    };
}
member!(super::GroupId, WmoGroup);
member!(super::PortalId, WmoPortal);
member!(super::DoodadSetId, WmoDoodadSet);
member!(super::FogId, WmoFog);
member!(super::AmbientId, WmoAmbient);
