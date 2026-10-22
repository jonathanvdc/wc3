//! Contact topology, bounded preparation and runtime policy controls.
use super::*;
use crate::physics::admission::Admission;
use crate::{compile_model_with_options, CompileOptions};
use bevy::math::Vec2;
use wc3::model::systems::model_physics::{CollisionMask, Motor, Physics};
use wc3::model::values::model::PhysicsGameFlags;

fn joint(def: &PhysicsDefinition, a: usize, b: usize, constraint: Constraint) -> JointDefinition {
    let frame = Frame {
        axis_x: Vec3::X,
        axis_y: Vec3::Y,
        axis_z: Vec3::Z,
        origin: Vec3::ZERO,
    };
    JointDefinition {
        a,
        b,
        anchor_a: Vec3::ZERO,
        anchor_b: Vec3::ZERO,
        rotation_a: Quat::IDENTITY,
        rotation_b: Quat::IDENTITY,
        authored: Joint {
            body_a: def.bodies[a].info.body.node,
            body_b: def.bodies[b].info.body.node,
            frame_a: frame,
            frame_b: frame,
            constraint,
        },
    }
}
fn distance() -> Constraint {
    Constraint::Distance {
        distance: 0.,
        spring: Spring::default(),
    }
}
fn derive(def: &mut PhysicsDefinition, limit: usize) {
    def.admission = Admission::new(limit);
    def.admission
        .prepare_source(&def.bodies, &def.joints, 0..def.bodies.len());
}
#[test]
fn direct_adjacency_is_not_transitive_and_duplicates_do_not_change_admission() {
    let (mut def, _) = fixture(4);
    def.bodies[0].inverse_mass = 0.;
    def.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    def.bodies[3].fixtures.clear();
    def.joints = vec![
        joint(&def, 0, 1, distance()),
        joint(&def, 1, 2, distance()),
        joint(&def, 1, 0, distance()),
        joint(&def, 1, 2, distance()),
    ];
    derive(&mut def, 3);
    assert_eq!(def.admission.pairs, [(0, 2)]);
    assert!(
        def.pairs.is_empty(),
        "inference must not author semantic grants"
    );
    for body in &mut def.bodies {
        body.info.body.kind = PhysicsBodyKind::Kinematic;
    }
    assert_eq!(
        Admission::derive(&def.bodies, &def.joints, 0..4, 0),
        Ok(vec![])
    );
}
#[test]
fn preparation_gates_full_candidate_work_before_joint_exclusion_and_enumeration() {
    let (mut def, _) = fixture(4);
    def.joints = (0..4)
        .flat_map(|a| ((a + 1)..4).map(move |b| (a, b)))
        .map(|(a, b)| joint(&def, a, b, distance()))
        .collect();
    assert_eq!(
        Admission::derive(&def.bodies, &def.joints, 0..4, 5),
        Err(PhysicsContactError::PairBudgetExceeded {
            candidates: 6,
            limit: 5
        })
    );
    assert_eq!(
        Admission::derive(&def.bodies, &def.joints, 0..4, 6),
        Ok(vec![])
    );
    assert_eq!(
        Admission::derive(&def.bodies, &[], 0..4, 0),
        Err(PhysicsContactError::PairBudgetExceeded {
            candidates: 6,
            limit: 0
        })
    );
    for body in &mut def.bodies {
        body.info.body.kind = PhysicsBodyKind::Kinematic;
    }
    // There is no quadratic traversal of the non-dynamic population at this zero bound.
    assert_eq!(Admission::derive(&def.bodies, &[], 0..4, 0), Ok(vec![]));
}
#[test]
fn unresolved_controls_disable_only_the_opt_in_list_and_preserve_explicit_grants() {
    for variant in 0..5 {
        let (mut def, authored) = fixture(2);
        def.pairs.push((0, 1));
        let parent = def.bodies[1].info.body.node;
        let body = &mut def.bodies[0].info.body;
        match variant {
            0 => body.parent_body = Some(parent),
            1 => body.attachment_host = true,
            2 => body.ragdoll_root = true,
            value => {
                body.shapes.push(Shape {
                    transform: Mat4::IDENTITY,
                    geometry: ShapeGeometry::Sphere(0.5),
                    friction: 0.,
                    restitution: 0.,
                    density: 1.,
                    mass_properties: None,
                    game_flags: if value == 3 {
                        PhysicsGameFlags::from_bits_retain(1)
                    } else {
                        Default::default()
                    },
                    collision_mask: if value == 4 {
                        CollisionMask::from_bits_retain(1)
                    } else {
                        Default::default()
                    },
                });
            }
        }
        derive(&mut def, 10);
        assert_eq!(
            def.admission.error,
            Some(PhysicsContactError::UnsupportedControls)
        );
        assert_eq!(def.pairs, [(0, 1)]);
        let mut settings = PhysicsSimulation {
            contact_policy: PhysicsContactPolicy::UnconnectedDynamics,
            gravity: Vec3::ZERO,
            ..Default::default()
        };
        let mut state = PhysicsState::default();
        state.step(&def, &settings, 0., &authored, &HashMap::new());
        assert_eq!(
            state.contact_error(),
            Some(PhysicsContactError::UnsupportedControls)
        );
        settings.contact_policy = PhysicsContactPolicy::AuthoredPairs;
        state.step(&def, &settings, 0., &authored, &HashMap::new());
        assert_eq!(state.contact_error(), None);
    }
}
#[test]
fn unavailable_empty_definition_remains_observable_and_policy_switch_clears_status() {
    let mut def = PhysicsDefinition::default();
    def.admission.error = Some(PhysicsContactError::PairCountOverflow);
    assert!(def.observes_policy(PhysicsContactPolicy::UnconnectedDynamics));
    assert!(!def.observes_policy(PhysicsContactPolicy::AuthoredPairs));
    let mut state = PhysicsState::default();
    let mut settings = PhysicsSimulation {
        contact_policy: PhysicsContactPolicy::UnconnectedDynamics,
        ..Default::default()
    };
    state.step(&def, &settings, 0., &[], &HashMap::new());
    assert_eq!(
        state.contact_error(),
        Some(PhysicsContactError::PairCountOverflow)
    );
    settings.contact_policy = PhysicsContactPolicy::AuthoredPairs;
    state.step(&def, &settings, 0., &[], &HashMap::new());
    assert_eq!(state.contact_error(), None);
    state.clear();
    assert_eq!(state.contact_error(), None);
}
#[test]
fn available_policy_switch_resets_paused_zero_delta_pose_before_frame_rebase() {
    let (mut def, mut authored) = fixture(2);
    authored[1] = Mat4::from_translation(Vec3::X * 0.25);
    derive(&mut def, 10);
    let mut settings = PhysicsSimulation {
        frame: PhysicsFrame::World,
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.rebase(&def, &settings, 0., Some(Affine3A::IDENTITY));
    state.step(&def, &settings, 0., &authored, &HashMap::new());
    state.bodies[0].position = Vec3::splat(10.);
    state.bodies[0].velocity = Vec3::splat(12.);
    state.bodies[0].angular_velocity = Vec3::ONE;
    settings.contact_policy = PhysicsContactPolicy::UnconnectedDynamics;
    settings.paused = true;
    state.rebase(
        &def,
        &settings,
        0.,
        Some(Affine3A::from_translation(Vec3::X * 40.)),
    );
    assert_eq!(
        state.bodies[0].position,
        Vec3::splat(10.),
        "policy change cannot carry old-frame state"
    );
    state.step(&def, &settings, 0., &authored, &HashMap::new());
    assert_eq!(state.bodies[0].position, Vec3::ZERO);
    assert_eq!(state.bodies[0].velocity, Vec3::ZERO);
    assert_eq!(state.bodies[0].angular_velocity, Vec3::ZERO);
    settings.paused = false;
    state.step(&def, &settings, 0.1, &authored, &HashMap::new());
    assert!(state.bodies[0].position.distance(state.bodies[1].position) >= 0.99);
    settings.contact_policy = PhysicsContactPolicy::AuthoredPairs;
    state.step(&def, &settings, 0.1, &authored, &HashMap::new());
    assert_eq!(state.bodies[0].position, Vec3::ZERO);
    assert_eq!(state.bodies[1].position, Vec3::X * 0.25);
}
fn semantic_sources() -> (Model<'static>, Vec<RuntimeNode>) {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let mut all = Vec::new();
    for source in 0..2 {
        let mut bodies = Vec::new();
        for index in 0..3 {
            let node = builder
                .add_named_node(format!("source{source}-body{index}"))
                .unwrap();
            bodies.push(Body {
                node,
                kind: PhysicsBodyKind::Dynamic,
                gravity_scale: 1.,
                inertia_scale: 1.,
                linear_damping: 0.,
                angular_damping: 0.,
                follow_factor: 0.,
                parent_body: None,
                attachment_host: false,
                ragdoll_root: false,
                shapes: vec![Shape {
                    transform: Mat4::IDENTITY,
                    geometry: ShapeGeometry::Sphere(0.5),
                    friction: 0.,
                    restitution: 0.,
                    density: 1.,
                    mass_properties: None,
                    game_flags: Default::default(),
                    collision_mask: Default::default(),
                }],
            });
        }
        let frame = Frame {
            axis_x: Vec3::X,
            axis_y: Vec3::Y,
            axis_z: Vec3::Z,
            origin: Vec3::ZERO,
        };
        let joints = vec![Joint {
            body_a: bodies[0].node,
            body_b: bodies[1].node,
            frame_a: frame,
            frame_b: frame,
            constraint: Constraint::Prismatic {
                distances: Vec2::new(-1., 1.),
                reference_translation: 0.,
                motor_speed: 0.,
                friction_force: 0.,
                motor: Motor::default(),
            },
        }];
        all.extend(bodies.iter().map(|body| body.node));
        builder
            .add_system(System::M2(ModelSystem::Physics(Physics {
                kind: PhysicsKind::WornItem,
                resource: None,
                bodies,
                joints,
                collision_pairs: Vec::new(),
                wind_coefficients: None,
                follow_allow_list: Default::default(),
                vegetation: Vec::new(),
            })))
            .unwrap();
    }
    let model = builder.finish().unwrap();
    let (nodes, _) = model.prepare_pose().unwrap().into_parts();
    assert_eq!(nodes.len(), all.len());
    (model, nodes)
}
#[test]
fn compiler_preserves_source_isolation_omissions_and_current_prismatic_joint_adjacency() {
    let (model, nodes) = semantic_sources();
    let mut diagnostics = Vec::new();
    let compiled = PhysicsDefinition::compile(&model, &nodes, &mut diagnostics, 6);
    assert!(compiled.pairs.is_empty());
    assert_eq!(compiled.admission.pairs, [(0, 2), (1, 2), (3, 5), (4, 5)]);
    assert!(
        compiled.admission.error.is_none(),
        "each source retains isolated pairs under a shared preparation bound"
    );
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("slider limits")));
    let omitted = PhysicsDefinition::compile(&model, &nodes[..2], &mut Vec::new(), 3);
    assert_eq!(omitted.bodies.len(), 2);
    assert_eq!(omitted.joints.len(), 1);
    assert!(omitted.admission.pairs.is_empty());
    let aggregate = PhysicsDefinition::compile(&model, &nodes, &mut Vec::new(), 5);
    assert_eq!(
        aggregate.admission.error,
        Some(PhysicsContactError::PairBudgetExceeded {
            candidates: 6,
            limit: 5,
        })
    );
    assert!(
        aggregate.admission.pairs.is_empty(),
        "unavailable policy retains no partial grants"
    );
    let bounded = compile_model_with_options(
        &model,
        CompileOptions {
            physics_contact_pair_budget: 2,
            ..Default::default()
        },
        |_| None,
    )
    .unwrap();
    assert_eq!(
        bounded.physics.admission.error,
        Some(PhysicsContactError::PairBudgetExceeded {
            candidates: 3,
            limit: 2
        })
    );
    assert!(bounded.physics.pairs.is_empty());
    assert_eq!(
        PhysicsSimulation::default().contact_policy,
        PhysicsContactPolicy::AuthoredPairs
    );
}
#[test]
fn remap_retains_source_grants_without_cross_source_contacts_and_filters_collapsed_endpoints() {
    let (model, nodes) = semantic_sources();
    let mut def = PhysicsDefinition::compile(&model, &nodes, &mut Vec::new(), 16_384);
    let source = def.clone();
    def.append_remapped(&source, &(0..nodes.len()).collect::<Vec<_>>(), &nodes);
    assert_eq!(def.bodies.len(), 6, "shared target bodies are reused");
    assert_eq!(def.admission.pairs, [(0, 2), (1, 2), (3, 5), (4, 5)]);
    let mut mapped = Admission::default();
    mapped.append_remapped(&source.admission, &HashMap::from([(0, 5), (1, 3), (2, 4)]));
    mapped.retain_compiled(&source.bodies, &[]);
    assert_eq!(mapped.pairs, [(3, 4), (4, 5)]);
    let mut bounded = Admission::new(1);
    bounded.append_remapped(&source.admission, &HashMap::from([(0, 0), (1, 1), (2, 2)]));
    assert_eq!(
        bounded.error,
        Some(PhysicsContactError::PairBudgetExceeded {
            candidates: 2,
            limit: 1,
        })
    );
    assert!(bounded.pairs.is_empty());
    let mut collapsed = Admission::default();
    collapsed.append_remapped(&source.admission, &HashMap::from([(0, 1), (1, 1), (2, 1)]));
    assert!(collapsed.pairs.is_empty());
    let mut unavailable = Admission::default();
    unavailable.error = Some(PhysicsContactError::UnsupportedControls);
    mapped.append_remapped(&unavailable, &HashMap::new());
    assert_eq!(mapped.error, Some(PhysicsContactError::UnsupportedControls));
}

#[test]
fn retained_native_topology_ledgers_match_body_and_compound_fixture_candidate_counts() {
    // Original topology controls transcribed from the read-only native inventory.
    // They verify admission arithmetic, not import, contacts or native pose fidelity.
    type Topology<'a> = (&'a [usize], &'a [usize], &'a [(usize, usize)], usize, usize);
    let cases: &[Topology<'_>] = &[
        (
            &[0, 2, 1, 1, 1, 1, 1, 1],
            &[2, 3, 4, 5, 6, 7],
            &[(0, 1), (0, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7)],
            16,
            22,
        ),
        (
            &[0, 1, 1, 1, 1, 1, 3],
            &[1, 2, 3, 4, 5],
            &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (0, 6)],
            11,
            21,
        ),
        (
            &[1, 1, 1, 1, 1, 1, 1, 0],
            &[1, 2, 3, 4, 5, 6],
            &[(0, 1), (1, 2), (0, 3), (3, 4), (0, 5), (5, 6), (0, 7)],
            15,
            15,
        ),
    ];
    for &(fixtures, dynamic, edges, body_pairs, fixture_pairs) in cases {
        let (mut def, _) = fixture(fixtures.len());
        for (index, &count) in fixtures.iter().enumerate() {
            let shape = def.bodies[index].fixtures[0].clone();
            def.bodies[index].fixtures = vec![shape; count];
            def.bodies[index].info.body.kind = if dynamic.contains(&index) {
                PhysicsBodyKind::Dynamic
            } else {
                PhysicsBodyKind::Kinematic
            };
        }
        def.joints = edges
            .iter()
            .map(|&(a, b)| joint(&def, a, b, distance()))
            .collect();
        derive(&mut def, 100);
        assert_eq!(def.admission.pairs.len(), body_pairs);
        assert_eq!(
            def.admission
                .pairs
                .iter()
                .map(|&(a, b)| def.bodies[a].fixtures.len() * def.bodies[b].fixtures.len())
                .sum::<usize>(),
            fixture_pairs
        );
        // Independently decoded actual BDY4 controls: body zero is 0xC000
        // (host + ragdoll, parent index zero) in ALL three native files.
        // The first two anchors have no fixtures; the third has one capsule.
        def.bodies[0].info.body.attachment_host = true;
        def.bodies[0].info.body.ragdoll_root = true;
        derive(&mut def, 100);
        if fixtures[0] == 0 {
            assert!(def.admission.error.is_none());
            assert_eq!(def.admission.pairs.len(), body_pairs);
        } else {
            assert_eq!(
                def.admission.error,
                Some(PhysicsContactError::UnsupportedControls)
            );
            assert!(
                def.admission.pairs.is_empty(),
                "1338711 controls remain unresolved"
            );
        }
    }
}

#[test]
fn default_policy_preserves_overlap_and_authored_pair_order_without_implicit_contacts() {
    let (mut def, mut authored) = fixture(2);
    authored[1] = Mat4::from_translation(Vec3::X * 0.25);
    derive(&mut def, 1);
    assert_eq!(def.admission.pairs, [(0, 1)]);
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&def, &settings, 0., &authored, &HashMap::new());
    state.step(&def, &settings, 0.1, &authored, &HashMap::new());
    assert_eq!(state.bodies[0].position, Vec3::ZERO);
    assert_eq!(state.bodies[1].position, Vec3::X * 0.25);
    assert!(state.contact_error().is_none());
    def.pairs = vec![(1, 0), (0, 1), (1, 0)];
    def.joints.push(joint(&def, 0, 1, distance()));
    derive(&mut def, 1);
    assert!(
        def.admission.pairs.is_empty(),
        "a compiled joint removes only inferred grants"
    );
    assert_eq!(
        def.pairs,
        [(1, 0), (0, 1), (1, 0)],
        "authored order and duplicates remain unchanged"
    );
}

#[test]
fn zero_radius_dynamic_points_and_capsules_are_admitted_against_a_kinematic_sphere() {
    for geometry in [
        contact::Primitive::Sphere {
            center: Vec3::ZERO,
            radius: 0.,
        },
        contact::Primitive::Capsule {
            endpoints: [Vec3::ZERO; 2],
            radius: 0.,
        },
    ] {
        let (mut def, mut authored) = fixture(2);
        def.bodies[0].fixtures[0].geometry = geometry;
        def.bodies[0].fixtures[0].radius = 0.;
        def.bodies[0].radius = 0.;
        def.bodies[1].info.body.kind = PhysicsBodyKind::Kinematic;
        def.bodies[1].inverse_mass = 0.;
        authored[1] = Mat4::from_translation(Vec3::X * 0.25);
        derive(&mut def, 1);
        assert_eq!(
            def.admission.pairs,
            [(0, 1)],
            "valid zero-radius geometry must not silently lose admission"
        );
        assert!(def.admission.error.is_none());
        let settings = PhysicsSimulation {
            contact_policy: PhysicsContactPolicy::UnconnectedDynamics,
            gravity: Vec3::ZERO,
            ..Default::default()
        };
        let mut state = PhysicsState::default();
        state.step(&def, &settings, 0., &authored, &HashMap::new());
        state.step(&def, &settings, 0.1, &authored, &HashMap::new());
        assert_eq!(state.bodies[1].position, Vec3::X * 0.25);
        assert!(state.bodies[0].position.is_finite());
        assert!(state.bodies[0].position.distance(state.bodies[1].position) >= 0.4999);
    }
}

#[test]
fn unsupported_motion_on_fixtureless_anchors_does_not_block_contact_participants() {
    for control in 0..3 {
        let (mut def, _) = fixture(3);
        def.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
        def.bodies[0].inverse_mass = 0.;
        let saved = def.bodies[0].fixtures[0].clone();
        def.bodies[0].fixtures.clear();
        let parent = def.bodies[1].info.body.node;
        match control {
            0 => def.bodies[0].info.body.parent_body = Some(parent),
            1 => def.bodies[0].info.body.attachment_host = true,
            _ => def.bodies[0].info.body.ragdoll_root = true,
        }
        def.joints = vec![joint(&def, 0, 1, distance())];
        def.pairs = vec![(0, 2)];
        derive(&mut def, 10);
        assert_eq!(def.admission.error, None);
        assert_eq!(def.admission.pairs, [(1, 2)]);
        assert_eq!(
            def.pairs,
            [(0, 2)],
            "authored grants remain a separate authority"
        );
        assert_eq!(
            def.joints.len(),
            1,
            "fixtureless anchor remains in the solver topology"
        );
        // Giving the same flagged anchor a fixture makes it a potential endpoint.
        def.bodies[0].fixtures.push(saved);
        derive(&mut def, 10);
        assert_eq!(
            def.admission.error,
            Some(PhysicsContactError::UnsupportedControls)
        );
        assert!(def.admission.pairs.is_empty());
        assert_eq!(def.pairs, [(0, 2)]);
    }
}

#[test]
fn no_possible_dynamic_pair_needs_no_contact_control_interpretation() {
    for count in [0, 1, 4] {
        let (mut def, _) = fixture(count);
        for body in &mut def.bodies {
            body.info.body.kind = PhysicsBodyKind::Kinematic;
            body.info.body.attachment_host = true;
            body.info.body.ragdoll_root = true;
        }
        derive(&mut def, 0);
        assert!(def.admission.error.is_none());
        assert!(def.admission.pairs.is_empty());
        if count == 1 {
            def.bodies[0].info.body.kind = PhysicsBodyKind::Dynamic;
            derive(&mut def, 0);
            assert!(
                def.admission.error.is_none(),
                "one lone dynamic has no possible partner"
            );
        }
    }
    // Zero radius is supported geometry, not an excuse to ignore endpoint controls.
    let (mut def, _) = fixture(2);
    def.bodies[0].fixtures[0].geometry = contact::Primitive::Sphere {
        center: Vec3::ZERO,
        radius: 0.,
    };
    def.bodies[0].fixtures[0].radius = 0.;
    def.bodies[0].radius = 0.;
    def.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    def.bodies[0].info.body.ragdoll_root = true;
    derive(&mut def, 10);
    assert_eq!(
        def.admission.error,
        Some(PhysicsContactError::UnsupportedControls)
    );
}
