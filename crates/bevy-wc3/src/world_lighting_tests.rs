use super::*;
use wc3::model::{Construct as SlotConstruct, Values as SlotValues};

fn policy() -> ColorAdjustment {
    ColorAdjustment {
        base: [20, 40, 60],
        alpha_only: false,
        exterior: false,
        force_exterior: false,
        transition_end: 1,
        portal_alpha: Arc::from([]),
        diagnostic: None,
    }
}
fn rgba(value: [u8; 4]) -> [f32; 4] {
    value.map(|v| v as f32 / 255.)
}
fn bytes(value: [f32; 4]) -> [u8; 4] {
    value.map(|v| byte(v * 255.))
}

#[test]
fn transition_fixup_subtracts_ambient_without_unsigned_wrap() {
    let policy = policy();
    assert_eq!(
        bytes(policy.color(0, rgba([100, 100, 100, 0]))),
        [40, 30, 20, 0]
    );
    assert_eq!(bytes(policy.color(0, rgba([10, 20, 30, 0]))), [0, 0, 0, 0]);
    assert_eq!(
        bytes(policy.color(0, rgba([100, 100, 100, 255]))),
        [0, 0, 0, 255]
    );
}
#[test]
fn nontransition_rgb_uses_authored_alpha_before_lighting_class_replaces_it() {
    let mut policy = policy();
    assert_eq!(
        bytes(policy.color(1, rgba([100, 100, 100, 64]))),
        [90, 80, 70, 0]
    );
    policy.exterior = true;
    assert_eq!(
        bytes(policy.color(1, rgba([100, 100, 100, 64]))),
        [90, 80, 70, 255]
    );
    policy.force_exterior = true;
    assert_eq!(
        bytes(policy.color(0, rgba([100, 100, 100, 0]))),
        [40, 30, 20, 255]
    );
}
#[test]
fn alpha_only_preserves_rgb_and_transition_alpha_but_sets_other_alpha() {
    let mut policy = policy();
    policy.alpha_only = true;
    assert_eq!(
        bytes(policy.color(0, rgba([100, 120, 140, 80]))),
        [100, 120, 140, 80]
    );
    assert_eq!(
        bytes(policy.color(1, rgba([100, 120, 140, 80]))),
        [100, 120, 140, 0]
    );
}
fn portal(exterior: bool) -> PortalBoundary {
    PortalBoundary {
        vertices: vec![
            Vec3::new(-1., -1., 0.),
            Vec3::new(1., -1., 0.),
            Vec3::new(1., 1., 0.),
            Vec3::new(-1., 1., 0.),
        ],
        normal: Vec3::Z,
        distance: 0.,
        side: 1,
        exterior,
    }
}
#[test]
fn exterior_portals_attenuate_by_signed_plane_and_nearest_polygon_edge() {
    let portals = [portal(true)];
    assert_eq!(portal_blend(Vec3::ZERO, &portals), 1.);
    assert!((portal_blend(Vec3::new(0., 0., 2.), &portals) - 0.7).abs() < 1e-6);
    assert_eq!(portal_blend(Vec3::new(0., 0., -2.), &portals), 1.);
    assert!((portal_blend(Vec3::new(3., 0., 0.), &portals) - 0.7).abs() < 1e-6);
    assert_eq!(portal_blend(Vec3::new(0., 0., 10.), &portals), 0.);
    assert_eq!(portal_blend(Vec3::ZERO, &[portal(true), portal(false)]), 0.);
    assert_eq!(portal_blend(Vec3::ZERO, &[]), 0.);
}
#[test]
fn portal_fixup_precedes_rgb_normalization_and_keeps_source_index() {
    let mut policy = policy();
    policy.base = [0; 3];
    policy.portal_alpha = Arc::from([Some(128)]);
    let raw = rgba([200, 160, 120, 17]);
    assert_eq!(bytes(policy.color(0, raw)), [25, 20, 15, 128]);
    assert_eq!(raw, rgba([200, 160, 120, 17]));
    assert_ne!(policy.color(1, raw), policy.color(0, raw));
}

fn authored_context(
    side: i16,
    adjusted: bool,
    translation: [f32; 3],
) -> (Model<'static>, MeshInstance<'static>) {
    let mut slot_values = SlotValues::default();
    use wc3::model::controls::world::{
        Group, GroupId, ModelMetadata as WorldMetadata, PortalId, RegionControls, WorldSystem,
    };
    use wc3::model::controls::world_draw::DrawControls as WorldDraw;
    use wc3::model::scene::world::{GroupOptions, Portal, PortalConnection, WorldOptions};
    use wc3::model::{Draw, DrawSubset, GeometryData, Mesh, Node, PrimitiveSelection};
    let mut builder = Model::builder("authored lighting");
    builder.set_metadata(Some(ModelMetadata::Wmo(WorldMetadata {
        ambient: [20, 40, 60, 255],
        options: WorldOptions {
            vertex_color_alpha_only: true,
            ..SlotConstruct::construct(&mut slot_values)
        },
        ..SlotConstruct::construct(&mut slot_values)
    })));
    let node = Node::named("group", &mut slot_values);
    slot_values
        .set(node.transform.translation, translation)
        .unwrap();
    let node = builder.add_node(node).unwrap();
    let mut mesh = Mesh::new(
        GeometryData::triangles([[0., 0., 0.], [0., 0., 2.], [0., 1., 0.]], [[0, 1, 2]]).unwrap(),
    );
    let subset = mesh.add_subset(DrawSubset::new(PrimitiveSelection {
        connectivity: mesh.subsets.primary(),
        range: PrimitiveRange::Primitives { first: 0, count: 1 },
    }));
    let mesh = builder.add_mesh(mesh).unwrap();
    let system = builder
        .add_system(System::Wmo(WorldSystem {
            groups: vec![Group {
                options: GroupOptions {
                    exterior: true,
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            }],
            portals: vec![Portal {
                vertices: [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]]
                    .into_iter()
                    .collect(),
                plane: [0., 0., 1., 0.],
                ..SlotConstruct::construct(&mut slot_values)
            }],
            ..SlotConstruct::construct(&mut slot_values)
        }))
        .unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.draws.push(Draw::new(
        subset,
        Some(DrawControls::Wmo(WorldDraw {
            lighting: Lighting::Transition,
            ..SlotConstruct::construct(&mut slot_values)
        })),
    ));
    instance.controls = Some(InstanceControls::Wmo(Box::new(RegionControls {
        portals: vec![PortalConnection {
            portal: PortalId { system, index: 0 },
            neighbor: GroupId { system, index: 0 },
            side,
            adjusted_plane: adjusted,
            skip_viewer: false,
            unclassified: SlotConstruct::construct(&mut slot_values),
        }],
        ..SlotConstruct::construct(&mut slot_values)
    })));
    (
        {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap(),
        instance,
    )
}
#[test]
fn constructor_uses_authored_transition_draws_and_portal_connectivity() {
    let (model, instance) = authored_context(1, false, [0.; 3]);
    let policy = ColorAdjustment::new(&model, &instance).unwrap();
    assert_eq!(policy.transition_end, 3);
    assert!(policy.diagnostic().is_none());
    let raw = rgba([200, 160, 120, 17]);
    assert_eq!(bytes(policy.color(0, raw)), [0, 0, 0, 255]);
    let second = bytes(policy.color(1, raw));
    assert_eq!(second[3], 178); // 0.7 * 255 rounds to nearest even.
    assert_eq!(second[0], 60);
}
#[test]
fn ambiguous_portals_or_transformed_groups_keep_authored_transition_values() {
    for (side, adjusted, translation) in [
        (0, false, [0.; 3]),
        (1, true, [0.; 3]),
        (1, false, [1., 0., 0.]),
    ] {
        let (model, instance) = authored_context(side, adjusted, translation);
        let policy = ColorAdjustment::new(&model, &instance).unwrap();
        assert!(policy.diagnostic().is_some());
        let raw = rgba([200, 160, 120, 17]);
        assert_eq!(bytes(policy.color(0, raw)), [200, 160, 120, 17]);
    }
}
