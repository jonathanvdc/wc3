use super::super::contact;
use super::super::contact_work::PhysicsContactLimits;
use super::*;
use bevy::math::{EulerRot, Mat3, Quat};

fn box_vertices(half: Vec3) -> Vec<Vec3> {
    (0..8)
        .map(|bits| {
            Vec3::new(
                if bits & 1 == 0 { -half.x } else { half.x },
                if bits & 2 == 0 { -half.y } else { half.y },
                if bits & 4 == 0 { -half.z } else { half.z },
            )
        })
        .collect()
}
fn work() -> ContactWork {
    ContactWork::new(PhysicsContactLimits {
        queries_per_update: 1024,
        support_vertices_per_update: 1_000_000,
        ..Default::default()
    })
}
fn sphere(center: Vec3, radius: f32) -> ShapeRef<'static> {
    ShapeRef::new(
        Shape::Primitive(Primitive::Sphere { center, radius }),
        Mat4::IDENTITY,
    )
    .unwrap()
}
fn run(
    a: ShapeRef<'_>,
    b: ShapeRef<'_>,
    margin: f64,
    scale: f64,
) -> Result<Option<Contact>, Issue> {
    query(a, b, margin, scale, &mut Scratch::default(), &mut work())
}
fn close(a: Vec3, b: Vec3, tolerance: f32) {
    assert!(a.distance(b) < tolerance, "{a:?} != {b:?} ({tolerance})");
}
fn assert_witness(contact: Contact, tolerance: f32) {
    assert!((contact.normal.length() - 1.).abs() < 1e-6);
    assert!(
        (contact.points[0] - contact.points[1] - contact.normal * contact.depth).length()
            < tolerance,
        "witnesses {:?} normal {:?} depth {}",
        contact.points,
        contact.normal,
        contact.depth
    );
}
#[test]
fn thin_hull_rejects_bounding_sphere_false_positive() {
    let hull = Hull::new(box_vertices(Vec3::new(1., 0.0347222, 0.35)));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    assert!(run(a, sphere(Vec3::new(0., 0.3, 0.), 0.1), 0., 2.)
        .unwrap()
        .is_none());
    let hit = run(a, sphere(Vec3::new(0.17, 0.1, 0.08), 0.1), 0., 2.)
        .unwrap()
        .unwrap();
    assert!((hit.depth - 0.0347222).abs() < 2e-5);
    close(hit.normal, Vec3::Y, 2e-4);
    assert_witness(hit, 2e-5);
}
#[test]
fn original_surface_witnesses_swap_and_margin_never_inflate_shapes() {
    let hull = Hull::new(box_vertices(Vec3::new(1., 0.6, 0.4)));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    let center = Vec3::new(1.18, 0.13, -0.07);
    let b = sphere(center, 0.3);
    let forward = run(a, b, 0., 2.).unwrap().unwrap();
    let reverse = run(b, a, 0., 2.).unwrap().unwrap();
    close(forward.normal, -reverse.normal, 1e-4);
    close(forward.points[0], reverse.points[1], 1e-4);
    close(forward.points[1], reverse.points[0], 1e-4);
    assert!((forward.depth - 0.12).abs() < 2e-5);
    assert!((forward.points[1].distance(center) - 0.3).abs() < 2e-5);
    assert_witness(forward, 2e-5);
    let b = sphere(Vec3::new(1.35, 0.13, -0.07), 0.3);
    assert!(run(a, b, 0., 2.).unwrap().is_none());
    let near = run(a, b, 0.051, 2.).unwrap().unwrap();
    assert_eq!(near.depth, 0.);
    assert!((near.points[0].distance(near.points[1]) - 0.05).abs() < 2e-5);
    assert!((near.points[1].distance(Vec3::new(1.35, 0.13, -0.07)) - 0.3).abs() < 2e-5);
}
#[test]
fn rotated_mirrored_nonuniform_boxes_match_independent_analytical_sphere_oracle() {
    let half = Vec3::new(0.7, 0.45, 0.9);
    let hull = Hull::new(box_vertices(half));
    let mut seed = 173_u32;
    let mut random = || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) as f32 / (1_u32 << 24) as f32
    };
    for index in 0..96 {
        let rotation = Quat::from_euler(EulerRot::XYZ, random() * 5., random() * 5., random() * 5.);
        let scale = Vec3::new(
            if index & 1 == 0 {
                -0.6 - random()
            } else {
                0.6 + random()
            },
            0.6 + random(),
            if index & 2 == 0 {
                -0.6 - random()
            } else {
                0.6 + random()
            },
        );
        let translation = Vec3::new(random() * 4. - 2., random() * 4. - 2., random() * 4. - 2.);
        let matrix = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        let axes = Mat3::from_quat(rotation) * Mat3::from_diagonal(scale.signum());
        let extent = half * scale.abs();
        let radius = 0.21;
        let separation = match index % 3 {
            0 => -0.09,
            1 => 0.04,
            _ => 0.21,
        };
        // A unique +X face nearest point keeps the analytic witness unambiguous.
        let center = translation
            + axes
                * Vec3::new(
                    extent.x + radius + separation,
                    extent.y * 0.17,
                    -extent.z * 0.23,
                );
        let native_box = Primitive::Box {
            center: translation,
            axes,
            half_extents: extent,
        };
        let native_sphere = Primitive::Sphere { center, radius };
        let oracle = contact::query_margin(native_box, native_sphere, 0.05).unwrap();
        let actual = run(
            ShapeRef::new(Shape::Hull(&hull), matrix).unwrap(),
            sphere(center, radius),
            0.05,
            4.,
        )
        .unwrap();
        assert_eq!(actual.is_some(), oracle.is_some(), "case {index}");
        if let (Some(a), Some(b)) = (actual, oracle) {
            assert!(
                (a.depth - b.depth).abs() < 4e-5,
                "case {index}: {} != {}",
                a.depth,
                b.depth
            );
            close(a.normal, b.normal, 3e-4);
            close(a.points[0], b.points[0], 1e-4);
            close(a.points[1], b.points[1], 1e-4);
            if a.depth > 0. {
                assert_witness(a, 4e-5);
            }
        }
    }
}
#[test]
fn affine_support_uses_transpose_and_original_vertex_identity() {
    let vertices = box_vertices(Vec3::new(0.8, 0.25, 0.5));
    let hull = Hull::new(vertices.clone());
    let matrix = Mat4::from_cols(
        Vec3::new(-2., 0.2, 0.).extend(0.),
        Vec3::new(1.3, 0.6, 0.).extend(0.),
        Vec3::new(0.1, 0.3, 1.7).extend(0.),
        Vec3::new(2., -3., 4.).extend(1.),
    );
    let shape = ShapeRef::new(Shape::Hull(&hull), matrix).unwrap();
    for direction in [
        DVec3::X,
        DVec3::Y,
        DVec3::Z,
        DVec3::new(-0.3, 1.2, 0.4),
        DVec3::new(0.9, -0.8, 0.7),
    ] {
        let actual = shape.support(direction, &mut work()).unwrap();
        // Independently transform all vertices then select in world coordinates.
        let mut expected = None;
        let mut score = f64::NEG_INFINITY;
        for point in &vertices {
            let point = matrix.transform_point3(*point).as_dvec3();
            let candidate = point.dot(direction);
            if candidate > score {
                score = candidate;
                expected = Some(point);
            }
        }
        assert!(actual.distance(expected.unwrap()) < 1e-6);
    }
}
#[test]
fn hull_hull_and_capsule_contacts_retain_witness_consistency() {
    let hull = Hull::new(box_vertices(Vec3::new(0.7, 0.5, 0.4)));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    let b = ShapeRef::new(
        Shape::Hull(&hull),
        Mat4::from_translation(Vec3::new(1.2, 0.1, 0.03)),
    )
    .unwrap();
    let hit = run(a, b, 0., 3.).unwrap().unwrap();
    assert!((hit.depth - 0.2).abs() < 2e-5);
    close(hit.normal, Vec3::X, 1e-4);
    assert_witness(hit, 2e-5);
    let capsule = ShapeRef::new(
        Shape::Primitive(Primitive::Capsule {
            endpoints: [Vec3::new(0.85, 0.1, -0.2), Vec3::new(0.85, 0.1, 0.2)],
            radius: 0.2,
        }),
        Mat4::IDENTITY,
    )
    .unwrap();
    let hit = run(a, capsule, 0., 3.).unwrap().unwrap();
    assert!((hit.depth - 0.05).abs() < 3e-5);
    close(hit.normal, Vec3::X, 2e-4);
    assert_witness(hit, 3e-5);
}
#[test]
fn invalid_hulls_and_transforms_are_explicit_not_sphere_substitutes() {
    for vertices in [
        vec![],
        vec![Vec3::ZERO; 8],
        vec![
            Vec3::ZERO,
            Vec3::X,
            Vec3::Y,
            Vec3::ONE * 0.5 - Vec3::Z * 0.5,
        ],
    ] {
        let hull = Hull::new(vertices);
        assert_eq!(hull.issue(), Some(Issue::Degenerate));
        assert!(matches!(
            ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY),
            Err(Issue::Degenerate)
        ));
    }
    let hull = Hull::new(vec![Vec3::NAN; 8]);
    assert_eq!(hull.issue(), Some(Issue::InvalidInput));
    let hull = Hull::new(vec![Vec3::ZERO; MAX_HULL_VERTICES + 1]);
    assert_eq!(hull.issue(), Some(Issue::CapacityLimit));
    let hull = Hull::new(box_vertices(Vec3::ONE));
    assert!(matches!(
        ShapeRef::new(Shape::Hull(&hull), Mat4::from_scale(Vec3::new(1., 0., 1.))),
        Err(Issue::InvalidInput)
    ));
    assert!(matches!(
        ShapeRef::new(Shape::Hull(&hull), Mat4::from_translation(Vec3::NAN)),
        Err(Issue::InvalidInput)
    ));
}
#[test]
fn support_exhaustion_never_publishes_a_provisional_contact() {
    let hull = Hull::new(box_vertices(Vec3::ONE));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    let b = sphere(Vec3::new(1.1, 0.1, 0.1), 0.3);
    for limit in [0, 1, 7, 8, 9, 17, 26] {
        let mut work = ContactWork::new(PhysicsContactLimits {
            queries_per_update: 10,
            support_vertices_per_update: limit,
            ..Default::default()
        });
        assert!(matches!(
            query(a, b, 0., 3., &mut Scratch::default(), &mut work),
            Err(Issue::SupportLimit)
        ));
        assert_eq!(work.stats.support_vertices, limit as u64);
    }
    // The same scratch storage is overwritten/reused after an interrupted query.
    let mut scratch = Scratch::default();
    let pointer = scratch.vertices.as_ptr();
    let mut short = ContactWork::new(PhysicsContactLimits {
        queries_per_update: 10,
        support_vertices_per_update: 9,
        ..Default::default()
    });
    assert_eq!(
        query(a, b, 0., 3., &mut scratch, &mut short).err(),
        Some(Issue::SupportLimit)
    );
    let actual = query(a, b, 0., 3., &mut scratch, &mut work())
        .unwrap()
        .unwrap();
    let independent = run(a, b, 0., 3.).unwrap().unwrap();
    assert_eq!(actual.normal, independent.normal);
    assert_eq!(actual.points, independent.points);
    assert_eq!(actual.depth, independent.depth);
    assert_eq!(pointer, scratch.vertices.as_ptr());
}
#[test]
fn polytope_capacity_and_iteration_limits_are_never_best_effort_contacts() {
    let hull = Hull::new(box_vertices(Vec3::ONE));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    let b = sphere(Vec3::new(1.1, 0.13, 0.17), 0.3);
    let mut scratch = Scratch {
        max_vertices: 4,
        ..Default::default()
    };
    assert_eq!(
        query(a, b, 0., 3., &mut scratch, &mut work()).err(),
        Some(Issue::CapacityLimit)
    );
    let mut scratch = Scratch {
        max_faces: 4,
        ..Default::default()
    };
    assert_eq!(
        query(a, b, 0., 3., &mut scratch, &mut work()).err(),
        Some(Issue::CapacityLimit)
    );
    let mut scratch = Scratch {
        epa_steps: 0,
        ..Default::default()
    };
    assert_eq!(
        query(a, b, 0., 3., &mut scratch, &mut work()).err(),
        Some(Issue::IterationLimit)
    );
}

#[test]
fn scale_family_and_coincident_symmetry_use_finite_original_witnesses() {
    for scale in [1e-6_f32, 1., 1000.] {
        let hull = Hull::new(box_vertices(Vec3::ONE * scale));
        let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
        let b = sphere(Vec3::new(1.12, 0.13, -0.17) * scale, 0.3 * scale);
        let contact = run(a, b, 0., 3. * f64::from(scale)).unwrap().unwrap();
        assert!((contact.depth - 0.18 * scale).abs() < 3e-5 * scale);
        close(contact.normal, Vec3::X, 2e-4);
        assert_witness(contact, 3e-5 * scale);
    }
    let hull = Hull::new(box_vertices(Vec3::ONE));
    let shape = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    let contact = run(shape, shape, 0., 4.).unwrap().unwrap();
    // Identical inputs have a nonunique normal: reversal is not a meaningful
    // oracle here. Deterministic ties must still yield a genuine surface pair.
    assert!((contact.depth - 2.).abs() < 2e-5);
    assert_witness(contact, 2e-5);
    for point in contact.points {
        assert!((point.abs().max_element() - 1.).abs() < 2e-5);
        assert!(point.abs().max_element() <= 1. + 2e-5);
    }
}

#[test]
fn tiny_positive_gap_keeps_its_separation_certificate_in_a_large_hull_margin() {
    // Original mixed-feature-scale control: the gap is smaller than the hull's
    // convergence tolerance, but every Minkowski point has strictly positive X.
    let hull = Hull::new(
        (0..8)
            .map(|bits| {
                Vec3::new(
                    if bits & 1 == 0 { 0. } else { 1. },
                    if bits & 2 == 0 { 0. } else { 1. },
                    if bits & 4 == 0 { 0. } else { 1. },
                )
            })
            .collect(),
    );
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    let radius = 0.001_f32;
    let translation = Vec3::new(-0.001000005_f32, 0., 0.);
    let b = ShapeRef::new(
        Shape::Primitive(Primitive::Sphere {
            center: Vec3::ZERO,
            radius,
        }),
        Mat4::from_translation(translation),
    )
    .unwrap();
    let gap = -f64::from(translation.x) - f64::from(radius);
    assert!(gap > 0. && gap < 0.867 * RELATIVE_TOLERANCE);
    let contact = run(a, b, 1e-5, 0.867).unwrap().unwrap();
    assert_eq!(contact.depth, 0.);
    close(contact.normal, -Vec3::X, 1e-6);
    assert!(contact.points[0].as_dvec3().distance(DVec3::ZERO) < 1e-14);
    assert!((f64::from(contact.points[0].x) - f64::from(contact.points[1].x) - gap).abs() < 1e-14);
    assert!(
        (contact.points[1]
            .as_dvec3()
            .distance(translation.as_dvec3())
            - f64::from(radius))
        .abs()
            < 1e-14
    );
    assert!(run(a, b, 0., 0.867).unwrap().is_none());
    assert!(run(a, b, gap * 0.5, 0.867).unwrap().is_none());
}

#[test]
fn oversized_preparation_rejects_before_reading_or_copying_source_vertices() {
    for count in [MAX_HULL_VERTICES + 1, 10_000_000] {
        let mut reads = 0;
        let source = (0..count).map(|_| {
            reads += 1;
            Vec3::ZERO
        });
        let hull = Hull::prepare(source);
        assert_eq!(reads, 0, "the source must be preflighted before collect");
        assert_eq!(hull.issue(), Some(Issue::CapacityLimit));
        assert!(hull.vertices.is_empty());
    }
    // Already-owned input is also released, never retained in a dead query.
    let hull = Hull::new(vec![Vec3::ZERO; MAX_HULL_VERTICES + 1]);
    assert!(hull.vertices.is_empty());
}

#[test]
fn thin_face_exact_touch_and_adjacent_float_positions_match_analytic_box() {
    // Binary-exact dimensions distinguish a true boundary from the adjacent
    // positive gap and penetration, both below the broad relative tolerance.
    let half = Vec3::new(1., 0.03125, 0.4);
    let radius = 0.125;
    let touching = half.y + radius;
    let hull = Hull::new(box_vertices(half));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    let primitive = Primitive::Box {
        center: Vec3::ZERO,
        axes: Mat3::IDENTITY,
        half_extents: half,
    };
    for y in [touching.next_down(), touching, touching.next_up()] {
        let center = Vec3::new(0.17, y, 0.08);
        let b = sphere(center, radius);
        let oracle =
            contact::query_margin(primitive, Primitive::Sphere { center, radius }, 0.).unwrap();
        let result = run(a, b, 0., 2.).unwrap();
        assert_eq!(result.is_some(), oracle.is_some(), "y={y:?}");
        if let (Some(actual), Some(expected)) = (result, oracle) {
            close(actual.normal, Vec3::Y, 2e-4);
            assert!((actual.depth - expected.depth).abs() < 2e-8);
            assert_witness(actual, 2e-7);
            assert!((actual.points[0].y - half.y).abs() < 2e-7);
            assert!((actual.points[1].distance(center) - radius).abs() < 2e-7);
            if y == touching {
                assert_eq!(actual.depth, 0.);
                let reverse = run(b, a, 0., 2.).unwrap().unwrap();
                close(actual.normal, -reverse.normal, 2e-4);
                close(actual.points[0], reverse.points[1], 2e-7);
                close(actual.points[1], reverse.points[0], 2e-7);
            }
        }
    }
}

#[test]
fn projected_float_thin_face_overlap_does_not_require_generic_epa_seed() {
    let half = Vec3::new(1., 0.03, 0.4);
    let center = Vec3::new(0.17, 0.13, 0.08);
    let radius = 0.1;
    let depth = f64::from(half.y) + f64::from(radius) - f64::from(center.y);
    assert!(depth > 0. && depth < 2. * RELATIVE_TOLERANCE);
    let hull = Hull::new(box_vertices(half));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    for (left, right, normal) in [
        (a, sphere(center, radius), Vec3::Y),
        (sphere(center, radius), a, -Vec3::Y),
    ] {
        let actual = run(left, right, 0., 2.).unwrap().unwrap();
        close(actual.normal, normal, 2e-4);
        assert!((f64::from(actual.depth) - depth).abs() < 2e-8);
        assert_witness(actual, 2e-7);
    }
}

#[test]
fn accepted_triangle_projection_does_not_invent_a_tangential_search_direction() {
    // This original symmetric support triangle lies exactly in y=z and contains
    // the origin. Summing three weighted vertices leaves a spurious ~1e-16
    // tangential residual; its geometric plane projection is exactly zero.
    let vertices = [
        DVec3::new(
            0.19999998807907104,
            -1.1000000014901161,
            -1.1000000014901161,
        ),
        DVec3::new(-2.138254625851704, 1.1104003223898933, 1.1104003223898933),
        DVec3::new(0.1402295627712119, 1.027062488781937, 1.027062488781937),
    ]
    .map(|point| Vertex {
        point,
        a: point,
        b: DVec3::ZERO,
    });
    let candidate = affine(&vertices, 7).unwrap();
    assert_eq!(candidate.mask, 7);
    assert_eq!(candidate.point, DVec3::ZERO);
    assert!(candidate.weights.iter().take(3).all(|weight| *weight > 0.));
}

#[test]
fn shallow_triangle_projection_retains_its_geometric_normal() {
    for height in [1e-8, 1e-10, 1e-12] {
        let vertices = [
            DVec3::new(-1.17, -height, 0.32),
            DVec3::new(0.83, -height, 0.32),
            DVec3::new(0.13, -height, -0.48),
        ]
        .map(|point| Vertex {
            point,
            a: point,
            b: DVec3::ZERO,
        });
        let candidate = affine(&vertices, 7).unwrap();
        assert_eq!(candidate.mask, 7);
        assert_eq!(candidate.point.x, 0.);
        assert_eq!(candidate.point.z, 0.);
        assert!((candidate.point.y + height).abs() < height * 1e-14);
        assert_eq!(-candidate.point.normalize(), DVec3::Y);
    }
}

#[test]
fn thin_hull_translated_local_sphere_uses_actual_fixture_scale_and_margin() {
    let half = Vec3::new(1., 0.03, 0.4);
    let radius = 0.1;
    let scale = f64::from(half.length()) + f64::from(radius);
    let hull = Hull::new(box_vertices(half));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    for y in [0.05, 0.13_f32.next_down(), 0.13, 0.13_f32.next_up()] {
        let center = Vec3::new(0.17, y, 0.08);
        let b = ShapeRef::new(
            Shape::Primitive(Primitive::Sphere {
                center: Vec3::ZERO,
                radius,
            }),
            Mat4::from_translation(center),
        )
        .unwrap();
        for margin in [0., 1e-5] {
            let expected = contact::query_margin(
                Primitive::Box {
                    center: Vec3::ZERO,
                    axes: Mat3::IDENTITY,
                    half_extents: half,
                },
                Primitive::Sphere { center, radius },
                margin as f32,
            )
            .unwrap();
            let actual = run(a, b, margin, scale).unwrap_or_else(|issue| {
                panic!("y={y:?} margin={margin:?} scale={scale:?}: {issue:?}")
            });
            assert_eq!(
                actual.is_some(),
                expected.is_some(),
                "y={y:?} margin={margin:?}"
            );
            if let (Some(actual), Some(expected)) = (actual, expected) {
                close(actual.normal, Vec3::Y, 2e-4);
                assert!((actual.depth - expected.depth).abs() < 2e-7);
                assert_witness(actual, 2e-7);
            }
        }
    }
}

#[test]
fn sphere_center_representation_is_invariant_across_order_and_margin() {
    let half = Vec3::new(1., 0.03, 0.4);
    let radius = 0.1;
    let scale = f64::from(half.length()) + f64::from(radius);
    let hull = Hull::new(box_vertices(half));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    for y in [0.05, 0.13_f32.next_down(), 0.13, 0.13_f32.next_up(), 0.3] {
        let center = Vec3::new(0.17, y, 0.08);
        let world = sphere(center, radius);
        let local = ShapeRef::new(
            Shape::Primitive(Primitive::Sphere {
                center: Vec3::ZERO,
                radius,
            }),
            Mat4::from_translation(center),
        )
        .unwrap();
        for margin in [0., 1e-5] {
            for reversed in [false, true] {
                let (world_a, world_b, local_a, local_b) = if reversed {
                    (world, a, local, a)
                } else {
                    (a, world, a, local)
                };
                let expected = run(world_a, world_b, margin, scale).unwrap_or_else(|issue| {
                    panic!("world y={y:?} margin={margin:?} reversed={reversed}: {issue:?}")
                });
                let actual = run(local_a, local_b, margin, scale).unwrap_or_else(|issue| {
                    panic!("local y={y:?} margin={margin:?} reversed={reversed}: {issue:?}")
                });
                assert_eq!(
                    actual.is_some(),
                    expected.is_some(),
                    "y={y:?} margin={margin:?} reversed={reversed}"
                );
                if let (Some(actual), Some(expected)) = (actual, expected) {
                    close(actual.normal, expected.normal, 2e-4);
                    // A sphere support-plane deficit epsilon permits tangential
                    // error sqrt(2*r*epsilon); compare two converged witnesses
                    // under twice that bound, plus their f32 reconstruction.
                    let witness_error =
                        (2. * (2. * f64::from(radius) * scale * RELATIVE_TOLERANCE).sqrt()) as f32
                            + 8. * f32::EPSILON * (center.length() + radius);
                    close(actual.points[0], expected.points[0], witness_error);
                    close(actual.points[1], expected.points[1], witness_error);
                    assert!((actual.depth - expected.depth).abs() < 2e-7);
                    for result in [actual, expected] {
                        let (hull_point, sphere_point) = if reversed {
                            (result.points[1], result.points[0])
                        } else {
                            (result.points[0], result.points[1])
                        };
                        assert!((hull_point.y - half.y).abs() < 2e-7);
                        assert!(hull_point.x.abs() <= half.x + 2e-7);
                        assert!(hull_point.z.abs() <= half.z + 2e-7);
                        assert!((sphere_point.distance(center) - radius).abs() < 2e-7);
                        assert_witness(result, 2e-7);
                    }
                }
            }
        }
    }
}

#[test]
fn exact_projected_body_affine_retains_thin_face_contact_without_inconclusive() {
    let half = Vec3::new(1., 0.03, 0.4);
    let radius = 0.1;
    // Exact f32 columns captured after the original solver's first correction.
    // Retain their full affine map; do not re-orthogonalize this near rotation.
    let matrix = Mat4::from_cols_array(&[
        1.,
        -2.8013747e-7,
        -9.639432e-16,
        0.,
        2.8013747e-7,
        1.,
        2.3733284e-9,
        0.,
        2.9908492e-16,
        -2.3733284e-9,
        1.,
        0.,
        0.17,
        0.13,
        0.08,
        1.,
    ]);
    let hull = Hull::new(box_vertices(half));
    let a = ShapeRef::new(Shape::Hull(&hull), Mat4::IDENTITY).unwrap();
    let b = ShapeRef::new(
        Shape::Primitive(Primitive::Sphere {
            center: Vec3::ZERO,
            radius,
        }),
        matrix,
    )
    .unwrap();
    let scale = f64::from(1.0774508_f32) + f64::from(radius);
    // Independent +Y support of the affine sphere certifies tiny penetration:
    // world support radius is r*|L^T Y|, not an assumed orthogonal radius.
    let linear = DMat3::from_cols(
        matrix.x_axis.truncate().as_dvec3(),
        matrix.y_axis.truncate().as_dvec3(),
        matrix.z_axis.truncate().as_dvec3(),
    );
    let depth = f64::from(half.y) + f64::from(radius) * (linear.transpose() * DVec3::Y).length()
        - f64::from(matrix.w_axis.y);
    assert!(depth > 0. && depth < scale * RELATIVE_TOLERANCE);
    for margin in [0., 1e-5] {
        for (left, right, normal) in [(a, b, Vec3::Y), (b, a, -Vec3::Y)] {
            let actual = run(left, right, margin, scale)
                .unwrap_or_else(|issue| panic!("margin={margin:?} normal={normal:?}: {issue:?}"))
                .unwrap();
            close(actual.normal, normal, 2e-4);
            assert!((f64::from(actual.depth) - depth).abs() < 2e-8);
            assert_witness(actual, 2e-7);
        }
    }
}

#[test]
fn affine_tetra_keeps_resolvable_thin_origin_containment() {
    for height in [1e-6, 1e-8, 1e-10] {
        let vertices = [
            DVec3::new(-1., -1., -height),
            DVec3::new(1., -1., -height),
            DVec3::new(0., 1., -height),
            DVec3::new(0., 0., height),
        ]
        .map(|point| Vertex {
            point,
            a: point,
            b: DVec3::ZERO,
        });
        let candidate = affine(&vertices, 15).expect("resolvable direct tetra system");
        assert!(candidate.point.length() < 1e-14);
        assert_eq!(candidate.mask, 15);
        for (actual, expected) in candidate.weights.into_iter().zip([0.125, 0.125, 0.25, 0.5]) {
            assert!((actual - expected).abs() < 1e-12);
        }
        let outside = vertices.map(|mut vertex| {
            vertex.point.z += 2. * height;
            vertex
        });
        assert!(
            affine(&outside, 15).is_none(),
            "negative barycentrics must still reject"
        );
    }
}
