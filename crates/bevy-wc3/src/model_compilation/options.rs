//! Application-selected model compilation and resource policy.
use crate::child_models::{ChildModelBinding, ChildModelKey};
use crate::composition::DrawSelection;
use crate::effects::ParticleEffect;
use crate::physics::DEFAULT_CONTACT_PAIR_BUDGET;
use std::collections::HashMap;
use wc3::model::scene::popcorn::{ExecutionOptions, ExecutionPreference};
use wc3::model::{FeatureId, MaterialSelection};

/// Application-selected authored state used during immutable model compilation.
#[derive(Clone, Debug)]
pub struct CompileOptions {
    /// Material appearance. Ambiguous defaults require an explicit selection.
    pub material: MaterialSelection,
    /// Authored geometry selection, independent of automatic LOD.
    pub draws: DrawSelection,
    /// Resolved optional models, keyed by their source feature and authored role.
    pub child_models: HashMap<ChildModelKey, ChildModelBinding>,
    /// Prepared external particle programmes, keyed by their authored Universal programme feature.
    /// Absent, foreign or wrong-family keys fail before image resolution, even for hidden geometry.
    pub particle_effects: HashMap<FeatureId, ParticleEffect>,
    /// Preparation policy for external programmes resolved by the asset loader.
    /// Already prepared `particle_effects` retain their own captured execution plan.
    /// Defaults to Automatic with `popcorn-jit`, otherwise Interpreter.
    pub particle_execution: ExecutionOptions,
    /// Maximum potential unconnected-dynamic body pairs prepared across source physics systems.
    /// Sources remain isolated; their shared bound is checked before joint exclusions or quadratic enumeration.
    /// Zero disables preparation when any potential pairs exist. Composition also caps retained remapped pairs.
    pub physics_contact_pair_budget: usize,
}
impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            material: Default::default(),
            draws: Default::default(),
            child_models: Default::default(),
            particle_effects: Default::default(),
            particle_execution: ExecutionOptions {
                preference: if cfg!(feature = "popcorn-jit") {
                    ExecutionPreference::Automatic
                } else {
                    ExecutionPreference::Interpreter
                },
                ..Default::default()
            },
            physics_contact_pair_budget: DEFAULT_CONTACT_PAIR_BUDGET,
        }
    }
}
