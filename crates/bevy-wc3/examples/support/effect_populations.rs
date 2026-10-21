//! Published effect counters; these are not inferred GPU draw or quad counts.
use bevy::prelude::*;
use bevy_wc3::{
    ModelEmitter, ModelParticleTreeStats, ParticleProgramEmitter, RibbonEmitter, SpriteEmitter,
};
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Population {
    pub emitters: usize,
    pub alive: usize,
    pub omitted: u64,
    pub skipped_seconds: f64,
}
#[derive(Default, Serialize)]
pub struct Ribbons {
    pub emitters: usize,
    pub sections: usize,
    pub segments: usize,
    pub omitted_sections: u64,
}
#[derive(Default, Serialize)]
pub struct Programmes {
    pub emitters: usize,
    pub alive: usize,
    pub sprites: usize,
    pub ribbon_segments: usize,
    pub omitted_geometry: usize,
    pub omitted_spatial_writes: u64,
    pub omitted_spatial_queries: u64,
    pub omitted: u64,
    pub suppressed: u64,
    pub skipped_seconds: f64,
}
#[derive(Default, Serialize)]
pub struct ModelTrees {
    pub trees: usize,
    pub actors: usize,
    pub pending: usize,
    pub omitted_actor_requests: u64,
    pub skipped_emission_seconds: f64,
}
#[derive(Default, Serialize)]
pub struct EffectPopulations {
    pub sprites: Population,
    pub models: Population,
    pub ribbons: Ribbons,
    pub programmes: Programmes,
    pub model_trees: ModelTrees,
}
pub fn collect(world: &mut World) -> EffectPopulations {
    let mut result = EffectPopulations::default();
    let mut sprites = world.query::<&SpriteEmitter>();
    for emitter in sprites.iter(world) {
        result.sprites.emitters += 1;
        result.sprites.alive += emitter.alive;
        result.sprites.omitted = result.sprites.omitted.saturating_add(emitter.dropped);
        result.sprites.skipped_seconds += emitter.skipped_seconds;
    }
    let mut models = world.query::<&ModelEmitter>();
    for emitter in models.iter(world) {
        result.models.emitters += 1;
        result.models.alive += emitter.alive;
        result.models.omitted = result.models.omitted.saturating_add(emitter.omitted);
        result.models.skipped_seconds += emitter.skipped_seconds;
    }
    let mut ribbons = world.query::<&RibbonEmitter>();
    for emitter in ribbons.iter(world) {
        result.ribbons.emitters += 1;
        result.ribbons.sections += emitter.sections;
        result.ribbons.segments += emitter.segments;
        result.ribbons.omitted_sections = result
            .ribbons
            .omitted_sections
            .saturating_add(emitter.omitted_sections);
    }
    let mut programmes = world.query::<&ParticleProgramEmitter>();
    for emitter in programmes.iter(world) {
        result.programmes.emitters += 1;
        result.programmes.alive += emitter.alive;
        result.programmes.sprites += emitter.sprites;
        result.programmes.ribbon_segments += emitter.ribbon_segments;
        result.programmes.omitted_geometry += emitter.omitted_geometry;
        result.programmes.omitted = result.programmes.omitted.saturating_add(emitter.omitted);
        result.programmes.omitted_spatial_writes = result
            .programmes
            .omitted_spatial_writes
            .saturating_add(emitter.omitted_spatial_writes);
        result.programmes.omitted_spatial_queries = result
            .programmes
            .omitted_spatial_queries
            .saturating_add(emitter.omitted_spatial_queries);
        result.programmes.suppressed = result
            .programmes
            .suppressed
            .saturating_add(emitter.suppressed);
        result.programmes.skipped_seconds += emitter.skipped_seconds;
    }
    let mut trees = world.query::<&ModelParticleTreeStats>();
    for tree in trees.iter(world) {
        result.model_trees.trees += 1;
        result.model_trees.actors += tree.actors;
        result.model_trees.pending += tree.pending;
        result.model_trees.omitted_actor_requests = result
            .model_trees
            .omitted_actor_requests
            .saturating_add(tree.omitted_actor_requests());
        result.model_trees.skipped_emission_seconds += tree.skipped_emission_seconds;
    }
    result
}
