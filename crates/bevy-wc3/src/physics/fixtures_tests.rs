use super::super::convex::RELATIVE_TOLERANCE;
use super::*;
use bevy::math::Quat;

// For these controls A contains a centered ball of radius rho and attains
// support rho along e, while B is a sphere separated by c along e. Thus
// h(n)-depth >= c*(1-n.dot(e)) = c*|n-e|^2/2. EPA's inner-polytope
// distance is a lower bound on depth, so its support gap bounds this term.
fn rounded_control_normal_bound(left: &Fixture, body: Mat4, right: &Fixture, c: f32) -> f32 {
    let scale = f64::from(left.radius * bound_scale(linear(body))) + f64::from(right.radius);
    let chord = (2. * scale * RELATIVE_TOLERANCE / f64::from(c)).sqrt();
    // Final contact normal is converted to f32 and normalized there.
    chord as f32 + 16. * f32::EPSILON
}

fn capsule() -> Shape<'static> {
    Shape {
        geometry: ShapeGeometry::Capsule {
            radius: 0.5,
            endpoints: [-Vec3::Z, Vec3::Z],
        },
        density: 2.,
        transform: Mat4::IDENTITY,
        friction: 0.,
        restitution: 0.,
        mass_properties: None,
        game_flags: Default::default(),
        collision_mask: Default::default(),
    }
}
#[test]
fn transformed_compound_capsules_retain_center_and_anisotropic_inertia() {
    let mut a = capsule();
    a.transform =
        Mat4::from_rotation_translation(Quat::from_rotation_y(0.5), Vec3::new(2., 1., 0.));
    let mut b = capsule();
    b.transform = Mat4::from_translation(Vec3::new(-2., -1., 0.));
    let (fa, ma) = Fixture::prepare(&a);
    let (fb, mb) = Fixture::prepare(&b);
    assert!(!fa.approximate && !fb.approximate);
    assert!(matches!(fa.geometry, Primitive::Capsule { .. }));
    let compound = Mass::compound([ma, mb].into_iter()).unwrap();
    assert_eq!(compound.center, Vec3::ZERO);
    assert!(compound.inertia().x_axis.y.abs() > 0.1);
    assert!((compound.mass - ma.mass - mb.mass).abs() < 1e-5);
}
#[test]
fn nonuniform_capsule_preparation_keeps_a_conservative_bound() {
    let mut shape = capsule();
    shape.transform = Mat4::from_cols(
        Vec3::X.extend(0.),
        Vec3::new(2., 1., 0.).extend(0.),
        (Vec3::Z * 3.).extend(0.),
        Vec3::new(4., 5., 6.).extend(1.),
    );
    let (fixture, mass) = Fixture::prepare(&shape);
    assert!(fixture.approximate);
    let Primitive::Sphere { center, radius } = fixture.geometry else {
        panic!("expected conservative bound");
    };
    for z in [-1., 1.] {
        for direction in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z] {
            let point = shape
                .transform
                .transform_point3(Vec3::Z * z + direction * 0.5);
            assert!(point.distance(center) <= radius);
        }
    }
    assert!(mass.inverse_inertia(1.).is_some());
}
#[test]
fn hull_queries_borrow_prepared_vertices_and_never_use_their_bound_as_contact() {
    use super::super::contact_work::PhysicsContactLimits;
    let vertices = (0..8)
        .map(|bits| {
            Vec3::new(
                if bits & 1 == 0 { -1. } else { 1. },
                if bits & 2 == 0 { -0.03 } else { 0.03 },
                if bits & 4 == 0 { -0.4 } else { 0.4 },
            )
        })
        .collect();
    let hull = Arc::new(Hull::new(vertices));
    let mut left = Fixture::test_primitive(
        Primitive::Sphere {
            center: Vec3::ZERO,
            radius: 1.1,
        },
        Vec3::ZERO,
        1.1,
        0.,
    );
    left.hull = Some(hull.clone());
    let right = Fixture::test_primitive(
        Primitive::Sphere {
            center: Vec3::ZERO,
            radius: 0.1,
        },
        Vec3::ZERO,
        0.1,
        0.,
    );
    let mut scratch = Scratch::default();
    let mut work = ContactWork::new(PhysicsContactLimits::default());
    assert!(left
        .contact(
            Mat4::IDENTITY,
            &right,
            Mat4::from_translation(Vec3::Y * 0.3),
            0.,
            &mut work,
            Some(&mut scratch)
        )
        .is_none());
    assert_eq!(work.stats.inconclusive_queries, 0);
    assert_eq!(Arc::strong_count(&hull), 2);
    let mut tiny = ContactWork::new(PhysicsContactLimits {
        queries_per_update: 4,
        support_vertices_per_update: 1,
        ..Default::default()
    });
    assert!(left
        .contact(
            Mat4::IDENTITY,
            &right,
            Mat4::from_translation(Vec3::Y * 0.05),
            0.,
            &mut tiny,
            Some(&mut scratch)
        )
        .is_none());
    assert_eq!(tiny.stats.support_budget_exhaustions, 1);
    assert_eq!(tiny.stats.support_vertices, 1);
    // The same per-update quota still permits an analytical query afterwards.
    assert!(right
        .contact(
            Mat4::IDENTITY,
            &right,
            Mat4::from_translation(Vec3::X * 0.1),
            0.,
            &mut tiny,
            None
        )
        .is_some());
    assert_eq!(tiny.stats.queries, 2);
    assert_eq!(tiny.stats.support_vertices, 1);
    assert_eq!(Arc::strong_count(&hull), 2);
}
#[test]
fn analytical_errors_are_observable_without_changing_valid_contact_math() {
    use super::super::contact_work::PhysicsContactLimits;
    let left = Fixture::test_primitive(
        Primitive::Sphere {
            center: Vec3::ZERO,
            radius: 0.5,
        },
        Vec3::ZERO,
        0.5,
        0.,
    );
    let mut work = ContactWork::new(PhysicsContactLimits::default());
    let actual = left
        .contact(
            Mat4::IDENTITY,
            &left,
            Mat4::from_translation(Vec3::X * 0.7),
            0.,
            &mut work,
            None,
        )
        .unwrap();
    let expected = contact::query(
        Primitive::Sphere {
            center: Vec3::ZERO,
            radius: 0.5,
        },
        Primitive::Sphere {
            center: Vec3::X * 0.7,
            radius: 0.5,
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(actual.normal, expected.normal);
    assert_eq!(actual.points, expected.points);
    assert_eq!(actual.depth, expected.depth);
    assert!(left
        .contact(
            Mat4::from_translation(Vec3::NAN),
            &left,
            Mat4::IDENTITY,
            0.,
            &mut work,
            None
        )
        .is_none());
    assert_eq!(
        work.stats.last_issue,
        Some(PhysicsContactIssue::InvalidInput)
    );
    assert_eq!(work.stats.inconclusive_queries, 1);
}

fn prepared(geometry: ShapeGeometry<'static>, transform: Mat4) -> Fixture {
    Fixture::prepare(&Shape {
        geometry,
        transform,
        density: 1.,
        friction: 0.,
        restitution: 0.,
        mass_properties: None,
        game_flags: Default::default(),
        collision_mask: Default::default(),
    })
    .0
}

#[test]
fn box_and_capsule_pairs_use_the_original_surfaces_not_fixture_bounds() {
    use super::super::contact_work::PhysicsContactLimits;
    let thin = prepared(ShapeGeometry::Box(Vec3::new(1., 0.03, 0.4)), Mat4::IDENTITY);
    let small_box = prepared(
        ShapeGeometry::Box(Vec3::new(0.2, 0.05, 0.2)),
        Mat4::IDENTITY,
    );
    let cap = prepared(
        ShapeGeometry::Capsule {
            endpoints: [-Vec3::Z * 0.2, Vec3::Z * 0.2],
            radius: 0.05,
        },
        Mat4::IDENTITY,
    );
    for other in [small_box, cap] {
        let mut scratch = Scratch::default();
        let mut work = ContactWork::new(PhysicsContactLimits::default());
        assert!(thin
            .contact(
                Mat4::IDENTITY,
                &other,
                Mat4::from_translation(Vec3::Y * 0.3),
                0.,
                &mut work,
                Some(&mut scratch)
            )
            .is_none());
        assert!(work.stats.support_vertices > 0);
        assert_eq!(work.stats.inconclusive_queries, 0);
        let actual = thin
            .contact(
                Mat4::IDENTITY,
                &other,
                Mat4::from_translation(Vec3::Y * 0.05),
                0.,
                &mut work,
                Some(&mut scratch),
            )
            .unwrap();
        assert!(actual.normal.distance(Vec3::Y) < 2e-4);
        assert!((actual.depth - 0.03).abs() < 2e-5);
        assert!(
            (actual.points[0] - actual.points[1] - actual.normal * actual.depth).length() < 2e-5
        );
        assert_eq!(work.stats.inconclusive_queries, 0);
    }
}

#[test]
fn affine_rounded_and_sheared_box_contacts_preserve_native_support_geometry() {
    use super::super::contact_work::PhysicsContactLimits;
    let sphere = prepared(ShapeGeometry::Sphere(0.05), Mat4::IDENTITY);
    let mut shear = Mat4::IDENTITY;
    shear.y_axis.x = 1.;
    let shapes = [
        prepared(
            ShapeGeometry::Sphere(1.),
            Mat4::from_scale(Vec3::new(2., 0.1, 0.1)),
        ),
        prepared(
            ShapeGeometry::Capsule {
                endpoints: [-Vec3::Z * 0.2, Vec3::Z * 0.2],
                radius: 1.,
            },
            Mat4::from_scale(Vec3::new(2., 0.1, 0.1)),
        ),
        prepared(ShapeGeometry::Box(Vec3::new(0.5, 0.1, 0.2)), shear),
    ];
    for left in shapes {
        let mut scratch = Scratch::default();
        let mut work = ContactWork::new(PhysicsContactLimits::default());
        // All three actual geometries have supportY=.1; their conservative
        // bounds overlap this sphere, but actual surfaces have a .15 gap.
        assert!(left
            .contact(
                Mat4::IDENTITY,
                &sphere,
                Mat4::from_translation(Vec3::Y * 0.3),
                0.,
                &mut work,
                Some(&mut scratch)
            )
            .is_none());
        assert_eq!(work.stats.inconclusive_queries, 0);
        assert!(work.stats.support_vertices > 0);
        let hit = left
            .contact(
                Mat4::IDENTITY,
                &sphere,
                Mat4::from_translation(Vec3::Y * 0.13),
                0.,
                &mut work,
                Some(&mut scratch),
            )
            .unwrap();
        // The sheared box has a planar Y face; the rounded supports at X=Z=0
        // have the same unique Y extremum. Both give depth .1+.05-.13=.02.
        let normal_bound = if matches!(left.source, Primitive::Box { .. }) {
            3e-4
        } else {
            rounded_control_normal_bound(&left, Mat4::IDENTITY, &sphere, 0.13)
        };
        assert!(
            hit.normal.distance(Vec3::Y) <= normal_bound,
            "{hit:?}; {:?}",
            work.stats
        );
        assert!((hit.depth - 0.02).abs() < 3e-5);
        assert!((hit.points[0].y - 0.1).abs() < 3e-5);
        assert!((hit.points[1].y - 0.08).abs() < 3e-5);
        assert_eq!(work.stats.inconclusive_queries, 0);
    }
}

#[test]
fn exact_analytical_families_keep_zero_support_work_without_scratch() {
    use super::super::contact_work::PhysicsContactLimits;
    let sphere = prepared(ShapeGeometry::Sphere(0.5), Mat4::IDENTITY);
    let capsule = prepared(
        ShapeGeometry::Capsule {
            endpoints: [-Vec3::Z, Vec3::Z],
            radius: 0.1,
        },
        Mat4::IDENTITY,
    );
    let cuboid = prepared(ShapeGeometry::Box(Vec3::splat(0.5)), Mat4::IDENTITY);
    for (left, right) in [
        (sphere.clone(), sphere.clone()),
        (sphere.clone(), capsule.clone()),
        (sphere, cuboid),
        (capsule.clone(), capsule),
    ] {
        let mut work = ContactWork::new(PhysicsContactLimits {
            support_vertices_per_update: 0,
            ..Default::default()
        });
        let matrix = Mat4::from_translation(Vec3::X * 0.1);
        let actual = left
            .contact(Mat4::IDENTITY, &right, matrix, 0., &mut work, None)
            .unwrap();
        let (right_geometry, approximate) = right.transformed_geometry(matrix);
        assert!(!approximate);
        let expected = contact::query_margin(left.geometry, right_geometry, 0.)
            .unwrap()
            .unwrap();
        assert_eq!(actual.normal, expected.normal);
        assert_eq!(actual.points, expected.points);
        assert_eq!(actual.depth, expected.depth);
        assert_eq!(work.stats.support_vertices, 0);
        assert_eq!(work.stats.last_issue, None);
    }
}

#[test]
fn primitive_support_failures_remain_typed_and_never_fall_back_to_bounds() {
    use super::super::contact_work::PhysicsContactLimits;
    let cuboid = prepared(ShapeGeometry::Box(Vec3::splat(0.5)), Mat4::IDENTITY);
    let mut scratch = Scratch::default();
    let mut work = ContactWork::new(PhysicsContactLimits {
        support_vertices_per_update: 1,
        ..Default::default()
    });
    assert!(cuboid
        .contact(
            Mat4::IDENTITY,
            &cuboid,
            Mat4::from_translation(Vec3::X * 0.8),
            0.,
            &mut work,
            Some(&mut scratch)
        )
        .is_none());
    assert_eq!(
        work.stats.last_issue,
        Some(PhysicsContactIssue::SupportLimit)
    );
    assert_eq!(work.stats.support_vertices, 1);
    assert_eq!(work.stats.support_budget_exhaustions, 1);
    assert_eq!(work.stats.inconclusive_queries, 0);
    // Exhausted support still permits a supported analytical pair.
    let sphere = prepared(ShapeGeometry::Sphere(0.5), Mat4::IDENTITY);
    assert!(sphere
        .contact(
            Mat4::IDENTITY,
            &sphere,
            Mat4::from_translation(Vec3::X * 0.8),
            0.,
            &mut work,
            None
        )
        .is_some());
    assert_eq!(
        work.stats.last_issue,
        Some(PhysicsContactIssue::SupportLimit)
    );
    let mut missing = ContactWork::new(PhysicsContactLimits::default());
    assert!(cuboid
        .contact(
            Mat4::IDENTITY,
            &cuboid,
            Mat4::IDENTITY,
            0.,
            &mut missing,
            None
        )
        .is_none());
    assert_eq!(
        missing.stats.last_issue,
        Some(PhysicsContactIssue::ScratchUnavailable)
    );
    assert_eq!(missing.stats.inconclusive_queries, 1);
    assert_eq!(missing.stats.support_vertices, 0);
}

#[test]
fn zero_radius_capsule_and_point_against_solid_box_match_independent_point_oracle() {
    use super::super::contact_work::PhysicsContactLimits;
    let cuboid = prepared(ShapeGeometry::Box(Vec3::splat(0.5)), Mat4::IDENTITY);
    for endpoints in [[-Vec3::Z * 0.2, Vec3::Z * 0.2], [Vec3::ZERO; 2]] {
        let cap = prepared(
            ShapeGeometry::Capsule {
                endpoints,
                radius: 0.,
            },
            Mat4::IDENTITY,
        );
        for x in [0.49, 0.7] {
            let matrix = Mat4::from_translation(Vec3::X * x);
            let mut scratch = Scratch::default();
            let mut work = ContactWork::new(PhysicsContactLimits::default());
            let actual = cuboid.contact(
                Mat4::IDENTITY,
                &cap,
                matrix,
                0.,
                &mut work,
                Some(&mut scratch),
            );
            // The segment lies entirely within the box's Y/Z ranges, so its
            // distance/penetration along the unique X face equals this point.
            let expected = contact::query_margin(
                Primitive::Box {
                    center: Vec3::ZERO,
                    axes: Mat3::IDENTITY,
                    half_extents: Vec3::splat(0.5),
                },
                Primitive::Sphere {
                    center: Vec3::X * x,
                    radius: 0.,
                },
                0.,
            )
            .unwrap();
            assert_eq!(actual.is_some(), expected.is_some());
            assert_eq!(work.stats.inconclusive_queries, 0);
            assert!(work.stats.support_vertices > 0);
            if let (Some(actual), Some(expected)) = (actual, expected) {
                assert!(actual.normal.distance(expected.normal) < 1e-5);
                assert!((actual.depth - expected.depth).abs() < 1e-6);
                assert!((actual.points[0].x - 0.5).abs() < 1e-6);
                assert!((actual.points[1].x - x).abs() < 1e-6);
                assert!(actual.points[1].y.abs() < 1e-6);
                assert!(actual.points[1].z.abs() <= endpoints[1].z.abs() + 1e-6);
            }
        }
    }
}

#[test]
fn noncommuting_body_and_source_affines_keep_their_original_order() {
    use super::super::contact_work::PhysicsContactLimits;
    // Source sphere -> ellipsoid radii (2,.1,.1), center=(.25,0,0).
    // Body quarter-turn then translation puts its center at (1,.25,0)
    // and radii (.1,2,.1). Reversing the composition instead puts it near
    // (2.25,0,0) with radii (2,.1,.1), a completely different support set.
    let source = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 0.1, 0.1),
        Quat::IDENTITY,
        Vec3::X * 0.25,
    );
    let body = Mat4::from_cols(
        Vec3::Y.extend(0.),
        (-Vec3::X).extend(0.),
        Vec3::Z.extend(0.),
        Vec3::X.extend(1.),
    );
    let left = prepared(ShapeGeometry::Sphere(1.), source);
    let sphere = prepared(ShapeGeometry::Sphere(0.05), Mat4::IDENTITY);
    let mut scratch = Scratch::default();
    let mut work = ContactWork::new(PhysicsContactLimits::default());
    let separated = Mat4::from_translation(Vec3::new(1.3, 0.25, 0.));
    assert!(left
        .contact(body, &sphere, separated, 0., &mut work, Some(&mut scratch))
        .is_none());
    let hit = left
        .contact(
            body,
            &sphere,
            Mat4::from_translation(Vec3::new(1.13, 0.25, 0.)),
            0.,
            &mut work,
            Some(&mut scratch),
        )
        .unwrap();
    assert_eq!(work.stats.inconclusive_queries, 0);
    assert!(work.stats.support_vertices > 0);
    assert!(
        hit.normal.distance(Vec3::X)
            <= rounded_control_normal_bound(&left, body, &sphere, 1.13_f32 - 1.),
        "{hit:?}; {:?}",
        work.stats
    );
    assert!((hit.depth - 0.02).abs() < 3e-5);
    assert!((hit.points[0].x - 1.1).abs() < 3e-5);
    assert!((hit.points[1].x - 1.08).abs() < 3e-5);
    let native_witness = (body * source).inverse().transform_point3(hit.points[0]);
    assert!((native_witness.length() - 1.).abs() < 3e-5);
    let sphere_center = Vec3::new(1.13, 0.25, 0.);
    assert!((hit.points[1].distance(sphere_center) - 0.05).abs() < 3e-5);
}
