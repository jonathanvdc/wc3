//! Structural and relationship checks before publishing a graph.
use crate::model::error::{Location, Owner, ReferenceError};
use crate::model::profiles::{FeaturePayload, Profile};
use crate::model::ValueSlot;
use crate::model::{
    AppearanceSchema, BindingSchema as _, Error, GeometrySchema, Link, Model, SharedClock,
    SystemPayload, Time, ValidateLinks, VertexInfluences,
};
use std::collections::{BTreeSet, HashSet};
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
        if let Some(track) = self.tracks().find(|track| !slots.contains(&track.target())) {
            return Err(Error::at(
                Location::for_owner(track.target()),
                ReferenceError::UnconsumedSlot {
                    slot: track.target(),
                },
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
            node.transform
                .validate(&self.values)
                .map_err(|error| error.with_context(Location::for_owner(id).field("transform")))?;
            let mut ancestors = BTreeSet::new();
            ancestors.insert(id);
            let mut parent = node.parent;
            while let Some(p) = parent {
                if !ancestors.insert(p) {
                    return Err(Error::at(
                        Location::for_owner(id).field("parent"),
                        ReferenceError::Cycle { target: p.into() },
                    ));
                }
                parent = self
                    .nodes
                    .get(p)
                    .ok_or_else(|| {
                        Error::at(
                            Location::for_owner(id).field("parent"),
                            if self.scope.contains(p) {
                                ReferenceError::Unknown { target: p.into() }
                            } else {
                                ReferenceError::Foreign { target: p.into() }
                            },
                        )
                    })?
                    .parent;
            }
        }
        for (id, mesh) in self.meshes.iter() {
            mesh.validate().map_err(|error| error.with_owner(id))?;
        }
        for (id, material) in self.materials.iter() {
            material.validate().map_err(|error| error.with_owner(id))?;
        }
        for (id, clip) in self.clips.iter() {
            Time::seconds(clip.source_start.as_seconds() + clip.duration.as_seconds())
                .map_err(|error| error.with_owner(id))?;
            if clip.duration.as_seconds() < 0. {
                return Err(
                    Error::new("clip.duration", "duration must be nonnegative").with_owner(id)
                );
            }
        }
        for (id, clock) in self.clocks.iter() {
            if matches!(clock, SharedClock::Repeating { period } if period.as_seconds() <= 0.) {
                return Err(
                    Error::new("clock.period", "repeating period must be positive").with_owner(id),
                );
            }
        }
        for (id, skin) in self.skins.iter() {
            skin.binding
                .validate(skin.joints.len())
                .map_err(|error| error.with_owner(id))?;
        }
        self.validate_relationships()?;
        P::validate_model(self)?;
        P::validate_resources(self)
    }
    fn relationship_failure(&self, link: Link) -> Option<ReferenceError> {
        macro_rules! reference {
            ($id:expr, $table:expr) => {{
                let id = $id;
                if $table.contains_key(id) {
                    None
                } else if self.scope.contains(id) {
                    Some(ReferenceError::Unknown { target: id.into() })
                } else {
                    Some(ReferenceError::Foreign { target: id.into() })
                }
            }};
        }
        match link {
            Link::Value(slot) => (!self.values.contains(slot)).then_some(ReferenceError::Unknown {
                target: slot.into(),
            }),
            Link::ImageResource(id) => reference!(id, self.dependencies.images),
            Link::ModelResource(id) => reference!(id, self.dependencies.models),
            Link::EffectResource(id) => reference!(id, self.dependencies.effects),
            Link::PhysicsResource(id) => reference!(id, self.dependencies.physics),
            Link::BoneOverrideResource(id) => reference!(id, self.dependencies.bone_overrides),
            Link::SystemMember { system, member } => match self.systems.get(system) {
                None => reference!(system, self.systems),
                Some(value) => (!value.contains(member)).then_some(ReferenceError::Unknown {
                    target: Owner::SystemMember { system, member },
                }),
            },
            Link::InvalidValue => Some(ReferenceError::InvalidValue),
            Link::Node(id) => reference!(id, self.nodes),
            Link::FeatureAlternatives { id, kinds } => match self.features.get(id) {
                None => reference!(id, self.features),
                Some(feature) if !kinds.contains(&feature.payload.kind()) => {
                    Some(ReferenceError::FeatureKindMismatch {
                        target: id,
                        expected: kinds.to_vec(),
                        actual: feature.payload.kind(),
                    })
                }
                _ => None,
            },
            Link::TypedFeature { id, kind } => match self.features.get(id) {
                None => reference!(id, self.features),
                Some(feature) if feature.payload.kind() != kind => {
                    Some(ReferenceError::FeatureKindMismatch {
                        target: id,
                        expected: vec![kind],
                        actual: feature.payload.kind(),
                    })
                }
                _ => None,
            },
            Link::EventSource(id) => reference!(id, self.event_sources),
            Link::Event(id) => {
                if self.event(id).is_some() {
                    None
                } else if self.scope.contains(id) {
                    Some(ReferenceError::Unknown { target: id.into() })
                } else {
                    Some(ReferenceError::Foreign { target: id.into() })
                }
            }
            Link::Feature(id) => reference!(id, self.features),
            Link::Instance(id) => reference!(id, self.instances),
            Link::TypedMesh { id, kind } => match self.meshes.get(id) {
                None => reference!(id, self.meshes),
                Some(mesh) if mesh.geometry.kind() != kind => {
                    Some(ReferenceError::GeometryKindMismatch {
                        target: id,
                        expected: kind,
                        actual: mesh.geometry.kind(),
                    })
                }
                _ => None,
            },
            Link::Mesh(id) => reference!(id, self.meshes),
            Link::Material(id) => reference!(id, self.materials),
            Link::SampledImage(_) => None,
            Link::Clip(id) => reference!(id, self.clips),
            Link::Skin(id) => reference!(id, self.skins),
            Link::Clock(id) => reference!(id, self.clocks),
            Link::Animation(id) => reference!(id, self.animations),
            Link::System(id) => reference!(id, self.systems),
        }
    }
    fn check_relationships<T: ValidateLinks + ?Sized>(
        &self,
        owner: Owner,
        field: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.check_relationships_at(|| Location::for_owner(owner).field(field), value)
    }
    fn check_relationships_at<T: ValidateLinks + ?Sized>(
        &self,
        location: impl FnOnce() -> Location,
        value: &T,
    ) -> Result<(), Error> {
        let mut failure = None;
        value.visit_links(&mut |link| {
            if failure.is_none() {
                failure = self.relationship_failure(link);
            }
        });
        if let Some(cause) = failure {
            Err(Error::at(location(), cause))
        } else {
            Ok(())
        }
    }
    fn validate_relationships(&self) -> Result<(), Error> {
        let mut failure = None;
        self.values.visit_located_values(&mut |slot, link| {
            if failure.is_none() {
                if let Some(cause) = self.relationship_failure(link) {
                    failure = Some((slot, cause));
                }
            }
        });
        if let Some((slot, cause)) = failure {
            return Err(Error::at(Location::for_owner(slot).field("value"), cause));
        }
        for (id, source) in self.event_sources.iter() {
            self.check_relationships(id.into(), "source", source)?;
        }
        for (id, value) in self.dependencies.images.iter() {
            self.check_relationships(id.into(), "declaration", value)?;
        }
        for (id, value) in self.dependencies.models.iter() {
            self.check_relationships(id.into(), "declaration", value)?;
        }
        for (id, value) in self.dependencies.effects.iter() {
            self.check_relationships(id.into(), "declaration", value)?;
        }
        for (id, value) in self.dependencies.physics.iter() {
            self.check_relationships(id.into(), "declaration", value)?;
        }
        for (id, value) in self.dependencies.bone_overrides.iter() {
            self.check_relationships(id.into(), "declaration", value)?;
        }
        for (id, node) in self.nodes.iter() {
            self.check_relationships(id.into(), "parent", &node.parent)?;
            self.check_relationships_at(
                || {
                    Location::for_owner(id)
                        .field("transform")
                        .field("translation")
                },
                &node.transform.translation,
            )?;
            self.check_relationships_at(
                || Location::for_owner(id).field("transform").field("rotation"),
                &node.transform.rotation,
            )?;
            self.check_relationships_at(
                || Location::for_owner(id).field("transform").field("scale"),
                &node.transform.scale,
            )?;
            self.check_relationships(id.into(), "visible", &node.visible)?;
            self.check_relationships(id.into(), "properties", &node.properties)?;
        }
        for (id, clip) in self.clips.iter() {
            self.check_relationships(id.into(), "properties", &clip.properties)?;
        }
        for (index, &id) in self.global_animations.iter().enumerate() {
            if let Some(cause) = self.relationship_failure(Link::Animation(id)) {
                return Err(Error::at(
                    Location::new().field("global_animations").index(index),
                    cause,
                ));
            }
        }
        for (id, animation) in self.animations.iter() {
            self.check_relationships(id.into(), "tracks", &animation.tracks)?;
            for (event_id, event) in &animation.events {
                self.check_relationships((*event_id).into(), "source", event)?;
            }
        }
        for (id, skin) in self.skins.iter() {
            self.check_relationships(id.into(), "joints", &skin.joints)?;
            self.check_relationships(id.into(), "binding", &skin.binding)?;
        }
        for (id, mesh) in self.meshes.iter() {
            self.check_relationships(id.into(), "properties", &mesh.properties)?;
            self.check_relationships(id.into(), "geometry", &mesh.geometry)?;
        }
        for (id, instance) in self.instances.iter() {
            self.check_relationships(id.into(), "properties", &instance.properties)?;
            self.check_relationships(id.into(), "node", &instance.node)?;
            self.check_relationships(id.into(), "mesh", &instance.mesh)?;
            self.check_relationships(id.into(), "skin", &instance.skin)?;
            for draw in &instance.draws {
                self.check_relationships(draw.id().into(), "properties", &draw.properties)?;
            }
        }
        for (id, material) in self.materials.iter() {
            self.check_relationships(id.into(), "surface", &material.surface)?;
        }
        for (id, clip) in self.clips.iter() {
            self.check_relationships(id.into(), "animations", &clip.animations)?;
        }
        for (id, feature) in self.features.iter() {
            self.check_relationships(id.into(), "node", &feature.node)?;
            self.check_relationships(id.into(), "payload", &feature.payload)?;
        }
        let mut failure = None;
        self.appearance.visit_links(&mut |link| {
            if failure.is_none() {
                failure = self.relationship_failure(link);
            }
        });
        if let Some(cause) = failure {
            return Err(Error::at(Location::new().field("appearance"), cause));
        }
        let mut failure = None;
        self.metadata.visit_links(&mut |link| {
            if failure.is_none() {
                failure = self.relationship_failure(link);
            }
        });
        if let Some(cause) = failure {
            return Err(Error::at(Location::new().field("metadata"), cause));
        }
        for (id, system) in self.systems.iter() {
            self.check_relationships(id.into(), "system", system)?;
        }
        for (owner, ids) in self
            .clips
            .iter()
            .map(|(id, clip)| (Some(id), &clip.animations))
            .chain([(None, &self.global_animations)])
        {
            let mut seen = HashSet::new();
            for (index, &id) in ids.iter().enumerate() {
                if !seen.insert(id) {
                    let location = if let Some(owner) = owner {
                        Location::for_owner(owner).field("animations")
                    } else {
                        Location::new().field("global_animations")
                    };
                    return Err(Error::at(
                        location.index(index),
                        ReferenceError::Duplicate { target: id.into() },
                    ));
                }
            }
        }
        let mut event_ids = HashSet::new();
        for (owner, animation) in self.animations.iter() {
            for &id in animation.events.keys() {
                if !self.scope.contains(id) {
                    return Err(Error::at(
                        Location::for_owner(owner).field("events"),
                        ReferenceError::Foreign { target: id.into() },
                    ));
                }
                if !event_ids.insert(id) {
                    return Err(Error::at(
                        Location::for_owner(owner).field("events"),
                        ReferenceError::Duplicate { target: id.into() },
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
        for (id, instance) in self.instances.iter() {
            for draw in &instance.draws {
                if !draw_ids.insert(draw.id()) {
                    return Err(Error::at(
                        Location::for_owner(id).field("draws"),
                        ReferenceError::Duplicate {
                            target: draw.id().into(),
                        },
                    ));
                }
            }
        }
        if !expected.is_empty() && self.looks().len() == 0 {
            return Err(Error::at(
                Location::new().field("appearance"),
                ReferenceError::RequiredLook,
            ));
        }
        for (id, look) in self.looks() {
            let mut assignments = BTreeSet::new();
            for (index, binding) in look.bindings.iter().enumerate() {
                if !assignments.insert((binding.instance, binding.draw)) {
                    return Err(Error::at(
                        Location::for_owner(id).field("bindings").index(index),
                        ReferenceError::Duplicate {
                            target: binding.draw.into(),
                        },
                    ));
                }
                if !self.materials.contains_key(binding.material) {
                    return Err(Error::at(
                        Location::for_owner(id)
                            .field("bindings")
                            .index(index)
                            .field("material"),
                        ReferenceError::Unknown {
                            target: binding.material.into(),
                        },
                    ));
                }
                if self.instances.get(binding.instance).is_none_or(|instance| {
                    !instance.draws.iter().any(|draw| draw.id() == binding.draw)
                }) {
                    return Err(Error::at(
                        Location::for_owner(id)
                            .field("bindings")
                            .index(index)
                            .with_owner(binding.instance),
                        ReferenceError::Unknown {
                            target: binding.draw.into(),
                        },
                    ));
                }
            }
            if assignments != expected {
                return Err(Error::at(
                    Location::for_owner(id).field("bindings"),
                    ReferenceError::UnassignedDraws {
                        draws: expected.difference(&assignments).copied().collect(),
                    },
                ));
            }
        }
        for (id, instance) in self.instances.iter() {
            let mesh = &self.meshes[&instance.mesh];
            for draw in &instance.draws {
                if mesh.subset(draw.subset).is_none() {
                    return Err(Error::at(
                        Location::for_owner(id)
                            .field("draws")
                            .with_owner(draw.id())
                            .field("subset"),
                        ReferenceError::Unknown {
                            target: draw.subset.into(),
                        },
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
                        Error::at(
                            Location::for_owner(id).field("skin"),
                            ReferenceError::RequiredSkin,
                        )
                    })?;
                for (vertex, row) in mesh.influences().iter().enumerate() {
                    if let Some(joints) = row.into_joint_weights() {
                        for influence in joints {
                            if influence.weight > 0.
                                && influence.joint as usize >= skin.joints.len()
                            {
                                return Err(Error::at(
                                    Location::for_owner(id).field("influences").index(vertex),
                                    ReferenceError::JointOutOfBounds {
                                        joint: influence.joint as usize,
                                        joints: skin.joints.len(),
                                    },
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
