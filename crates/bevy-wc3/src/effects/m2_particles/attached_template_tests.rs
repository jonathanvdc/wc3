//! Original controls for owner-local adopted templates and shared interpretation.
use super::*;
use crate::effects::age_sprites_gpu::AgeSpriteData;
use std::f32::consts::FRAC_PI_2;

fn template_pool(attached: bool, inherit: bool) -> (SlotValues, adopted_pool::AdoptedPool) {
    let mut values = SlotValues::default();
    let mut definition = emitter(&mut values);
    values.set(definition.rate, 4.).unwrap();
    values.set(definition.speed, 0.).unwrap();
    values.set(definition.lifespan, 4.).unwrap();
    let controls = definition.controls.as_ref().unwrap();
    values.set(controls.speed_variation, 0.).unwrap();
    values.set(controls.width, 4.).unwrap();
    values.set(controls.length, 6.).unwrap();
    definition.flags.set_attached(attached);
    definition.flags.set_inherit_velocity(inherit);
    definition.inherit_velocity = 0.;
    let template = templates::FrozenTemplate::new(&Arc::new(definition), &values);
    (values, adopted_pool::AdoptedPool::new(Arc::new(template)))
}

fn published(
    pool: &mut adopted_pool::AdoptedPool,
    values: &SlotValues,
    owner: Mat4,
    time: f64,
) -> AgeSpriteData {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.import_values(values).unwrap();
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let frame = EffectFrame {
        traversal: Default::default(),
        change: Default::default(),
        root_visible: true,
        pose: None,
        selection: Default::default(),
        time,
        sample: &sample,
        node_poses: &[Mat4::IDENTITY],
        node_visible: &[true],
        basis_world: Mat4::IDENTITY,
    };
    let mut data = AgeSpriteData {
        records: Default::default(),
        live_indices: Default::default(),
        uniform: Default::default(),
        style: pool.template.kernel().gpu_style.clone(),
    };
    pool.publish(&frame, owner, &mut data);
    data
}
fn center(data: &AgeSpriteData, index: usize) -> Vec3 {
    let position = data.records[index].position;
    Vec3::new(position[0], position[1], position[2])
}

#[test]
fn attached_births_ignore_driving_translation_and_publish_in_current_owner_frame() {
    let (values, mut attached) = template_pool(true, false);
    let (detached_values, mut detached) = template_pool(false, false);
    let placements = [
        Mat4::from_scale_rotation_translation(
            Vec3::new(2., 0.5, 1.),
            Quat::from_rotation_z(0.6),
            Vec3::new(-30., 8., 2.),
        ),
        Mat4::from_scale_rotation_translation(
            Vec3::new(2., 0.5, 1.),
            Quat::from_rotation_z(0.6),
            Vec3::new(40., -9., 7.),
        ),
    ];
    for (index, placement) in placements.into_iter().enumerate() {
        let source = adopted_pool::SourceDrive {
            start: index as f64 * 0.25,
            end: (index + 1) as f64 * 0.25,
            placement,
            basis: Mat4::IDENTITY,
            velocity: Vec3::ZERO,
            emission_scale: 1.,
        };
        attached.drive(source, &mut adopted_pool::AdoptionWork::default());
        detached.drive(source, &mut adopted_pool::AdoptionWork::default());
    }
    assert_eq!(attached.storage.births.len(), 2);
    assert_eq!(detached.storage.births.len(), 2);
    let local: Vec<_> = attached
        .storage
        .births
        .iter()
        .map(|birth| birth.origin)
        .collect();
    assert!(
        local.iter().any(|point| point.length() > 0.1),
        "nonzero local positions distinguish rotation from translation"
    );
    for (index, birth) in detached.storage.births.iter().enumerate() {
        assert!(birth
            .origin
            .abs_diff_eq(placements[index].transform_point3(local[index]), 1e-5));
    }
    let original: Vec<_> = attached
        .storage
        .births
        .iter()
        .map(|birth| (birth.time, birth.origin, birth.velocity))
        .collect();
    for owner in [
        Mat4::from_scale_rotation_translation(
            Vec3::new(1.5, 0.75, 2.),
            Quat::from_rotation_z(1.2),
            Vec3::new(2., 3., -1.),
        ),
        Mat4::from_scale_rotation_translation(
            Vec3::new(-2., 1., 0.5),
            Quat::from_rotation_y(-0.7),
            Vec3::new(-5., 1., 4.),
        ),
    ] {
        // The same clock and births are observed under a changed owner frame.
        let attached_data = published(&mut attached, &values, owner, 0.5);
        let detached_data = published(&mut detached, &detached_values, owner, 0.5);
        assert_eq!(attached_data.records.len(), 2);
        assert_eq!(detached_data.records.len(), 2);
        for (index, point) in local.iter().enumerate() {
            assert!(center(&attached_data, index).abs_diff_eq(owner.transform_point3(*point), 1e-5));
            assert!(center(&detached_data, index)
                .abs_diff_eq(detached.storage.births[index].origin, 1e-5));
        }
        assert_eq!(
            attached
                .storage
                .births
                .iter()
                .map(|birth| (birth.time, birth.origin, birth.velocity))
                .collect::<Vec<_>>(),
            original,
            "publishing owner movement does not stamp or reset births"
        );
    }
}

#[test]
fn attached_templates_keep_one_fractional_carry_across_different_parent_drives() {
    let mut values = SlotValues::default();
    let mut definition = emitter(&mut values);
    values.set(definition.rate, 1.).unwrap();
    values.set(definition.speed, 0.).unwrap();
    definition.flags.set_attached(true);
    let frozen = templates::FrozenTemplate::new(&Arc::new(definition), &values);
    let mut pool = adopted_pool::AdoptedPool::new(Arc::new(frozen));
    let mut work = adopted_pool::AdoptionWork::default();
    for x in [-10., 20.] {
        pool.drive(drive(0., 0.25, x), &mut work);
    }
    assert!(pool.storage.births.is_empty());
    for x in [30., -40.] {
        pool.drive(drive(0.25, 0.5, x), &mut work);
    }
    assert_eq!(
        pool.storage.births.len(),
        1,
        "fractional emission is shared, not one accumulator per parent"
    );
    assert_eq!(
        pool.storage.births[0].origin,
        Vec3::ZERO,
        "even the final contributing parent does not become the local origin"
    );
    assert_eq!(pool.storage.dropped, 0);
    assert_eq!(pool.storage.skipped_seconds, 0.);
}

#[test]
fn attached_template_inheritance_preserves_the_common_inverse_owner_policy() {
    let (values, mut pool) = template_pool(true, true);
    // Local +X is world +Y with scale2. Inherited world +6Y must therefore
    // become local +3X under the shared interpreter, not local +6Y.
    let owner = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 3., 1.),
        Quat::from_rotation_z(FRAC_PI_2),
        Vec3::new(8., -4., 1.),
    );
    let mut source = drive(0., 0.25, 0.);
    source.placement = owner;
    source.velocity = Vec3::Y * 6.;
    pool.drive(source, &mut adopted_pool::AdoptionWork::default());
    assert_eq!(pool.storage.births.len(), 1);
    assert!(pool.storage.births[0]
        .velocity
        .abs_diff_eq(Vec3::X * 3., 1e-5));
    let data = published(&mut pool, &values, owner, 0.25);
    let velocity = data.records[0].velocity_tail;
    assert!(Vec3::new(velocity[0], velocity[1], velocity[2]).abs_diff_eq(Vec3::Y * 6., 1e-5));
}
