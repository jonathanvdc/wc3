use super::*;
fn shape(primitive: Primitive, domain: Domain) -> Shape {
    Shape {
        primitive,
        domain,
        dimensions: Vec3::new(2., 4., 6.),
        radius: 2.,
        inner_radius: 1.,
        height: 6.,
        hemisphere: false,
        scale: Vec3::ONE,
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    }
}
fn draws(index: u32) -> [f32; 6] {
    let mut state = crate::model::scene::popcorn::runtime::RandomState {
        seed: u64::from(index),
        counter: 0,
    };
    std::array::from_fn(|_| (state.next() >> 40) as f32 / 16777216.)
}
#[test]
fn sphere_volume_has_uniform_shell_volume_and_surface_uses_area_weights() {
    let volume = shape(Primitive::Sphere, Domain::Volume);
    let surface = shape(Primitive::Sphere, Domain::Surface);
    let mut cubes = 0.;
    let mut inner = 0;
    let mut mean = Vec3::ZERO;
    for i in 0..32768 {
        let d = draws(i);
        let point = volume.sample(d).unwrap();
        let radius = point.length();
        assert!((0.9999..=2.0001).contains(&radius));
        cubes += radius.powi(3);
        mean += point;
        let r = surface.sample(d).unwrap().length();
        if (r - 1.).abs() < 1e-4 {
            inner += 1
        } else {
            assert!((r - 2.).abs() < 1e-4)
        }
    }
    assert!((cubes / 32768. - 4.5).abs() < 0.08);
    assert!((inner as f32 / 32768. - 0.2).abs() < 0.015);
    assert!((mean / 32768.).length() < 0.04);
}
#[test]
fn cylinder_volume_uses_declared_radial_policy_and_surface_has_no_caps() {
    let volume = shape(Primitive::Cylinder, Domain::Volume);
    let surface = shape(Primitive::Cylinder, Domain::Surface);
    let d = [0., 0.5, 0.25, 0., 0., 0.];
    let p = volume.sample(d).unwrap();
    assert!((p.x - (1. + 0.25f32.powf(0.75))).abs() < 1e-6);
    assert_eq!(p.z, 0.);
    for i in 0..4096 {
        let p = surface.sample(draws(i)).unwrap();
        let r = p.truncate().length();
        assert!((r - 1.).abs() < 1e-4 || (r - 2.).abs() < 1e-4);
        assert!(p.z.abs() <= 3.);
    }
}
#[test]
fn box_surface_is_area_weighted_and_degenerate_shapes_remain_legal() {
    let box_shape = shape(Primitive::Box, Domain::Surface);
    let mut faces = [0; 3];
    for i in 0..32768 {
        let p = box_shape.sample(draws(i)).unwrap();
        if (p.x.abs() - 1.).abs() < 1e-5 {
            faces[0] += 1
        } else if (p.y.abs() - 2.).abs() < 1e-5 {
            faces[1] += 1
        } else {
            assert!((p.z.abs() - 3.).abs() < 1e-5);
            faces[2] += 1
        }
    }
    for (actual, expected) in faces.into_iter().zip([24. / 44., 12. / 44., 8. / 44.]) {
        assert!((actual as f32 / 32768. - expected).abs() < 0.02)
    }
    for dims in [Vec3::ZERO, Vec3::new(0., 2., 0.), Vec3::new(0., 2., 4.)] {
        let mut s = box_shape.clone();
        s.dimensions = dims;
        s.validate().unwrap();
        for i in 0..64 {
            let p = s.sample(draws(i)).unwrap();
            assert!(p.abs().cmple(dims * 0.5 + Vec3::splat(1e-6)).all())
        }
    }
    let mut disk = shape(Primitive::Cylinder, Domain::Volume);
    disk.height = 0.;
    disk.radius = 0.;
    disk.inner_radius = 0.;
    disk.validate().unwrap();
    assert_eq!(disk.sample([0.5; 6]).unwrap(), Vec3::ZERO);
}
#[test]
fn ellipsoid_fold_rotation_and_translation_apply_once() {
    let mut s = shape(Primitive::Ellipsoid, Domain::Surface);
    s.radius = 1.;
    s.inner_radius = 0.;
    s.hemisphere = true;
    s.scale = Vec3::new(2., 3., 4.);
    s.rotation = Shape::rotation(Vec3::new(90., 0., 90.));
    s.translation = Vec3::new(10., 20., 30.);
    // South pole is folded to +Z, scaled, rotated X then Z, and translated.
    let p = s.sample([0., 0., 0., 0., 0., 0.]).unwrap();
    assert!((p - Vec3::new(14., 20., 30.)).length() < 1e-5);
}
#[test]
fn capsule_and_cone_stay_within_their_declared_domains() {
    for domain in [Domain::Volume, Domain::Surface] {
        let mut capsule = shape(Primitive::Capsule, domain);
        capsule.inner_radius = 0.;
        let mut cone = shape(Primitive::Cone, domain);
        cone.inner_radius = 0.;
        for i in 0..8192 {
            let d = draws(i);
            let p = capsule.sample(d).unwrap();
            let segment = Vec3::new(0., 0., p.z.clamp(-3., 3.));
            let distance = (p - segment).length();
            assert!(distance <= 2.0001);
            if domain == Domain::Surface {
                assert!((distance - 2.).abs() < 1e-4)
            }
            let p = cone.sample(d).unwrap();
            assert!((0.0..=6.).contains(&p.z));
            let radius = 2. * (1. - p.z / 6.);
            assert!(p.truncate().length() <= radius + 1e-4);
            if domain == Domain::Surface {
                assert!((p.truncate().length() - radius).abs() < 1e-4)
            }
        }
    }
}
