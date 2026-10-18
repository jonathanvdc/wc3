//! Structural and relationship checks before publishing a graph.
use crate::model::profiles::{FeaturePayload, Profile};
use crate::model::ValueSlot;
use crate::model::{
    AppearanceSchema, BindingSchema as _, Error, GeometrySchema, Link, Model, PropertyIdentity,
    PropertyTarget, SharedClock, SystemPayload, Time, TimeSource, ValidateLinks, VertexInfluences,
};
use std::collections::{BTreeSet, HashMap, HashSet};
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
            node.controls.visit_links(&mut visit);
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
            mesh.controls.visit_links(&mut visit);
            mesh.geometry.visit_links(&mut visit);
        }
        for slot in slots.clone() {
            if let ValueSlot::Scalar(id) = slot {
                if let Some(source) = self.values.boolean_source(id) {
                    slots.insert(source.slot());
                }
            }
        }
        if self
            .channels()
            .any(|channel| !slots.contains(&channel.target()))
        {
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
        self.units.validate()?;
        self.appearance.validate()?;
        if let Some(bounds) = self.bounds {
            bounds.validate()?;
        }
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
        let mut missing = false;
        let mut property_types = HashMap::new();
        let mut declared_properties = HashSet::new();
        let mut animated_properties = HashSet::new();
        let mut conflicting_types = false;
        let mut visit = |link| {
            missing |= match link {
                Link::Value(slot) => !self.values.contains(slot),
                Link::ImageResource(id) => !self.dependencies.images.contains_key(id),
                Link::ModelResource(id) => !self.dependencies.models.contains_key(id),
                Link::EffectResource(id) => !self.dependencies.effects.contains_key(id),
                Link::PhysicsResource(id) => !self.dependencies.physics.contains_key(id),
                Link::BoneOverrideResource(id) => {
                    !self.dependencies.bone_overrides.contains_key(id)
                }

                Link::Model(id) => id != self.id,
                Link::AppearanceSlot(id) => !self.appearance.contains_slot(id),
                Link::SystemMember { system, member } => !self
                    .systems
                    .get(system)
                    .is_some_and(|value| value.contains(member)),
                Link::InvalidValue => true,
                Link::Property {
                    target,
                    value_type,
                    definition,
                } => {
                    if definition {
                        declared_properties.insert(target.clone());
                    } else {
                        animated_properties.insert(target.clone());
                    }
                    if property_types
                        .insert(target.clone(), value_type)
                        .is_some_and(|previous| previous != value_type)
                    {
                        conflicting_types = true;
                    }
                    false
                }
                Link::Node(id) => !self.nodes.contains_key(id),
                Link::FeatureAlternatives { id, kinds } => self
                    .features
                    .get(id)
                    .is_none_or(|feature| !kinds.contains(&feature.payload.kind())),
                Link::TypedFeature { id, kind } => self
                    .features
                    .get(id)
                    .is_none_or(|feature| feature.payload.kind() != kind),
                Link::Event(id) => self.event(id).is_none(),
                Link::Feature(id) => !self.features.contains_key(id),
                Link::Instance(id) => !self.instances.contains_key(id),
                Link::TypedMesh { id, kind } => self
                    .meshes
                    .get(id)
                    .is_none_or(|mesh| mesh.geometry.kind() != kind),
                Link::Mesh(id) => !self.meshes.contains_key(id),
                Link::Material(id) => !self.materials.contains_key(id),
                Link::SampledImage(_) => false,
                Link::Clip(id) => !self.clips.contains_key(id),
                Link::Skin(id) => !self.skins.contains_key(id),
                Link::Clock(id) => !self.clocks.contains_key(id),
                Link::Animation(id) => !self.animations.contains_key(id),
                Link::Controller(id) => !self.controllers.contains_key(id),
                Link::System(id) => !self.systems.contains_key(id),
            };
        };
        self.values.visit_values(&mut visit);
        for value in self.dependencies.images.values() {
            value.visit_links(&mut visit);
        }
        for value in self.dependencies.models.values() {
            value.visit_links(&mut visit);
        }
        for value in self.dependencies.effects.values() {
            value.visit_links(&mut visit);
        }
        for value in self.dependencies.physics.values() {
            value.visit_links(&mut visit);
        }
        for value in self.dependencies.bone_overrides.values() {
            value.visit_links(&mut visit);
        }
        for node in self.nodes.values() {
            node.parent.visit_links(&mut visit);
            node.transform.translation.visit_links(&mut visit);
            node.transform.rotation.visit_links(&mut visit);
            node.transform.scale.visit_links(&mut visit);
            node.visible.visit_links(&mut visit);
            node.controls.visit_links(&mut visit);
        }
        for clip in self.clips.values() {
            clip.controls.visit_links(&mut visit);
        }
        self.global_animations.visit_links(&mut visit);
        for controller in self.controllers.values() {
            controller.controls.visit_links(&mut visit);
            controller.animations.visit_links(&mut visit);
        }
        for animation in self.animations.values() {
            for channel in &animation.channels {
                channel.visit_links(&mut visit);
            }
            for event in animation.events.values() {
                event.node.visit_links(&mut visit);
                event.feature.visit_links(&mut visit);
                if let TimeSource::Shared(id) = event.clock {
                    id.visit_links(&mut visit);
                }
            }
        }
        for skin in self.skins.values() {
            skin.joints.visit_links(&mut visit);
            skin.binding.visit_links(&mut visit);
        }
        for mesh in self.meshes.values() {
            mesh.controls.visit_links(&mut visit);
            mesh.geometry.visit_links(&mut visit);
        }
        for instance in self.instances.values() {
            instance.controls.visit_links(&mut visit);
            instance.node.visit_links(&mut visit);
            instance.mesh.visit_links(&mut visit);
            instance.skin.visit_links(&mut visit);
            for draw in &instance.draws {
                draw.controls.visit_links(&mut visit);
            }
        }
        for material in self.materials.values() {
            material.surface.visit_links(&mut visit);
        }
        for clip in self.clips.values() {
            clip.animations.visit_links(&mut visit);
        }
        for feature in self.features.values() {
            feature.node.visit_links(&mut visit);
            feature.payload.visit_links(&mut visit);
        }
        self.appearance.visit_links(&mut visit);
        self.metadata.visit_links(&mut visit);
        for system in self.systems.values() {
            system.visit_links(&mut visit);
        }
        if animated_properties.iter().any(|target| {
            matches!(target, PropertyIdentity::Profile { .. })
                && !declared_properties.contains(target)
        }) {
            return Err(Error::new(
                "animation.target",
                "structural destination is not declared by an authored owner field",
            ));
        }
        for target in property_types.keys() {
            let target = PropertyTarget::<P>::from_identity(target)?;
            P::validate_property(self, &target)?;
        }
        if conflicting_types {
            return Err(Error::new(
                "animation.properties",
                "one logical property is bound to incompatible value types",
            ));
        }
        if missing {
            return Err(Error::new(
                "relationships",
                "an object reference is absent, belongs to another graph, has the wrong payload type, or a rest value is invalid",
            ));
        }

        for ids in self
            .clips
            .values()
            .map(|clip| &clip.animations)
            .chain(
                self.controllers
                    .values()
                    .map(|controller| &controller.animations),
            )
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
        for event in self
            .animations
            .values()
            .flat_map(|animation| animation.events.values())
        {
            if let (Some(feature), Some(node)) = (event.feature, event.node) {
                if self.features[&feature].node != node {
                    return Err(Error::new(
                        "event.anchor",
                        "feature belongs to a different node",
                    ));
                }
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
