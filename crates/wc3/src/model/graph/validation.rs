//! Structural and relationship checks before publishing a graph.
use crate::model::profiles::{FeaturePayload, Profile};
use crate::model::ValueSlot;
use crate::model::{
    AppearanceSchema, BindingSchema as _, Error, GeometrySchema, Link, Model, SharedClock,
    SystemPayload, Time, ValidateLinks, VertexInfluences,
};
use std::cell::Cell;
use std::collections::{BTreeSet, HashSet};
use std::iter::once;
impl<P: Profile> Model<'_, P> {
    /// Checks that native programs only write slots represented by their consuming definitions.
    pub(crate) fn validate_animation_consumers(&self) -> Result<(), Error> {
        let mut slots = HashSet::new();
        let mut visit = |link| {
            if let Link::Value(slot) = link {
                slots.insert(slot);
            }
        };
        for node in self.nodes.values() {
            node.transform.translation.visit_links(&mut visit);
            node.transform.rotation.visit_links(&mut visit);
            node.transform.scale.visit_links(&mut visit);
            node.visible.visit_links(&mut visit);
            node.properties.visit_links(&mut visit);
        }
        for material in self.materials.values() {
            material.surface.visit_links(&mut visit);
        }
        for feature in self.features.values() {
            feature.payload.visit_links(&mut visit);
        }
        for system in self.systems.values() {
            system.visit_links(&mut visit);
        }
        for mesh in self.meshes.values() {
            mesh.properties.visit_links(&mut visit);
            mesh.geometry.visit_links(&mut visit);
        }
        for slot in slots.clone() {
            if let ValueSlot::Scalar(id) = slot {
                if let Some(source) = self.values.boolean_source(id) {
                    slots.insert(source.slot());
                }
            }
        }
        if self.tracks().any(|track| !slots.contains(&track.target())) {
            return Err(Error::new(
                "animation.target",
                "native output has no consuming field for an animated slot",
            ));
        }
        Ok(())
    }
}
impl<P: Profile> Model<'_, P> {
    /// Checks values, graph relationships, metadata, and owned resources.
    pub fn validate(&self) -> Result<(), Error> {
        self.values.validate_constants()?;
        self.appearance.validate()?;
        for (id, node) in self.nodes.iter() {
            node.transform.validate(&self.values)?;
            let mut ancestors = BTreeSet::new();
            ancestors.insert(id);
            let mut parent = node.parent;
            while let Some(p) = parent {
                if !ancestors.insert(p) {
                    return Err(Error::new("nodes.parent", "hierarchy contains a cycle"));
                }
                parent = self
                    .nodes
                    .get(p)
                    .ok_or_else(|| Error::new("nodes.parent", "parent is absent"))?
                    .parent;
            }
        }
        for mesh in self.meshes.values() {
            mesh.validate()?;
        }
        for material in self.materials.values() {
            material.validate()?;
        }
        for clip in self.clips.values() {
            Time::seconds(clip.source_start.as_seconds() + clip.duration.as_seconds())?;
            if clip.duration.as_seconds() < 0. {
                return Err(Error::new("clip.duration", "duration must be nonnegative"));
            }
        }
        for clock in self.clocks.values() {
            if matches!(clock, SharedClock::Repeating { period } if period.as_seconds() <= 0.) {
                return Err(Error::new(
                    "clock.period",
                    "repeating period must be positive",
                ));
            }
        }
        for skin in self.skins.values() {
            skin.binding.validate(skin.joints.len())?;
        }
        self.validate_relationships()?;
        P::validate_model(self)?;
        P::validate_resources(self)
    }
}
impl<P: Profile> Model<'_, P> {
    fn validate_relationships(&self) -> Result<(), Error> {
        let mut missing = None;
        let location = Cell::new(("values", 0));
        let mut visit = |link| {
            let invalid = match &link {
                Link::Value(slot) => !self.values.contains(*slot),
                Link::ImageResource(id) => !self.dependencies.images.contains_key(*id),
                Link::ModelResource(id) => !self.dependencies.models.contains_key(*id),
                Link::EffectResource(id) => !self.dependencies.effects.contains_key(*id),
                Link::PhysicsResource(id) => !self.dependencies.physics.contains_key(*id),
                Link::BoneOverrideResource(id) => {
                    !self.dependencies.bone_overrides.contains_key(*id)
                }

                Link::SystemMember { system, member } => !self
                    .systems
                    .get(*system)
                    .is_some_and(|value| value.contains(*member)),
                Link::InvalidValue => true,
                Link::Node(id) => !self.nodes.contains_key(*id),
                Link::FeatureAlternatives { id, kinds } => self
                    .features
                    .get(*id)
                    .is_none_or(|feature| !kinds.contains(&feature.payload.kind())),
                Link::TypedFeature { id, kind } => self
                    .features
                    .get(*id)
                    .is_none_or(|feature| feature.payload.kind() != *kind),
                Link::EventSource(id) => !self.event_sources.contains_key(*id),
                Link::Event(id) => self.event(*id).is_none(),
                Link::Feature(id) => !self.features.contains_key(*id),
                Link::Instance(id) => !self.instances.contains_key(*id),
                Link::TypedMesh { id, kind } => self
                    .meshes
                    .get(*id)
                    .is_none_or(|mesh| mesh.geometry.kind() != *kind),
                Link::Mesh(id) => !self.meshes.contains_key(*id),
                Link::Material(id) => !self.materials.contains_key(*id),
                Link::SampledImage(_) => false,
                Link::Clip(id) => !self.clips.contains_key(*id),
                Link::Skin(id) => !self.skins.contains_key(*id),
                Link::Clock(id) => !self.clocks.contains_key(*id),
                Link::Animation(id) => !self.animations.contains_key(*id),
                Link::System(id) => !self.systems.contains_key(*id),
            };
            if invalid && missing.is_none() {
                missing = Some((location.get(), link));
            }
        };
        self.values.visit_values(&mut visit);
        for (index, source) in self.event_sources.values().enumerate() {
            location.set(("event_sources", index));
            source.visit_links(&mut visit);
        }
        for (index, value) in self.dependencies.images.values().enumerate() {
            location.set(("image_resources", index));
            value.visit_links(&mut visit);
        }
        for (index, value) in self.dependencies.models.values().enumerate() {
            location.set(("model_resources", index));
            value.visit_links(&mut visit);
        }
        for (index, value) in self.dependencies.effects.values().enumerate() {
            location.set(("effect_resources", index));
            value.visit_links(&mut visit);
        }
        for (index, value) in self.dependencies.physics.values().enumerate() {
            location.set(("physics_resources", index));
            value.visit_links(&mut visit);
        }
        for (index, value) in self.dependencies.bone_overrides.values().enumerate() {
            location.set(("bone_override_resources", index));
            value.visit_links(&mut visit);
        }
        for (index, node) in self.nodes.values().enumerate() {
            location.set(("nodes", index));
            node.parent.visit_links(&mut visit);
            node.transform.translation.visit_links(&mut visit);
            node.transform.rotation.visit_links(&mut visit);
            node.transform.scale.visit_links(&mut visit);
            node.visible.visit_links(&mut visit);
            node.properties.visit_links(&mut visit);
        }
        for (index, clip) in self.clips.values().enumerate() {
            location.set(("clips", index));
            clip.properties.visit_links(&mut visit);
        }
        location.set(("global_animations", 0));
        self.global_animations.visit_links(&mut visit);
        for (index, animation) in self.animations.values().enumerate() {
            location.set(("animations", index));
            for track in &animation.tracks {
                track.visit_links(&mut visit);
            }
            for event in animation.events.values() {
                event.visit_links(&mut visit);
            }
        }
        for (index, skin) in self.skins.values().enumerate() {
            location.set(("skins", index));
            skin.joints.visit_links(&mut visit);
            skin.binding.visit_links(&mut visit);
        }
        for (index, mesh) in self.meshes.values().enumerate() {
            location.set(("meshes", index));
            mesh.properties.visit_links(&mut visit);
            mesh.geometry.visit_links(&mut visit);
        }
        for (index, instance) in self.instances.values().enumerate() {
            location.set(("instances", index));
            instance.properties.visit_links(&mut visit);
            instance.node.visit_links(&mut visit);
            instance.mesh.visit_links(&mut visit);
            instance.skin.visit_links(&mut visit);
            for draw in &instance.draws {
                draw.properties.visit_links(&mut visit);
            }
        }
        for (index, material) in self.materials.values().enumerate() {
            location.set(("materials", index));
            material.surface.visit_links(&mut visit);
        }
        for (index, clip) in self.clips.values().enumerate() {
            location.set(("clips", index));
            clip.animations.visit_links(&mut visit);
        }
        for (index, feature) in self.features.values().enumerate() {
            location.set(("features", index));
            feature.node.visit_links(&mut visit);
            feature.payload.visit_links(&mut visit);
        }
        location.set(("appearance", 0));
        self.appearance.visit_links(&mut visit);
        location.set(("metadata", 0));
        self.metadata.visit_links(&mut visit);
        for (index, system) in self.systems.values().enumerate() {
            location.set(("systems", index));
            system.visit_links(&mut visit);
        }
        if let Some(((owner, index), link)) = missing {
            return Err(Error::new(
                format!("relationships.{owner}[{index}]"),
                format!("invalid {link:?}: reference is absent, belongs to another graph, has the wrong payload type, or contains an invalid rest value"),
            ));
        }

        for ids in self
            .clips
            .values()
            .map(|clip| &clip.animations)
            .chain(once(&self.global_animations))
        {
            let mut seen = HashSet::new();
            if ids.iter().any(|id| !seen.insert(id)) {
                return Err(Error::new(
                    "animation.selection",
                    "animation selected more than once",
                ));
            }
        }
        let mut event_ids = HashSet::new();
        for animation in self.animations.values() {
            if animation
                .events
                .keys()
                .any(|id| !self.scope.contains(*id) || !event_ids.insert(*id))
            {
                return Err(Error::new(
                    "animation.events",
                    "event identity belongs to another graph or multiple animations",
                ));
            }
        }
        let mut draw_ids = BTreeSet::new();
        let expected: BTreeSet<_> = self
            .instances
            .iter()
            .flat_map(|(id, instance)| instance.draws.iter().map(move |draw| (id, draw.id())))
            .collect();
        for instance in self.instances.values() {
            for draw in &instance.draws {
                if !draw_ids.insert(draw.id()) {
                    return Err(Error::new(
                        "instance.draw",
                        "draw identity is used more than once",
                    ));
                }
            }
        }
        if !expected.is_empty() && self.looks().len() == 0 {
            return Err(Error::new("look", "draws require a complete material look"));
        }
        for (_, look) in self.looks() {
            let mut assignments = BTreeSet::new();
            for binding in &look.bindings {
                if !assignments.insert((binding.instance, binding.draw)) {
                    return Err(Error::new("look.bindings", "draw has multiple assignments"));
                }
                if !self.materials.contains_key(binding.material)
                    || self.instances.get(binding.instance).is_none_or(|instance| {
                        !instance.draws.iter().any(|draw| draw.id() == binding.draw)
                    })
                {
                    return Err(Error::new(
                        "look.bindings",
                        "material or instance draw is absent",
                    ));
                }
            }
            if assignments != expected {
                return Err(Error::new(
                    "look.bindings",
                    "look must assign every instance draw",
                ));
            }
        }
        for instance in self.instances.values() {
            let mesh = &self.meshes[&instance.mesh];
            for draw in &instance.draws {
                if mesh.subset(draw.subset).is_none() {
                    return Err(Error::new(
                        "instance.draw.subset",
                        "draw subset belongs to another mesh or is absent",
                    ));
                }
            }
            if mesh
                .influences()
                .iter()
                .filter_map(VertexInfluences::into_joint_weights)
                .flatten()
                .any(|influence| influence.weight > 0.)
            {
                let skin = instance
                    .skin
                    .or_else(|| mesh.skin())
                    .and_then(|id| self.skins.get(id))
                    .ok_or_else(|| {
                        Error::new("instance.skin", "skinned geometry requires a skin")
                    })?;
                if mesh
                    .influences()
                    .iter()
                    .filter_map(VertexInfluences::into_joint_weights)
                    .flatten()
                    .any(|v| v.weight > 0. && v.joint as usize >= skin.joints.len())
                {
                    return Err(Error::new("instance.skin", "influence joint is absent"));
                }
            }
        }
        Ok(())
    }
}
