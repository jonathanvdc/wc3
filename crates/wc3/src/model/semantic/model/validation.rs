//! Structural and relationship checks before publishing a graph.
use crate::model::semantic::profiles::{FeaturePayload, Profile};
use crate::model::semantic::{
    AppearanceSchema, BindingSchema as _, Clock, Error, GeometrySchema, Link, Model, ModelId,
    PropertyIdentity, PropertyTarget, Resources, SystemPayload, ValidateLinks, VertexInfluences,
};
use std::collections::{BTreeSet, HashMap, HashSet};
impl<P: Profile> Model<'_, P> {
    pub(crate) fn validate(
        &self,
        resources: &Resources<'_, P>,
        models: &BTreeSet<ModelId>,
    ) -> Result<(), Error> {
        self.units.validate()?;
        self.appearance.validate()?;
        if let Some(bounds) = self.bounds {
            bounds.validate()?;
        }
        for (id, node) in &self.nodes {
            node.transform.validate()?;
            let mut ancestors = BTreeSet::new();
            ancestors.insert(*id);
            let mut parent = node.parent;
            while let Some(p) = parent {
                if !ancestors.insert(p) {
                    return Err(Error::new("nodes.parent", "hierarchy contains a cycle"));
                }
                parent = self
                    .nodes
                    .get(&p)
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
            if clip.duration.as_seconds() < 0. {
                return Err(Error::new("clip.duration", "duration must be nonnegative"));
            }
        }
        for clock in self.clocks.values() {
            if clock.period.as_seconds() <= 0. {
                return Err(Error::new("clock.period", "period must be positive"));
            }
        }
        for skin in self.skins.values() {
            skin.binding.validate(skin.joints.len())?;
        }
        self.validate_relationships(resources, models)?;
        P::validate_model(self)
    }
}
impl<P: Profile> Model<'_, P> {
    fn validate_relationships(
        &self,
        resources: &Resources<'_, P>,
        models: &BTreeSet<ModelId>,
    ) -> Result<(), Error> {
        let mut missing = false;
        let mut property_types = HashMap::new();
        let mut declared_properties = HashSet::new();
        let mut animated_properties = HashSet::new();
        let mut conflicting_types = false;
        let mut visit = |link| {
            missing |= match link {
                Link::Model(id) => !models.contains(&id),
                Link::AppearanceSlot(id) => !self.appearance.contains_slot(id),
                Link::SystemMember { system, member } => !self
                    .systems
                    .get(&system)
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
                Link::Node(id) => !self.nodes.contains_key(&id),
                Link::FeatureAlternatives { id, kinds } => self
                    .features
                    .get(&id)
                    .is_none_or(|feature| !kinds.contains(&feature.payload.kind())),
                Link::TypedFeature { id, kind } => self
                    .features
                    .get(&id)
                    .is_none_or(|feature| feature.payload.kind() != kind),
                Link::Event(id) => self.event(id).is_none(),
                Link::Feature(id) => !self.features.contains_key(&id),
                Link::Instance(id) => !self.instances.contains_key(&id),
                Link::TypedMesh { id, kind } => self
                    .meshes
                    .get(&id)
                    .is_none_or(|mesh| mesh.geometry.kind() != kind),
                Link::Mesh(id) => !self.meshes.contains_key(&id),
                Link::Material(id) => !self.materials.contains_key(&id),
                Link::Texture(id) => resources.texture(id).is_none(),
                Link::Clip(id) => !self.clips.contains_key(&id),
                Link::Skin(id) => !self.skins.contains_key(&id),
                Link::Clock(id) => !self.clocks.contains_key(&id),
                Link::Controller(id) => !self.controllers.contains_key(&id),
                Link::TextureTransform(id) => !self.texture_transforms.contains_key(&id),
                Link::System(id) => !self.systems.contains_key(&id),
            };
        };
        for node in self.nodes.values() {
            node.parent.visit_links(&mut visit);
            node.controls.visit_links(&mut visit);
        }
        for transform in self.texture_transforms.values() {
            transform.visit_links(&mut visit);
        }
        for clip in self.clips.values() {
            clip.controls.visit_links(&mut visit);
        }
        for controller in self.controllers.values() {
            controller.controls.visit_links(&mut visit);
            for channel in &controller.channels {
                channel.visit_links(&mut visit);
            }
            for event in controller.events.values() {
                event.node.visit_links(&mut visit);
                event.feature.visit_links(&mut visit);
                if let Clock::Shared(id) = event.clock {
                    id.visit_links(&mut visit);
                }
            }
        }
        for (_, texture) in resources.textures() {
            texture.controls.visit_links(&mut visit);
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
            clip.alias.visit_links(&mut visit);
            for channel in &clip.channels {
                channel.visit_links(&mut visit);
            }
            for event in &clip.events {
                event.node.visit_links(&mut visit);
                event.feature.visit_links(&mut visit);
                if let Clock::Shared(id) = event.clock {
                    id.visit_links(&mut visit);
                }
            }
        }
        for feature in self.features.values() {
            feature.node.visit_links(&mut visit);
            feature.payload.visit_links(&mut visit);
        }
        for channel in &self.timeline_channels {
            if channel.clock() != Clock::Timeline {
                return Err(Error::new(
                    "timeline_channels.clock",
                    "expected the absolute authoring clock",
                ));
            }
            channel.visit_links(&mut visit);
        }
        for event in &self.timeline_events {
            event.node.visit_links(&mut visit);
            event.feature.visit_links(&mut visit);
            if event.clock != Clock::Timeline {
                return Err(Error::new(
                    "timeline_events.clock",
                    "expected the absolute authoring clock",
                ));
            }
        }
        for channel in &self.shared_channels {
            channel.visit_links(&mut visit);
        }
        for event in &self.shared_events {
            event.node.visit_links(&mut visit);
            event.feature.visit_links(&mut visit);
            if let Clock::Shared(id) = event.clock {
                id.visit_links(&mut visit);
            } else {
                return Err(Error::new(
                    "shared_events.clock",
                    "event must use a shared clock",
                ));
            }
        }
        self.appearance.visit_links(&mut visit);
        self.controls.visit_links(&mut visit);
        for system in self.systems.values() {
            system.visit_links(&mut visit);
        }
        if animated_properties.iter().any(|target| {
            matches!(
                target,
                PropertyIdentity::Profile { .. }
                    | PropertyIdentity::TextureTranslation(_)
                    | PropertyIdentity::TextureRotation(_)
                    | PropertyIdentity::TextureScale(_)
            ) && !declared_properties.contains(target)
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

        for (id, clip) in &self.clips {
            let mut chain = BTreeSet::new();
            chain.insert(*id);
            let mut alias = clip.alias;
            if alias.is_some() && (!clip.channels.is_empty() || !clip.events.is_empty()) {
                return Err(Error::new(
                    "clip.alias",
                    "an alias cannot define its own channels or events",
                ));
            }
            while let Some(target) = alias {
                if !chain.insert(target) {
                    return Err(Error::new("clip.alias", "playback aliases form a cycle"));
                }
                alias = self.clips[&target].alias;
            }
        }
        for event in self
            .shared_events
            .iter()
            .chain(self.timeline_events.iter())
            .chain(self.clips.values().flat_map(|clip| clip.events.iter()))
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
            .flat_map(|(id, instance)| instance.draws.iter().map(move |draw| (*id, draw.id())))
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
                if !self.materials.contains_key(&binding.material)
                    || self
                        .instances
                        .get(&binding.instance)
                        .is_none_or(|instance| {
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
                    .and_then(|id| self.skins.get(&id))
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
