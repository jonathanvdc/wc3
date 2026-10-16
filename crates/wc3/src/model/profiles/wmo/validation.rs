//! Checks relationships between local world-profile objects.
use super::{Volume, WorldSystem};
use crate::model::{Bounds, Error};

impl WorldSystem<'_> {
    /// Checks portal, group, volume, set, and liquid relationships without a codec context.
    pub fn validate(&self) -> Result<(), Error> {
        let volume_valid = |volume: &Volume| match volume {
            Volume::Sphere { center, radii } => {
                center.iter().chain(radii).all(|v| v.is_finite())
                    && radii[0] >= 0.
                    && radii[1] >= radii[0]
            }
            Volume::Planes { planes, fade } => {
                planes.iter().flatten().all(|v| v.is_finite()) && fade.is_finite() && *fade >= 0.
            }
        };
        for group in &self.groups {
            if group
                .bounds
                .iter()
                .flatten()
                .any(|value| !value.is_finite())
                || group.bounds[0]
                    .iter()
                    .zip(group.bounds[1])
                    .any(|(low, high)| *low > high)
            {
                return Err(Error::new(
                    "world.groups.bounds",
                    "catalog bounds must be finite and ordered",
                ));
            }
        }
        for portal in &self.portals {
            if portal
                .vertices
                .iter()
                .flatten()
                .chain(portal.plane)
                .any(|value| !value.is_finite())
            {
                return Err(Error::new(
                    "world.portals",
                    "portal vertices and plane must be finite",
                ));
            }
        }
        if self
            .visibility
            .iter()
            .any(|polygon| polygon.iter().flatten().any(|value| !value.is_finite()))
            || self
                .occluders
                .iter()
                .flatten()
                .any(|value| !value.is_finite())
        {
            return Err(Error::new(
                "world.visibility",
                "visibility geometry must be finite",
            ));
        }
        for fog in &self.fogs {
            if !volume_valid(&fog.volume)
                || [
                    fog.above_water.end,
                    fog.above_water.start_fraction,
                    fog.underwater.end,
                    fog.underwater.start_fraction,
                ]
                .iter()
                .any(|value| !value.is_finite())
            {
                return Err(Error::new(
                    "profile.fogs",
                    "fog boundary or doodad-set activation is invalid",
                ));
            }
        }
        for ambient in &self.ambient_volumes {
            if !volume_valid(&ambient.volume) {
                return Err(Error::new(
                    "profile.ambient_volumes",
                    "ambient boundary or doodad-set activation is invalid",
                ));
            }
        }
        Ok(())
    }
}

pub(super) fn model(value: &crate::model::Model<'_, super::Wmo>) -> Result<(), Error> {
    if value.units() != crate::model::LengthUnit::Unspecified
        || value.basis() != crate::model::Basis::default()
    {
        return Err(Error::new(
            "model.coordinates",
            "native model coordinates use the file family's implicit basis without authored global unit metadata",
        ));
    }

    if let Some(bounds) = value.bounds() {
        aabb_bounds(bounds, "world.bounds")?;
    }
    for (_, mesh) in value.meshes() {
        aabb_bounds(mesh.bounds, "world.geometry.bounds")?;
    }
    use crate::model::{Billboard, Inheritance, Transform};
    if value.clips().len() != 0
        || value.clocks().len() != 0
        || value.controllers().len() != 0
        || value.skins().len() != 0
        || value.texture_transforms().len() != 0
        || !value.timeline_channels().is_empty()
        || !value.shared_channels().is_empty()
        || !value.timeline_events().is_empty()
        || !value.shared_events().is_empty()
    {
        return Err(Error::new(
            "world.animation",
            "static world models have no skeleton, timeline, event program, or layered controller",
        ));
    }
    if value.systems().len() != 1 {
        return Err(Error::new(
            "world.systems",
            "one static world model contains one world-system collection",
        ));
    }
    for (_, node) in value.nodes() {
        if node.parent.is_some()
            || node.inheritance != Inheritance::default()
            || node.billboard != Billboard::None
            || !node.visible
            || node.camera_anchored
            || node.transform.pivot != [0.; 3]
        {
            return Err(Error::new(
                "world.nodes",
                "static world actors use absolute transforms without inheritance, visibility, billboard or camera constraints",
            ));
        }
    }
    for (_, instance) in value.instances() {
        if instance.selection_group != 0 {
            return Err(Error::new(
                "world.regions.selection_group",
                "native world regions have no geometry selection-group field",
            ));
        }

        for ambient in &instance.controls.ambients {
            if value
                .system(ambient.system)
                .and_then(|world| world.ambient_volumes.get(ambient.index as usize))
                .is_some_and(|ambient| ambient.global)
            {
                return Err(Error::new(
                    "world.regions.ambient",
                    "globally active illumination cannot be selected by individual regions",
                ));
            }
        }
        for draw in &instance.draws {
            if let Some(controls) = &draw.controls {
                if let Some(bounds) = controls.bounds {
                    aabb_bounds(bounds, "world.draw.bounds")?;
                }
            }
        }

        if value
            .node(instance.node)
            .expect("validated placement")
            .transform
            != Transform::default()
        {
            return Err(Error::new(
                "world.regions.transform",
                "native world geometry is authored directly in world coordinates",
            ));
        }
    }
    for (_, feature) in value.features() {
        let node = value.node(feature.node).expect("validated feature anchor");
        match &feature.payload {
            super::Node::Light(light) => {
                if light.color.binding.is_some() || light.intensity.binding.is_some() {
                    return Err(Error::new(
                        "world.light.curves",
                        "static world lights use constant properties or authored flicker programs",
                    ));
                }
                if node.transform.scale != [1.; 3]
                    || (light.modern.is_some() && node.transform.rotation != Default::default())
                {
                    return Err(Error::new(
                        "world.light.transform",
                        "light shape scale and modern Euler orientation belong to the light definition",
                    ));
                }
            }
            super::Node::Attachment(_) => {
                if node.transform.scale[0] != node.transform.scale[1]
                    || node.transform.scale[1] != node.transform.scale[2]
                {
                    return Err(Error::new(
                        "world.doodad.scale",
                        "native world doodads have uniform scale",
                    ));
                }
            }
            super::Node::PointLight(_) | super::Node::LightProgram(_) => {
                if node.transform.rotation != Default::default() || node.transform.scale != [1.; 3]
                {
                    return Err(Error::new(
                        "world.region.light.transform",
                        "region lights use absolute translation; program Euler orientation belongs to the program definition",
                    ));
                }
            }

            super::Node::Camera(unavailable)
            | super::Node::Particle(unavailable)
            | super::Node::Ribbon(unavailable)
            | super::Node::Event(unavailable)
            | super::Node::GeometryAnimation(unavailable)
            | super::Node::CollisionShape(unavailable)
            | super::Node::ParticleProgram(unavailable)
            | super::Node::FaceActor(unavailable)
            | super::Node::Extent(unavailable)
            | super::Node::ParticleCopy(unavailable)
            | super::Node::Projector(unavailable)
            | super::Node::Force(unavailable)
            | super::Node::Warp(unavailable)
            | super::Node::HitTest(unavailable)
            | super::Node::AttachmentVolume(unavailable)
            | super::Node::Billboard(unavailable)
            | super::Node::Turret(unavailable)
            | super::Node::IkJoint(unavailable)
            | super::Node::IkTwoJoint(unavailable)
            | super::Node::IkCcd(unavailable)
            | super::Node::OneBoneSolver(unavailable)
            | super::Node::ShadowBox(unavailable)
            | super::Node::ViewVolume(unavailable)
            | super::Node::AtlasParticles(unavailable) => match *unavailable {},
        }
    }
    Ok(())
}

pub(super) fn resources(value: &crate::model::Resources<'_, super::Wmo>) -> Result<(), Error> {
    use crate::model::{ColorSpace, Resource, Sampler};
    for (_, texture) in value.textures() {
        if !matches!(
            texture.resource,
            Resource::Path(_) | Resource::FileDataId(_) | Resource::EmbeddedImage { .. }
        ) {
            return Err(Error::new(
                "world.textures.resource",
                "world surfaces refer to filenames, database identities, or embedded lightmap images",
            ));
        }
        if texture.color_space != ColorSpace::Unspecified || texture.sampler != Sampler::default() {
            return Err(Error::new(
                "world.textures.sampling",
                "world texture definitions do not declare transfer functions or global sampling overrides; addressing belongs to the surface",
            ));
        }
    }
    Ok(())
}

fn aabb_bounds(bounds: Bounds, path: &str) -> Result<(), Error> {
    let radius = bounds
        .minimum
        .iter()
        .zip(bounds.maximum)
        .map(|(low, high)| ((high - low) / 2.).powi(2))
        .sum::<f32>()
        .sqrt();
    if bounds.sphere_radius != radius {
        return Err(Error::new(
            path,
            "world bounds store an axis-aligned box; the sphere radius must equal its derived half-diagonal",
        ));
    }
    Ok(())
}
