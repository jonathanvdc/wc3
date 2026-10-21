use super::*;

#[test]
fn driving_camera_conversion_preserves_world_position_and_converts_axes_and_units_once() {
    let basis = Mat4::from_scale_rotation_translation(
        Vec3::new(-2., 3., 0.5),
        Quat::from_rotation_y(0.6),
        Vec3::new(20., -40., 80.),
    );
    let camera_world = Mat4::from_translation(Vec3::new(600., 400., -200.));
    let camera_model = Affine3A::from_mat4(basis.inverse() * camera_world);
    let space_from_world = Mat4::from_rotation_x(FRAC_PI_2);
    let scene = scene_context(basis, Some(camera_model), space_from_world);
    assert!(
        scene
            .camera_position(0)
            .unwrap()
            .distance(Vec3::new(6., 2., 4.))
            < 1e-5
    );
    let second = scene_context(
        basis,
        Some(Affine3A::from_mat4(
            basis.inverse() * Mat4::from_translation(Vec3::new(100., 200., 300.)),
        )),
        space_from_world,
    );
    assert!(
        second
            .camera_position(0)
            .unwrap()
            .distance(Vec3::new(1., -3., 2.))
            < 1e-5
    );
    assert!(
        scene
            .camera_position(0)
            .unwrap()
            .distance(Vec3::new(6., 2., 4.))
            < 1e-5,
        "another instance's explicitly selected camera cannot change a prior snapshot"
    );
}
#[test]
fn unavailable_driving_camera_does_not_invent_world_origin_or_fail_the_programme() {
    assert!(scene_context(Mat4::IDENTITY, None, Mat4::IDENTITY)
        .camera_position(0)
        .is_none());
    assert!(scene_context(
        Mat4::IDENTITY,
        Some(Affine3A::from_translation(Vec3::splat(f32::INFINITY))),
        Mat4::IDENTITY,
    )
    .camera_position(0)
    .is_none());
}

#[test]
fn native_camera_inverse_callback_chain_suppresses_only_unavailable_render_geometry() {
    use wc3::model::formats::pkb::{Document, EffectCompileOptions};
    let program = Document::read(include_bytes!(
        "../../../../wc3/tests/fixtures/pkb/camera.pkb"
    ))
    .unwrap()
    .compile_effect(1, &EffectCompileOptions::default())
    .unwrap();
    let effect = ParticleEffect::new(program.clone(), |_| Some(Handle::default()));
    let renderer = &effect.renderers[0];
    let mut instance = program.instantiate(Default::default(), 0).unwrap();
    instance.start(&program, 0, SceneInputs::default()).unwrap();
    instance
        .advance(&program, 0.01, SceneInputs::default())
        .unwrap();
    let particle = instance.particles(renderer.layer).next().unwrap();
    assert!(sample_point(renderer, &particle, Mat4::IDENTITY).is_none());
    assert!(sprite_record(renderer, &particle, Mat4::IDENTITY).is_none());
    assert_eq!(instance.active_count(), 1);
    instance
        .advance(
            &program,
            0.01,
            SceneInputs {
                scene: SceneContext::with_camera(Vec3::new(1., 2., 3.)).unwrap(),
                ..Default::default()
            },
        )
        .unwrap();
    let particle = instance.particles(renderer.layer).next().unwrap();
    assert_eq!(
        sample_point(renderer, &particle, Mat4::IDENTITY)
            .unwrap()
            .position,
        Vec3::new(100., 200., 300.)
    );
    assert!(sprite_record(renderer, &particle, Mat4::IDENTITY).is_some());
}
