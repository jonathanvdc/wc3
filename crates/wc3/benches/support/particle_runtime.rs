//! Shared original particle-runtime fixture admission and authored-output oracle.
use wc3::model::scene::popcorn::{
    EffectFieldId, EffectInstance, EffectLayerId, EffectLimits, EffectProgram, SceneInputs,
    ValueType,
};

pub(super) const DT: f64 = 1. / 64.;
pub(super) const WARMUP: usize = 2;
pub(super) const STEPS: usize = 64;

pub(super) fn fixture(children: usize) -> &'static [u8] {
    match children {
        1 => include_bytes!("../fixtures/particle_runtime/burst-1.pkb"),
        8 => include_bytes!("../fixtures/particle_runtime/burst-8.pkb"),
        128 => include_bytes!("../fixtures/particle_runtime/burst-128.pkb"),
        1024 => include_bytes!("../fixtures/particle_runtime/burst-1024.pkb"),
        _ => unreachable!(),
    }
}

pub(super) fn limits(capacity: usize) -> EffectLimits {
    EffectLimits {
        particles: capacity + 1, // One renderer-free controller is also resident.
        pending_events: capacity + 1,
        occurrences: capacity + 1,
        ..Default::default()
    }
}

pub(super) fn assert_stable(instance: &EffectInstance, children: usize) {
    assert_eq!(instance.active_count(), children + 1);
    let stats = instance.stats();
    assert_eq!(stats.spawned, (children + 1) as u64);
    assert_eq!(stats.retired, 0);
    assert_eq!(stats.omitted_births, 0);
    assert_eq!(stats.omitted_executions, 0);
    assert_eq!(stats.suppressed_births, 0);
    assert_eq!(stats.omitted_spatial_writes, 0);
    assert_eq!(stats.omitted_spatial_queries, 0);
    assert_eq!(stats.skipped_seconds, 0.);
}

pub(super) fn warm(program: &EffectProgram, children: usize, capacity: usize) -> EffectInstance {
    let mut instance = program.instantiate(limits(capacity), 42).unwrap();
    instance
        .start(program, program.default_entry(), SceneInputs::default())
        .unwrap();
    for _ in 0..WARMUP {
        instance
            .advance(program, DT, SceneInputs::default())
            .unwrap();
    }
    assert_stable(&instance, children);
    assert_eq!(instance.particles(EffectLayerId(0)).count(), 1);
    assert_eq!(instance.particles(EffectLayerId(1)).count(), children);
    assert_eq!(instance.time(), WARMUP as f64 * DT);
    instance
}

// Independent authored-output oracle, deliberately outside all Criterion clocks.
// The generator's first Hermite segment has equal endpoint tangents, so its
// closed cubic is p0 + m*t + (3*delta-3*m)*t² + (2*m-2*delta)*t³.
fn expected_fields(time: f64) -> [[f64; 4]; 14] {
    let t = time / 16.;
    assert!(
        t > 0. && t < 1.,
        "the entire measured trajectory must interpolate"
    );
    let p0 = [0.125, 0.25, 0.5, 1.];
    let p1 = [0.5, 0.375, 0.25, 0.75];
    let tangent = [0.25, 0.125, -0.125, -0.125];
    let mut color = [0.; 4];
    for i in 0..4 {
        let delta = p1[i] - p0[i];
        color[i] = (p0[i]
            + tangent[i] * t
            + (3. * delta - 3. * tangent[i]) * t * t
            + (2. * tangent[i] - 2. * delta) * t * t * t)
            * p0[i];
    }
    [
        [0.; 4],
        [0.; 4],
        [0.; 4],
        [1., 0., 0., 0.],
        [0.75, 0., 0., 0.],
        p0,
        [0.25, 0., 0., 0.],
        [0.0625, 0., 0., 0.],
        [0.; 4],
        [(1. + t) * 0.25 * (0.75 * 0.5 + 0.5), 0., 0., 0.],
        [0.25 + t - 0.125, 0., 0., 0.],
        [0.; 4], // Enabled is checked as exact boolean bits below.
        color,
        [0.; 4],
    ]
}

fn verify_layout(program: &EffectProgram) {
    let names: &[&[u8]] = &[
        b"self.lifeRatio",
        b"self.invLife",
        b"basePosition",
        b"baseSize",
        b"baseAlpha",
        b"baseColor",
        b"baseRotation",
        b"coefficient",
        b"Position",
        b"Size",
        b"Rotation",
        b"Enabled",
        b"Color",
        b"TextureID",
    ];
    assert_eq!(program.layers().len(), 2);
    assert_eq!(program.renderers().len(), 1);
    let child = &program.layers()[1];
    assert_eq!(child.fields.len(), names.len());
    for (field, expected) in child.fields.iter().zip(names) {
        assert_eq!(field.name.as_ref(), *expected);
    }
    let renderer = &program.renderers()[0];
    assert_eq!(renderer.layer, EffectLayerId(1));
    assert_eq!(renderer.inputs.len(), 6);
    for (input, (semantic, field, name)) in renderer.inputs.iter().zip([
        (0, 8, None),
        (1, 9, None),
        (2, 11, None),
        (100, 10, Some(b"Rotation".as_slice())),
        (100, 12, Some(b"Color".as_slice())),
        (100, 13, Some(b"TextureID".as_slice())),
    ]) {
        assert_eq!(input.semantic, semantic);
        assert_eq!(input.field, EffectFieldId(field));
        assert_eq!(input.additional_name.as_deref(), name);
    }
}

pub(super) fn verify_frame(instance: &EffectInstance, children: usize) {
    assert_stable(instance, children);
    let mut roots = instance.particles(EffectLayerId(0));
    let root = roots.next().unwrap();
    assert!(roots.next().is_none());
    assert!(root.parent.is_none());
    assert_eq!(root.birth_time, 0.);
    let expected = expected_fields(instance.time());
    let mut seen = vec![false; children];
    for child in instance.particles(EffectLayerId(1)) {
        assert_eq!(child.parent, Some(root.id));
        assert!(child.birth_time > 0. && child.birth_time <= DT);
        let serial = usize::try_from(child.birth_serial).unwrap() - 1;
        assert!(serial < children && !seen[serial]);
        seen[serial] = true;
        for (index, values) in expected.iter().enumerate() {
            let field = child.field(EffectFieldId(index as u32)).unwrap();
            let kind = match index {
                2 | 8 => ValueType::Float3,
                5 | 12 => ValueType::Float4,
                11 => ValueType::Bool,
                _ => ValueType::Float,
            };
            assert_eq!(field.kind(), kind);
            let words = field.words();
            if index == 11 {
                assert_eq!(words, [u32::MAX, 0, 0, 0]);
                continue;
            }
            for component in 0..kind.components() {
                let actual = f64::from(f32::from_bits(words[component]));
                // Independent f64 polynomial versus bounded f32 arithmetic;
                // constants/zero pass-through fields remain exact below.
                if matches!(index, 9 | 10 | 12) {
                    let error = 32. * f64::from(f32::EPSILON) * values[component].abs().max(1.);
                    assert!(
                        actual.is_finite() && (actual - values[component]).abs() <= error,
                        "frame {} field {index} component {component}: {actual} != {}",
                        instance.time(),
                        values[component]
                    );
                } else {
                    assert_eq!(words[component], (values[component] as f32).to_bits());
                }
            }
            assert!(words[kind.components()..].iter().all(|word| *word == 0));
        }
    }
    assert!(seen.into_iter().all(|present| present));
}

pub(super) fn verify_trajectory(program: &EffectProgram, children: usize, capacity: usize) {
    verify_layout(program);
    let mut probe = warm(program, children, capacity);
    verify_frame(&probe, children);
    for _ in 0..STEPS {
        probe.advance(program, DT, SceneInputs::default()).unwrap();
        verify_frame(&probe, children);
    }
    assert_eq!(probe.time(), (WARMUP + STEPS) as f64 * DT);
}
