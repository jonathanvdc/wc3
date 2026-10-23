use super::super::prepared::ParticleEffect;
use super::*;
use crate::effects::age_sprites_gpu::AgeSpriteSource;
use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectLimits, ExecutionOptions, ExecutionPreference, NumericPolicy, SceneInputs,
};

fn effect() -> ParticleEffect {
    let document = Document::read(include_bytes!(
        "../../../../../wc3/tests/fixtures/pkb/immutable-128.pkb"
    ))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let program = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap()
        .with_execution(ExecutionOptions {
            preference: ExecutionPreference::ImmutableAge,
            numeric_policy: NumericPolicy::Graphics,
        });
    ParticleEffect::new(program, |_| None)
}
fn validate_shader(source: &str) {
    let mut active = vec![true];
    let mut text = String::from(
        r#"
struct View { world_from_view: mat4x4<f32>, view_from_world: mat4x4<f32>, world_position: vec3<f32> };
@group(0) @binding(0) var<uniform> view: View;
fn position_world_to_clip(p: vec3<f32>) -> vec4<f32> { return vec4(p, 1.0); }
fn shade_effect(color: vec4<f32>, position: vec3<f32>, normal: vec3<f32>, frag_coord: vec4<f32>, unlit: bool, fog_enabled: bool, fog_blend: u32) -> vec4<f32> { return color; }
"#,
    );
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#import") {
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("#ifdef ") {
            active.push(
                *active.last().unwrap()
                    && matches!(name, "SAMPLED_SPRITE" | "IMMUTABLE_APPEARANCE"),
            );
        } else if let Some(name) = trimmed.strip_prefix("#ifndef ") {
            active.push(
                *active.last().unwrap()
                    && !matches!(name, "SAMPLED_SPRITE" | "IMMUTABLE_APPEARANCE"),
            );
        } else if trimmed == "#else" {
            let previous = active.pop().unwrap();
            active.push(*active.last().unwrap() && !previous);
        } else if trimmed == "#endif" {
            active.pop().unwrap();
        } else if *active.last().unwrap() {
            text.push_str(line);
            text.push('\n');
        }
    }
    assert_eq!(active, [true]);
    let module = parse_str(&text).unwrap_or_else(|error| panic!("{}", error.emit_to_string(&text)));
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .unwrap();
}
#[test]
fn original_recipe_emits_valid_shared_geometry_and_publishes_immutable_births() {
    let effect = effect();
    let renderer = &effect.renderers[0];
    let appearance = renderer
        .immutable
        .clone()
        .expect("positive original stationary recipe");
    validate_shader(&appearance.source);
    let mut instance = effect
        .program
        .instantiate(
            EffectLimits {
                particles: 132,
                ..Default::default()
            },
            42,
        )
        .unwrap();
    instance
        .start(
            &effect.program,
            effect.program.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    for _ in 0..2 {
        instance
            .advance(&effect.program, 1. / 64., SceneInputs::default())
            .unwrap();
    }
    assert!(
        instance.immutable_time().is_some(),
        "positive publication must not be a sampled fallback"
    );
    let mut state = ImmutableRendererState::new(appearance, 132).unwrap();
    let mut data = AgeSpriteData {
        source: Default::default(),
        live_indices: Default::default(),
        uniform: Default::default(),
        style: renderer.style.clone(),
    };
    assert!(state.publish(
        renderer,
        &instance,
        Mat4::IDENTITY,
        AgeSpriteUniform::default(),
        &mut data
    ));
    assert_eq!(data.live_indices.len(), 128);
    let AgeSpriteSource::Immutable(before) = data.source.clone() else {
        panic!("route substituted CPU samples");
    };
    let indices = data.live_indices.clone();
    instance
        .advance(&effect.program, 1. / 64., SceneInputs::default())
        .unwrap();
    assert!(state.publish(
        renderer,
        &instance,
        Mat4::IDENTITY,
        AgeSpriteUniform::default(),
        &mut data
    ));
    let AgeSpriteSource::Immutable(after) = &data.source else {
        panic!("route substituted CPU samples");
    };
    assert!(
        Arc::ptr_eq(&before.births, &after.births),
        "appearance-only advance must not replace births"
    );
    assert!(
        Arc::ptr_eq(&indices, &data.live_indices),
        "unchanged live order is resident"
    );
    assert_eq!(
        data.uniform.axes[0][0],
        instance.immutable_time().unwrap() as f32
    );
    state.reset();
    instance
        .start(
            &effect.program,
            effect.program.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    for _ in 0..2 {
        instance
            .advance(&effect.program, 1. / 64., SceneInputs::default())
            .unwrap();
    }
    assert!(state.publish(
        renderer,
        &instance,
        Mat4::IDENTITY,
        AgeSpriteUniform::default(),
        &mut data
    ));
    let AgeSpriteSource::Immutable(reset) = &data.source else {
        panic!();
    };
    assert!(!Arc::ptr_eq(&before.births.lineage, &reset.births.lineage));
}
#[test]
fn denied_execution_does_not_extrapolate_the_gpu_clock() {
    let effect = effect();
    let renderer = &effect.renderers[0];
    let appearance = renderer.immutable.clone().unwrap();
    let mut instance = effect
        .program
        .instantiate(
            EffectLimits {
                particles: 132,
                ..Default::default()
            },
            42,
        )
        .unwrap();
    instance
        .start(
            &effect.program,
            effect.program.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    for _ in 0..2 {
        instance
            .advance(&effect.program, 1. / 64., SceneInputs::default())
            .unwrap();
    }
    let mut state = ImmutableRendererState::new(appearance, 132).unwrap();
    let mut data = AgeSpriteData {
        source: Default::default(),
        live_indices: Default::default(),
        uniform: Default::default(),
        style: renderer.style.clone(),
    };
    assert!(state.publish(
        renderer,
        &instance,
        Mat4::IDENTITY,
        AgeSpriteUniform::default(),
        &mut data
    ));
    instance
        .advance_with_step_budget(&effect.program, 1. / 64., SceneInputs::default(), || false)
        .unwrap();
    assert!(instance.immutable_time().is_none());
    assert!(
        !state.publish(
            renderer,
            &instance,
            Mat4::IDENTITY,
            AgeSpriteUniform::default(),
            &mut data
        ),
        "the caller must publish CPU samples after denied work"
    );
}

#[test]
fn stationary_enabled_is_sampled_once_for_both_visible_and_hidden_incarnations() {
    let mut cached = None;
    assert!(!stationary_enabled(
        &mut cached,
        7,
        100,
        AppearancePhase::Initialized,
        || false
    ));
    assert!(!stationary_enabled(
        &mut cached,
        7,
        100,
        AppearancePhase::Initialized,
        || panic!("hidden birth reevaluated")
    ));
    assert!(stationary_enabled(
        &mut cached,
        8,
        101,
        AppearancePhase::Evolved,
        || true
    ));
    assert!(stationary_enabled(
        &mut cached,
        8,
        101,
        AppearancePhase::Evolved,
        || panic!("visible birth reevaluated")
    ));
    // Both generation and serial are required: reset/reuse may repeat either one.
    assert!(!stationary_enabled(
        &mut cached,
        8,
        102,
        AppearancePhase::Evolved,
        || false
    ));
    cached = None;
    assert!(stationary_enabled(
        &mut cached,
        8,
        102,
        AppearancePhase::Evolved,
        || true
    ));
}

#[test]
fn first_authorized_phase_resamples_visibility_and_replaces_initialized_geometry() {
    let mut cache = None;
    assert!(!stationary_enabled(
        &mut cache,
        1,
        4,
        AppearancePhase::Initialized,
        || false
    ));
    assert!(stationary_enabled(
        &mut cache,
        1,
        4,
        AppearancePhase::Evolved,
        || true
    ));
    assert!(stationary_enabled(
        &mut cache,
        1,
        4,
        AppearancePhase::Evolved,
        || panic!("evolved visibility recomputed")
    ));
}

#[test]
fn initial_endpoint_births_publish_authored_header_before_first_authorized_evolution() {
    let effect = effect();
    let renderer = &effect.renderers[0];
    let mut instance = effect
        .program
        .instantiate(
            EffectLimits {
                particles: 132,
                step_seconds: 1. / 64.,
                ..Default::default()
            },
            42,
        )
        .unwrap();
    instance
        .start(
            &effect.program,
            effect.program.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    // One explicit internal step: the last of128 exact generated fractions is1.
    // The other127 births evolve their remaining intervals in this same step.
    instance
        .advance(&effect.program, 1. / 64., SceneInputs::default())
        .unwrap();
    assert!(
        instance.immutable_time().is_none(),
        "control must contain initial-only births"
    );
    let mut state = ImmutableRendererState::new(renderer.immutable.clone().unwrap(), 132).unwrap();
    let mut data = AgeSpriteData {
        source: Default::default(),
        live_indices: Default::default(),
        uniform: Default::default(),
        style: renderer.style.clone(),
    };
    assert!(state.publish(
        renderer,
        &instance,
        Mat4::IDENTITY,
        AgeSpriteUniform::default(),
        &mut data
    ));
    assert_eq!(data.live_indices.len(), 128);
    let AgeSpriteSource::Immutable(initial) = data.source.clone() else {
        panic!();
    };
    let mut initial_count = 0;
    for &slot in data.live_indices.iter() {
        let words = &initial.births.blocks[slot as usize / 64];
        let row = (slot as usize % 64) * initial.births.words_per_slot;
        let expected = instance
            .particles(renderer.layer)
            .find(|p| p.id.slot == slot)
            .unwrap();
        let phase = if expected.immutable_sample().is_some() {
            AppearancePhase::Evolved
        } else {
            initial_count += 1;
            AppearancePhase::Initialized
        };
        assert_eq!(words[row + wgsl::PHASE_WORD], phase as u32);
        let header = sprite_record(renderer, &expected, Mat4::IDENTITY).unwrap();
        assert_eq!(
            &words[row..row + 28],
            cast_slice::<_, u32>(from_ref(&header))
        );
    }
    assert!(
        initial_count > 0,
        "original control must exercise an endpoint cohort"
    );
    instance
        .advance(&effect.program, 1. / 64., SceneInputs::default())
        .unwrap();
    assert!(state.publish(
        renderer,
        &instance,
        Mat4::IDENTITY,
        AgeSpriteUniform::default(),
        &mut data
    ));
    let AgeSpriteSource::Immutable(evolved) = &data.source else {
        panic!();
    };
    assert!(
        !Arc::ptr_eq(&initial.births, &evolved.births),
        "first authorization updates each birth once"
    );
    for &slot in data.live_indices.iter() {
        let words = &evolved.births.blocks[slot as usize / 64];
        let row = (slot as usize % 64) * evolved.births.words_per_slot;
        assert_eq!(
            words[row + wgsl::PHASE_WORD],
            AppearancePhase::Evolved as u32
        );
    }
}

#[test]
fn life_only_births_remain_certified_across_old_time_horizons_without_cpu_recipe_walks() {
    let effect = effect();
    let renderer = &effect.renderers[0];
    let appearance = renderer
        .immutable
        .clone()
        .expect("actual original Life recipe");
    let recipe = effect.program.immutable_recipe(renderer.layer).unwrap();
    assert!((0..recipe.node_count()).all(|index| !matches!(
        recipe.node(index).unwrap().expression,
        ImmutableExpression::Time
    )));
    assert!(!appearance.requires_time_horizon);
    let mut instance = effect
        .program
        .instantiate(
            EffectLimits {
                particles: 132,
                step_seconds: 1. / 64.,
                ..Default::default()
            },
            42,
        )
        .unwrap();
    instance
        .start(
            &effect.program,
            effect.program.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    for _ in 0..2 {
        instance
            .advance(&effect.program, 1. / 64., SceneInputs::default())
            .unwrap();
    }
    let mut state = ImmutableRendererState::new(appearance, 132).unwrap();
    let mut data = AgeSpriteData {
        source: Default::default(),
        live_indices: Default::default(),
        uniform: Default::default(),
        style: renderer.style.clone(),
    };
    assert!(state.publish(
        renderer,
        &instance,
        Mat4::IDENTITY,
        AgeSpriteUniform::default(),
        &mut data
    ));
    let certified = state.certifications;
    assert_eq!(
        certified, 128,
        "every actual evolved birth still needs admission"
    );
    let AgeSpriteSource::Immutable(initial) = data.source.clone() else {
        panic!();
    };
    // Cross the old1s and subsequent geometric horizons while the same cohort lives.
    for _ in 0..225 {
        instance
            .advance(&effect.program, 1. / 64., SceneInputs::default())
            .unwrap();
        assert!(state.publish(
            renderer,
            &instance,
            Mat4::IDENTITY,
            AgeSpriteUniform::default(),
            &mut data
        ));
        assert_eq!(data.live_indices.len(), 128);
        assert_eq!(
            state.certifications, certified,
            "unchanged Life births were re-certified"
        );
    }
    let AgeSpriteSource::Immutable(after) = &data.source else {
        panic!();
    };
    assert!(Arc::ptr_eq(&initial.births, &after.births));
    assert_eq!(state.certified_until, f32::MAX);
    assert!(data.uniform.axes[0][0] > 2.);
}
