//! Checks relationships between local world-profile objects.
use super::{Volume, WorldSystem};
use crate::model::error::{
    GeometryError, Location, ProfileError, ProfileKind, ProfileViolation, StaticModelComponent,
};
use crate::model::geometry_schema::{check_finite, check_finite_values};
use crate::model::Model;
use crate::model::TimeSource;
use crate::model::{Bounds, Error};
use glam::Vec3;

impl WorldSystem<'_> {
    /// Checks finite group and portal geometry and environmental volume boundaries.
    pub fn validate(&self) -> Result<(), Error> {
        for (index, group) in self.groups.iter().enumerate() {
            check_finite(group.bounds.iter().map(|v| v.to_array()), || {
                Location::new().field("groups").index(index).field("bounds")
            })?;
            for (component, (minimum, maximum)) in group.bounds[0]
                .to_array()
                .into_iter()
                .zip(group.bounds[1].to_array())
                .enumerate()
            {
                if minimum > maximum {
                    return Err(Error::at(
                        Location::new()
                            .field("groups")
                            .index(index)
                            .field("bounds")
                            .index(0)
                            .index(component),
                        GeometryError::InvertedBounds { minimum, maximum },
                    ));
                }
            }
        }
        for (index, portal) in self.portals.iter().enumerate() {
            check_finite(portal.vertices.iter().map(|v| v.to_array()), || {
                Location::new()
                    .field("portals")
                    .index(index)
                    .field("vertices")
            })?;
            check_finite_values(portal.plane.to_array(), || {
                Location::new().field("portals").index(index).field("plane")
            })?;
        }
        for (index, polygon) in self.visibility.iter().enumerate() {
            check_finite(polygon.iter().map(|v| v.to_array()), || {
                Location::new().field("visibility").index(index)
            })?;
        }
        check_finite(self.occluders.iter().map(|v| v.to_array()), || {
            Location::new().field("occluders")
        })?;
        for (index, fog) in self.fogs.iter().enumerate() {
            volume(&fog.volume, || {
                Location::new().field("fogs").index(index).field("volume")
            })?;
            for (field, parameters) in [
                ("above_water", &fog.above_water),
                ("underwater", &fog.underwater),
            ] {
                for (name, value) in [
                    ("end", parameters.end),
                    ("start_fraction", parameters.start_fraction),
                ] {
                    if !value.is_finite() {
                        return Err(Error::at(
                            Location::new()
                                .field("fogs")
                                .index(index)
                                .field(field)
                                .field(name),
                            GeometryError::NonFinite { value },
                        ));
                    }
                }
            }
        }
        for (index, ambient) in self.ambient_volumes.iter().enumerate() {
            volume(&ambient.volume, || {
                Location::new()
                    .field("ambient_volumes")
                    .index(index)
                    .field("volume")
            })?;
        }
        Ok(())
    }
}
fn volume(value: &Volume, location: impl Fn() -> Location) -> Result<(), Error> {
    match value {
        Volume::Sphere { center, radii } => {
            check_finite_values(center.to_array(), || location().field("center"))?;
            check_finite_values(radii.to_array(), || location().field("radii"))?;
            if radii[0] < 0. {
                return Err(Error::at(
                    location().field("radii").index(0),
                    GeometryError::NegativeRadius { radius: radii[0] },
                ));
            }
            if radii[1] < radii[0] {
                return Err(Error::at(
                    location().field("radii").index(1),
                    GeometryError::InvertedBounds {
                        minimum: radii[0],
                        maximum: radii[1],
                    },
                ));
            }
        }
        Volume::Planes { planes, fade } => {
            check_finite(planes.iter().map(|v| v.to_array()), || {
                location().field("planes")
            })?;
            if !fade.is_finite() {
                return Err(Error::at(
                    location().field("fade"),
                    GeometryError::NonFinite { value: *fade },
                ));
            }
            if *fade < 0. {
                return Err(Error::at(
                    location().field("fade"),
                    ProfileError::new(
                        ProfileKind::Wmo,
                        ProfileViolation::NegativeVolumeFade { actual: *fade },
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn model(value: &Model<'_, super::Wmo>) -> Result<(), Error> {
    if let Some(bounds) = value.bounds() {
        aabb_bounds(bounds, || Location::new().field("bounds"))?;
    }
    for (id, mesh) in value.meshes() {
        aabb_bounds(mesh.bounds, || Location::for_owner(id).field("bounds"))?;
    }
    use crate::model::{Billboard, Inheritance, Transform};
    for (component, actual) in [
        (StaticModelComponent::Clips, value.clips().len()),
        (StaticModelComponent::Clocks, value.clocks().len()),
        (StaticModelComponent::Skins, value.skins().len()),
        (
            StaticModelComponent::OrdinaryTracks,
            value
                .global_tracks()
                .filter(|track| track.clock() == TimeSource::Animation)
                .count(),
        ),
        (
            StaticModelComponent::SharedTracks,
            value
                .global_tracks()
                .filter(|track| matches!(track.clock(), TimeSource::Shared(_)))
                .count(),
        ),
        (
            StaticModelComponent::Events,
            value
                .animations()
                .values()
                .map(|animation| animation.events.len())
                .sum(),
        ),
    ] {
        if actual != 0 {
            return Err(ProfileError::new(
                ProfileKind::Wmo,
                ProfileViolation::StaticAnimation { component, actual },
            )
            .into());
        }
    }
    if value.systems().len() != 1 {
        return Err(Error::at(
            Location::new().field("systems"),
            ProfileError::new(
                ProfileKind::Wmo,
                ProfileViolation::WorldSystemCount {
                    expected: 1,
                    actual: value.systems().len(),
                },
            ),
        ));
    }
    for (id, node) in value.nodes() {
        if node.parent.is_some()
            || node.inheritance != Inheritance::default()
            || node.billboard != Billboard::None
            || value.values().get(node.visible) != Some(&true)
            || node.camera_anchored
            || node.transform.defaults(value.values()).pivot != Vec3::from_array([0.; 3])
        {
            return Err(Error::at(
                Location::for_owner(id),
                ProfileError::new(ProfileKind::Wmo, ProfileViolation::AbsoluteNodeTransform),
            ));
        }
    }
    for (id, instance) in value.instances() {
        if instance.selection_group != 0 {
            return Err(Error::at(
                Location::for_owner(id).field("selection_group"),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::RegionSelectionGroup {
                        actual: instance.selection_group,
                    },
                ),
            ));
        }

        for ambient in &instance.properties.ambients {
            if value
                .system(ambient.system)
                .and_then(|world| world.ambient_volumes.get(ambient.index as usize))
                .is_some_and(|ambient| ambient.global)
            {
                return Err(Error::at(
                    Location::for_owner(id)
                        .field("properties")
                        .field("ambients")
                        .with_owner(ambient.system),
                    ProfileError::new(ProfileKind::Wmo, ProfileViolation::GlobalAmbientSelection),
                ));
            }
        }
        for draw in &instance.draws {
            if let Some(controls) = &draw.properties {
                if let Some(bounds) = controls.bounds {
                    aabb_bounds(bounds, || {
                        Location::for_owner(draw.id())
                            .field("properties")
                            .field("bounds")
                    })?;
                }
            }
        }

        if value
            .node(instance.node)
            .expect("validated placement")
            .transform
            .defaults(value.values())
            != Transform::default()
        {
            return Err(Error::at(
                Location::for_owner(id).field("node"),
                ProfileError::new(ProfileKind::Wmo, ProfileViolation::RegionTransform),
            ));
        }
    }
    for (id, feature) in value.features() {
        let node = value.node(feature.node).expect("validated feature anchor");
        match &feature.payload {
            super::Node::Light(light) => {
                if value.is_animated(light.color) || value.is_animated(light.intensity) {
                    return Err(Error::at(
                        Location::for_owner(id),
                        ProfileError::new(ProfileKind::Wmo, ProfileViolation::LightAnimation),
                    ));
                }
                if node.transform.defaults(value.values()).scale != Vec3::from_array([1.; 3])
                    || (light.modern.is_some()
                        && node.transform.defaults(value.values()).rotation != Default::default())
                {
                    return Err(Error::at(
                        Location::for_owner(id)
                            .with_owner(feature.node)
                            .field("transform"),
                        ProfileError::new(ProfileKind::Wmo, ProfileViolation::LightTransform),
                    ));
                }
            }
            super::Node::Attachment(_) => {
                if node.transform.defaults(value.values()).scale[0]
                    != node.transform.defaults(value.values()).scale[1]
                    || node.transform.defaults(value.values()).scale[1]
                        != node.transform.defaults(value.values()).scale[2]
                {
                    return Err(Error::at(
                        Location::for_owner(id)
                            .with_owner(feature.node)
                            .field("transform")
                            .field("scale"),
                        ProfileError::new(ProfileKind::Wmo, ProfileViolation::DoodadScale),
                    ));
                }
            }
            super::Node::PointLight(_) | super::Node::LightProgram(_) => {
                if node.transform.defaults(value.values()).rotation != Default::default()
                    || node.transform.defaults(value.values()).scale != Vec3::from_array([1.; 3])
                {
                    return Err(Error::at(
                        Location::for_owner(id)
                            .with_owner(feature.node)
                            .field("transform"),
                        ProfileError::new(ProfileKind::Wmo, ProfileViolation::RegionLightTransform),
                    ));
                }
            }

            super::Node::Camera(unavailable)
            | super::Node::Particle(unavailable)
            | super::Node::Ribbon(unavailable)
            | super::Node::GeometryAnimation(unavailable)
            | super::Node::CollisionShape(unavailable)
            | super::Node::ParticleProgram(unavailable)
            | super::Node::FaceActor(unavailable)
            | super::Node::Particle2(unavailable) => match *unavailable {},
        }
    }
    Ok(())
}

pub(super) fn resources(value: &Model<'_, super::Wmo>) -> Result<(), Error> {
    use crate::model::{AddressMode, ColorSpace, Sampler};
    for (id, material) in value.materials() {
        for (index, input) in material.surface.inputs().iter().enumerate() {
            if !matches!(input.sampling.u, AddressMode::Repeat | AddressMode::Clamp)
                || !matches!(input.sampling.v, AddressMode::Repeat | AddressMode::Clamp)
            {
                return Err(Error::at(
                    Location::for_owner(id)
                        .field("surface")
                        .index(index)
                        .field("sampling"),
                    ProfileError::new(
                        ProfileKind::Wmo,
                        ProfileViolation::AddressMode {
                            u: input.sampling.u,
                            v: input.sampling.v,
                        },
                    ),
                ));
            }
        }
    }
    for texture in value.sampled_images() {
        if texture.color_space != ColorSpace::Unspecified || texture.sampler != Sampler::default() {
            return Err(Error::at(
                Location::for_owner(texture.resource),
                ProfileError::new(
                    ProfileKind::Wmo,
                    ProfileViolation::ImageSampling {
                        color_space: texture.color_space,
                        sampler: texture.sampler,
                    },
                ),
            ));
        }
    }
    Ok(())
}

fn aabb_bounds(bounds: Bounds, location: impl Fn() -> Location) -> Result<(), Error> {
    let radius = bounds
        .minimum
        .to_array()
        .iter()
        .zip(bounds.maximum.to_array())
        .map(|(low, high)| ((high - low) / 2.).powi(2))
        .sum::<f32>()
        .sqrt();
    if bounds.sphere_radius != radius {
        return Err(Error::at(
            location().field("sphere_radius"),
            ProfileError::new(
                ProfileKind::Wmo,
                ProfileViolation::BoundsRadius {
                    expected: radius,
                    actual: bounds.sphere_radius,
                },
            ),
        ));
    }
    Ok(())
}
