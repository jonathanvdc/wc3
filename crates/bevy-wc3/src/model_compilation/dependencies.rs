//! Validate prepared resources against their owning Universal source.
use crate::child_models::{validate as validate_child_models, ChildModelBinding, ChildModelKey};
use crate::effects::ParticleEffect;
use crate::ModelError;
use std::collections::HashMap;
use wc3::model::profiles::Universal;
use wc3::model::{FeatureId, Model, NodeFeature};

fn validate_particle_effects(
    model: &Model<'_, Universal>,
    bindings: &HashMap<FeatureId, ParticleEffect>,
) -> Result<(), ModelError> {
    for id in bindings.keys() {
        let feature = model.feature(*id).ok_or_else(|| {
            ModelError::new(format!(
                "particle programme binding {id:?} belongs to an absent or foreign feature"
            ))
        })?;
        if !matches!(feature.payload, NodeFeature::ParticleProgram(_)) {
            return Err(ModelError::new(format!(
                "particle programme binding {id:?} does not match an authored particle programme"
            )));
        }
    }
    Ok(())
}

/// Check source-scoped prepared dependency ownership before resolving images.
pub(crate) fn validate_dependencies(
    model: &Model<'_, Universal>,
    child_models: &HashMap<ChildModelKey, ChildModelBinding>,
    particle_effects: &HashMap<FeatureId, ParticleEffect>,
) -> Result<(), ModelError> {
    validate_child_models(model, child_models)?;
    validate_particle_effects(model, particle_effects)
}
