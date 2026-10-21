use super::*;
fn close(a: Vec3, b: Vec3) {
    assert!((a - b).length() < 1e-5, "{a:?} != {b:?}");
}
#[test]
fn thin_capsules_do_not_contact_merely_because_body_bounds_overlap() {
    let a = Primitive::Capsule {
        endpoints: [Vec3::new(-5., 0., 0.), Vec3::new(5., 0., 0.)],
        radius: 0.1,
    };
    let b = Primitive::Capsule {
        endpoints: [Vec3::new(-5., 1., 0.), Vec3::new(5., 1., 0.)],
        radius: 0.1,
    };
    assert!(query(a, b).unwrap().is_none());
}
#[test]
fn offset_rotated_capsules_have_exact_surface_witnesses_and_swap_normals() {
    let a = Primitive::Capsule {
        endpoints: [Vec3::new(2., -2., 0.), Vec3::new(2., 2., 0.)],
        radius: 0.25,
    };
    let b = Primitive::Sphere {
        center: Vec3::new(2.4, 0., 0.),
        radius: 0.25,
    };
    let c = query(a, b).unwrap().unwrap();
    close(c.normal, Vec3::X);
    assert!((c.depth - 0.1).abs() < 1e-5);
    close(c.points[0], Vec3::new(2.25, 0., 0.));
    let d = query(b, a).unwrap().unwrap();
    close(c.normal, -d.normal);
    close(c.points[0], d.points[1]);
}
#[test]
fn obb_sphere_handles_rotation_containment_and_rejects_shear() {
    let axes = Mat3::from_rotation_z(0.7);
    let center = Vec3::new(2., 3., 4.);
    let half = Vec3::new(2., 0.5, 1.);
    let b = Primitive::Box {
        center,
        axes,
        half_extents: half,
    };
    let s = Primitive::Sphere {
        center: center + axes * Vec3::new(0., 0.7, 0.),
        radius: 0.3,
    };
    let c = query(s, b).unwrap().unwrap();
    close(c.normal, -axes.y_axis);
    assert!((c.depth - 0.1).abs() < 1e-5);
    let inner = query(
        Primitive::Sphere {
            center,
            radius: 0.2,
        },
        b,
    )
    .unwrap()
    .unwrap();
    close(inner.normal, -axes.y_axis);
    assert!((inner.depth - 0.7).abs() < 1e-5);
    assert_eq!(query(b, b).unwrap_err(), QueryError::UnsupportedPair);
    assert_eq!(
        query(
            s,
            Primitive::Box {
                center,
                axes: Mat3::from_cols(Vec3::X, Vec3::Y + Vec3::X, Vec3::Z),
                half_extents: half
            }
        )
        .unwrap_err(),
        QueryError::InvalidPrimitive
    );
}
#[test]
fn parallel_overlap_uses_a_balanced_contact_point_instead_of_an_arbitrary_end() {
    let a = Primitive::Capsule {
        endpoints: [-Vec3::X * 3., Vec3::X * 3.],
        radius: 0.3,
    };
    let b = Primitive::Capsule {
        endpoints: [-Vec3::X * 2. + Vec3::Y * 0.5, Vec3::X * 2. + Vec3::Y * 0.5],
        radius: 0.3,
    };
    let contact = query(a, b).unwrap().unwrap();
    assert!(contact.points.iter().all(|point| point.x.abs() < 1e-5));
}
#[test]
fn degenerate_capsules_and_crossing_axes_stay_finite() {
    for (a, b) in [
        ([Vec3::ZERO; 2], [Vec3::ZERO; 2]),
        ([-Vec3::X, Vec3::X], [-Vec3::Y, Vec3::Y]),
    ] {
        let c = query(
            Primitive::Capsule {
                endpoints: a,
                radius: 0.2,
            },
            Primitive::Capsule {
                endpoints: b,
                radius: 0.3,
            },
        )
        .unwrap()
        .unwrap();
        assert!(c.normal.is_finite());
        assert!((c.normal.length() - 1.).abs() < 1e-5);
        assert!((c.depth - 0.5).abs() < 1e-5);
    }
}

#[test]
fn persistence_margin_retains_original_surface_witnesses() {
    let a = Primitive::Sphere {
        center: Vec3::ZERO,
        radius: 1.,
    };
    let b = Primitive::Sphere {
        center: Vec3::X * (2. + 5e-6),
        radius: 1.,
    };
    assert!(query(a, b).unwrap().is_none());
    let contact = query_margin(a, b, 1e-5).unwrap().unwrap();
    assert_eq!(contact.depth, 0.);
    assert_eq!(contact.points[0], Vec3::X);
    assert!((contact.points[1].x - (1. + 5e-6)).abs() < 1e-7);
    assert!(query_margin(a, b, 1e-7).unwrap().is_none());
}
