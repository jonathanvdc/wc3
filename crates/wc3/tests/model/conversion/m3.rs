use std::borrow::Cow;
use wc3::model::common::IntoZeroVec;
use wc3::model::io;
use wc3::model::Model;
use wc3::model::{
    self as sem,
    conversion::{ApproximationPolicy, ExportOptions, ImportOptions},
    formats::m3::{self, records as nr},
    profiles::{m3 as surface, M3},
};
pub(super) fn asset() -> Model<'static, M3> {
    let mut builder = sem::Model::<M3>::builder("Triangle");
    let node = builder
        .add_node(sem::Node::named("Root").with_controls(surface::BoneControls::default()))
        .unwrap();
    let influences = (vec![
        [
            sem::Influence {
                joint: 0,
                weight: 1.
            },
            sem::Influence {
                joint: 0,
                weight: 0.
            },
            sem::Influence {
                joint: 0,
                weight: 0.
            },
            sem::Influence {
                joint: 0,
                weight: 0.
            }
        ];
        3
    ])
    .into_zero_vec();
    let geometry = sem::M3Geometry::new(
        (vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]).into_zero_vec(),
        (vec![[0., 0., 1.]; 3]).into_zero_vec(),
        (vec![[0, 1, 2]]).into_zero_vec(),
        influences,
    )
    .unwrap()
    .with_texture_coordinates(vec![(vec![[0., 0.], [1., 0.], [0., 1.]]).into_zero_vec()])
    .unwrap();
    let mesh = builder.add_mesh(sem::Mesh::new(geometry)).unwrap();
    let material = builder
        .add_material(sem::Material {
            name: Cow::Borrowed("Solid"),
            surface: surface::Material::Standard((surface::Standard::default()).into()),
        })
        .unwrap();
    let identity = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ];
    let skin = builder
        .add_skin(sem::Skin::new(
            vec![node],
            sem::ExplicitBinding::new((vec![identity]).into_zero_vec()).unwrap(),
        ))
        .unwrap();
    let frame = builder.add_node(sem::Node::named("")).unwrap();
    let mut instance = sem::MeshInstance::new(frame, mesh);
    instance.skin = Some(skin);
    instance.draws.push(sem::Draw::new(
        builder.add_primary_subset(mesh, 0, 1).unwrap(),
        None,
    ));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    let controller = builder
        .add_controller(sem::Controller {
            name: Cow::Borrowed("Movement"),
            ..Default::default()
        })
        .unwrap();
    builder
        .add_controller_channel(
            controller,
            sem::Channel::Vector3(sem::Track {
                target: node.translation(),
                clock: sem::Clock::Timeline,
                curve: sem::Curve::linear(
                    (vec![
                        sem::Time::seconds(0.).unwrap(),
                        sem::Time::seconds(1.).unwrap(),
                    ])
                    .into_zero_vec(),
                    (vec![[0., 0., 0.], [1., 2., 3.]]).into_zero_vec(),
                )
                .unwrap(),
            }),
        )
        .unwrap();
    let mut clip: sem::Clip<'_, M3> =
        sem::Clip::named("Move").with_duration(sem::Time::seconds(1.).unwrap());
    clip.controls.selection = Some(surface::ControllerSelection {
        name: "sequence".into(),
        controllers: vec![controller],
    });
    builder.add_clip(clip).unwrap();
    builder.finish().unwrap()
}
pub(super) fn approximate() -> ExportOptions {
    ExportOptions {
        approximation: ApproximationPolicy::Allow { tolerance: 0.004 },
        ..Default::default()
    }
}
#[test]
fn geometry_skin_and_animation_survive_native_round_trip() {
    let asset = asset();
    let native = io::m3::plan::<m3::StarCraft2>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap();
    let bytes = native.value.write().unwrap();
    let decoded = m3::Bundle::from_model(m3::Document::<m3::Md34>::read(&bytes.model).unwrap());
    let imported = io::m3::import(&decoded, &ImportOptions::default()).unwrap();
    let model = &imported.value;
    let (_, mesh) = model.meshes().iter().next().unwrap();
    assert_eq!(mesh.positions().get(1), Some([1., 0., 0.]));
    let sem::Topology::Triangles(triangles) = mesh.topology() else {
        panic!("triangle topology")
    };
    assert_eq!(triangles.get(0), Some([0, 1, 2]));
    assert_eq!(mesh.influences().get(2).unwrap().weights().next(), Some(1.));
    assert_eq!(
        model
            .nodes()
            .iter()
            .find(|(_, node)| node.name == "Root")
            .unwrap()
            .1
            .name,
        "Root"
    );
    let (_, controller) = model.controllers().iter().next().unwrap();
    let sem::Channel::Vector3(track) = &controller.channels[0] else {
        panic!("vector track")
    };
    assert_eq!(track.curve.values().get(1), Some([1., 2., 3.]));
    assert_eq!(model.clips().iter().count(), 1);
}
#[test]
fn quantization_requires_an_explicit_policy() {
    let error = io::m3::plan::<m3::StarCraft2>(&asset(), &ExportOptions::default()).unwrap_err();
    assert!(error.path.contains("normal"));
}
#[test]
fn profiles_select_root_and_envelope_independently() {
    let asset = asset();
    let beta = io::m3::plan::<m3::Beta>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap();
    assert_eq!(beta.value.model.root_record().unwrap().revision(), 23);
    assert_eq!(&beta.value.write().unwrap().model[..4], b"33DM");
    let heroes = io::m3::plan::<m3::HeroesOfTheStorm>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap();
    assert_eq!(heroes.value.model.root_record().unwrap().revision(), 30);
}
#[test]
fn standard_controls_remain_active_after_geometry_edit() {
    let mut asset = asset();
    let (id, material) = asset.materials().iter().next().unwrap();
    let mut material = material.clone();
    let surface::Material::Standard(standard) = &mut material.surface else {
        unreachable!()
    };
    standard.layer_blend = surface::values::LayerBlendOp::Lerp;
    standard.state.rasterization.flags.set_normal_blend(true);
    standard.state.rasterization.flags.set_normal_blend2(true);
    standard.state.rasterization.flags.set_unfogged(true);
    standard.specular.parameters.exponent = 17.;
    standard.state.ordering = -3;
    asset
        .edit(|builder| builder.set_material(id, material))
        .unwrap();
    let native = io::m3::plan::<m3::StarCraft2>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = native.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let material = native
        .model
        .container()
        .resolve::<nr::StandardMaterial<nr::V20>>(root.standard_materials.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(material.layer_blend_mode.raw(), 3);
    assert!(material.flags.normal_blend());
    assert!(material.flags.normal_blend2());
    assert!(material.flags.unfogged());
    assert_eq!(material.specular_exponent, 17.);
    assert_eq!(material.priority, -3);
}
#[test]
fn dormant_standard_and_hair_surfaces_are_authored_materials() {
    let mut asset = asset();
    let hair = surface::Hair {
        shift_primary: 0.25,
        ..Default::default()
    };
    asset
        .edit(|builder| {
            builder.add_material(sem::Material {
                name: Cow::Borrowed("Dormant hair"),
                surface: surface::Material::Hair((hair).into()),
            })
        })
        .unwrap();
    let native = io::m3::plan::<m3::StarCraft2>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let imported = io::m3::import(&native, &ImportOptions::default()).unwrap();
    assert!(imported
        .value
        .materials()
        .iter()
        .any(|(_, material)| matches!(material.surface, surface::Material::Hair(_))));
}

#[path = "m3_geometry_controls.rs"]
mod geometry_controls;
#[test]
fn imported_graph_widens_and_narrows_without_changing_identity_or_values() {
    let source = asset();
    let native = io::m3::plan::<m3::StarCraft2>(&source, &approximate())
        .unwrap()
        .execute()
        .unwrap();
    let bytes = native.value.write().unwrap();
    let decoded = m3::Bundle::from_model(m3::Document::<m3::Md34>::read(&bytes.model).unwrap());
    let imported = io::m3::import(&decoded, &ImportOptions::default())
        .unwrap()
        .value;
    let model_id = imported.id();
    let mesh_id = imported.meshes().iter().next().unwrap().0;
    let node_id = imported.nodes().iter().next().unwrap().0;
    assert!(imported.mesh(mesh_id).unwrap().positions().is_owned());
    let narrowed = imported
        .widen::<sem::profiles::Universal>()
        .try_narrow::<M3>()
        .unwrap();
    assert_eq!(narrowed.id(), model_id);
    assert!(narrowed.node(node_id).is_some());
    let mesh = narrowed.mesh(mesh_id).unwrap();
    assert!(mesh.positions().is_owned());
    assert_eq!(mesh.positions().get(1), Some([1., 0., 0.]));
    assert_eq!(narrowed.controllers().iter().count(), 1);
}
#[test]
fn native_profile_rejects_coordinate_metadata_that_cannot_round_trip() {
    let mut source = asset();
    let original = source.basis();
    assert!(source
        .edit(|builder| {
            builder.set_basis(sem::Basis::new(
                sem::Axis::PositiveY,
                sem::Axis::PositiveZ,
                sem::Handedness::Right,
            )?);
            Ok(())
        })
        .is_err());
    assert_eq!(source.basis(), original);
    assert!(source
        .edit(|builder| {
            builder.set_units(sem::LengthUnit::MetersPerUnit(0.5));
            Ok(())
        })
        .is_err());
    assert_eq!(source.units(), sem::LengthUnit::Unspecified);
}

#[test]
fn unknown_sections_and_vertex_qualifiers_require_explicit_drop() {
    use wc3::model::conversion::{IssueKind, UnknownPolicy};
    let original = asset();
    let source = io::m3::plan::<m3::StarCraft2>(&original, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = source.model.into_container();
    root.vertex_flags = nr::VertexFlags(root.vertex_flags.bits() | 0x1c);
    let root_id = document
        .section_id(document.root_reference().index)
        .unwrap();
    document.replace(root_id, &[root]).unwrap();
    document.append_raw(*b"XTRA", 1, 1, vec![7, 8, 9]).unwrap();
    let source = m3::Bundle::from_model(m3::Document::from_container(document).unwrap());
    assert!(io::m3::import(&source, &ImportOptions::default()).is_err());
    let imported = io::m3::import(
        &source,
        &ImportOptions {
            unknown: UnknownPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "m3.vertex_qualifier" && issue.kind == IssueKind::Dropped));
    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path.contains("opaque") && issue.kind == IssueKind::Dropped));
    assert!(source
        .model
        .container()
        .sections()
        .any(|(_, section)| section.tag() == *b"XTRA"));
    let output = io::m3::plan::<m3::StarCraft2>(&imported.value, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    assert_eq!(root.vertex_flags.bits() & 0x1c, 0);
    assert!(output
        .model
        .container()
        .sections()
        .all(|(_, section)| section.tag() != *b"XTRA"));
}

#[test]
fn contribution_groups_round_trip_sparse_slots_and_arbitrary_normal_weights() {
    let mut asset = asset();
    let (id, material) = asset.materials().iter().next().unwrap();
    let mut material = material.clone();
    let surface::Material::Standard(standard) = &mut material.surface else {
        unreachable!()
    };
    standard.specular.parameters.exponent = 23.;
    standard.specular.parameters.hdr_multiplier = 2.5;
    standard.emissive.parameters = 3.5;
    standard.emissive.input.secondary.input = Some(surface::TextureLayer {
        id: 71,
        ..Default::default()
    });
    standard.normals.primary.mask = Some(surface::TextureLayer {
        id: 81,
        ..Default::default()
    });
    standard.normals.weights = (0..5)
        .map(|index| surface::Property {
            initial: index as f32 * 0.2,
            reset: index as f32 * 0.1,
            ..Default::default()
        })
        .collect();
    asset
        .edit(|builder| builder.set_material(id, material))
        .unwrap();
    let native = io::m3::plan::<m3::StarCraft2>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let imported = io::m3::import(&native, &ImportOptions::default())
        .unwrap()
        .value;
    let (_, material) = imported
        .materials()
        .iter()
        .find(|(_, material)| material.name == "Solid")
        .unwrap();
    let surface::Material::Standard(standard) = &material.surface else {
        unreachable!()
    };
    assert_eq!(standard.specular.parameters.exponent, 23.);
    assert_eq!(standard.specular.parameters.hdr_multiplier, 2.5);
    assert_eq!(standard.emissive.parameters, 3.5);
    assert!(standard.emissive.input.primary.input.is_none());
    assert_eq!(
        standard.emissive.input.secondary.input.as_ref().unwrap().id,
        71
    );
    assert!(standard.normals.primary.input.is_none());
    assert_eq!(standard.normals.primary.mask.as_ref().unwrap().id, 81);
    assert_eq!(standard.normals.weights.len(), 5);
    for (index, weight) in standard.normals.weights.iter().enumerate() {
        assert_eq!(weight.initial, index as f32 * 0.2);
        assert_eq!(weight.reset, index as f32 * 0.1);
    }
}
