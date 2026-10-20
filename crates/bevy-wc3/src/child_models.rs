//! Role-aware model dependencies, independent of native containers.
use crate::assets::ModelAsset;
use crate::child_coordinates::CoordinatePolicy;
use crate::error::ModelError;
use bevy::asset::Handle;
use std::collections::HashMap;
use wc3::model::{
    Attachment, FeatureId, MdlxParticle1Resource, Model, ModelResourceId, NodeFeature,
    ParticleEmitter,
};

/// Authored use of an optional external model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChildModelRole {
    /// A model mounted at an attachment.
    Attachment,
    /// A complete model rendered for each particle birth.
    ParticleGeometry,
    /// A model supplying adopted particle-emitter templates.
    ParticleEmitters,
}

/// One role of one feature in the source model being compiled.
/// Feature IDs remain local to their owning semantic model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChildModelKey {
    /// Identity in the source model owning this dependency.
    pub feature: FeatureId,
    /// Authored use of the resolved model.
    pub role: ChildModelRole,
}
impl ChildModelKey {
    /// Address an authored attachment child.
    pub const fn attachment(feature: FeatureId) -> Self {
        Self {
            feature,
            role: ChildModelRole::Attachment,
        }
    }
    /// Address the model rendered by a particle.
    pub const fn particle_geometry(feature: FeatureId) -> Self {
        Self {
            feature,
            role: ChildModelRole::ParticleGeometry,
        }
    }
    /// Address a model supplying adopted particle emitters.
    pub const fn particle_emitters(feature: FeatureId) -> Self {
        Self {
            feature,
            role: ChildModelRole::ParticleEmitters,
        }
    }
}

/// A resolved model and the coordinate policy for this exact authored role.
#[derive(Clone, Debug)]
pub struct ChildModelBinding {
    /// Independently loaded child asset.
    pub model: Handle<ModelAsset>,
    /// Unit and axis interpretation for this role.
    pub coordinates: CoordinatePolicy,
}
impl ChildModelBinding {
    /// Bind a model using the host frame and units.
    pub fn new(model: Handle<ModelAsset>) -> Self {
        Self {
            model,
            coordinates: CoordinatePolicy::default(),
        }
    }
    /// Select how the child retains or replaces its standalone conversion.
    pub fn with_coordinates(mut self, coordinates: CoordinatePolicy) -> Self {
        self.coordinates = coordinates;
        self
    }
}

pub(crate) fn requests(model: &Model<'_>) -> Vec<(ChildModelKey, ModelResourceId)> {
    let mut requests = Vec::new();
    for (feature, value) in model.features() {
        match &value.payload {
            NodeFeature::Attachment(Attachment::Child(value)) => {
                requests.push((ChildModelKey::attachment(feature), value.resource))
            }
            NodeFeature::Particle(ParticleEmitter::MdlxEmitter1(value))
                if !value.details.text_resource =>
            {
                if let MdlxParticle1Resource::Model(Some(resource)) = value.details.resource {
                    requests.push((ChildModelKey::particle_geometry(feature), resource));
                }
            }
            NodeFeature::Particle(ParticleEmitter::M2(value)) => {
                if let Some(resource) = value.details.rendering.model {
                    requests.push((ChildModelKey::particle_geometry(feature), resource));
                }
                if let Some(resource) = value.details.rendering.child_model {
                    requests.push((ChildModelKey::particle_emitters(feature), resource));
                }
            }
            _ => {}
        }
    }
    requests
}

pub(crate) fn validate(
    model: &Model<'_>,
    bindings: &HashMap<ChildModelKey, ChildModelBinding>,
) -> Result<(), ModelError> {
    for key in bindings.keys() {
        let feature = model.feature(key.feature).ok_or_else(|| {
            ModelError::new(format!(
                "child model binding {:?} belongs to an absent or foreign feature",
                key
            ))
        })?;
        let valid = match (&feature.payload, key.role) {
            (NodeFeature::Attachment(Attachment::Child(_)), ChildModelRole::Attachment) => true,
            (
                NodeFeature::Particle(ParticleEmitter::MdlxEmitter1(value)),
                ChildModelRole::ParticleGeometry,
            ) => {
                !value.details.text_resource
                    && matches!(value.details.resource, MdlxParticle1Resource::Model(_))
            }
            (
                NodeFeature::Particle(ParticleEmitter::M2(value)),
                ChildModelRole::ParticleGeometry,
            ) => value.details.rendering.model.is_some(),
            (
                NodeFeature::Particle(ParticleEmitter::M2(value)),
                ChildModelRole::ParticleEmitters,
            ) => value.details.rendering.child_model.is_some(),
            _ => false,
        };
        if !valid {
            return Err(ModelError::new(format!(
                "child model binding {:?} does not match an authored model dependency",
                key
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "child_models_tests.rs"]
mod tests;
