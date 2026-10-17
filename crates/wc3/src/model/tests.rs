use crate::model::common::Borrowed as _;
use crate::model::PathResource;
use crate::model::Values as SlotValues;
use crate::model::*;
use zerovec::ZeroVec;
#[test]
fn foreign_graph_handles_are_rejected() {
    let mut first = ModelBuilder::new("first");
    let parent = first.add_named_node("parent").unwrap();
    first.finish().unwrap();
    let mut second = ModelBuilder::new("second");
    let child = second.new_node("child").with_parent(parent);
    second.add_node(child).unwrap();
    assert!(second.finish().is_err());
}

#[test]
fn forward_parents_are_resolved_and_cycles_rejected() {
    let slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new("model");
    let parent = builder.reserve_node();
    let child = builder.new_node("child").with_parent(parent);
    builder.add_node(child).unwrap();
    let definition = builder.new_node("parent");
    builder.set_node(parent, definition).unwrap();
    let mut model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let child = model
        .nodes()
        .iter()
        .find(|(_, node)| node.name == "child")
        .unwrap()
        .0;
    let result = model.edit(|draft| draft.set_node_parent(parent, Some(child)));
    assert!(result.is_err());
    assert_eq!(model.node(parent).unwrap().parent, None);
}

#[test]
fn borrowing_survives_graph_construction_and_clone() {
    let slot_values = SlotValues::default();
    let positions = ZeroVec::alloc_from_slice(&[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
    let faces = [[0, 1, 2]];
    let mut builder = ModelBuilder::new("triangle");
    let id = builder
        .add_mesh(Mesh::triangles(positions.as_borrowed(), faces.as_slice()))
        .unwrap();
    let graph = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let clone = graph.clone();
    assert_eq!(
        clone.mesh(id).unwrap().positions().as_bytes().as_ptr(),
        positions.as_bytes().as_ptr()
    );
}

#[test]
fn owned_array_detachment_shares_storage() {
    let values = ZeroVec::alloc_from_slice(&[1u32, 2, 3]);
    let pointer = values.as_bytes().as_ptr();
    assert_eq!(values.into_owned().as_bytes().as_ptr(), pointer);
}

#[test]
fn invalid_geometry_never_enters_a_published_model() {
    let slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new("bad geometry");
    builder
        .add_mesh(Mesh::triangles([[0., 0., 0.]], [[0, 1, 2]]))
        .unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
}

#[test]
fn boundary_keys_may_precede_playback() {
    let keys = [Time::seconds(-0.5).unwrap(), Time::seconds(1.).unwrap()];
    assert!(Curve::from_samples(keys, [0., 1.], Interpolation::Linear).is_ok());
    assert!(Time::seconds(f64::NAN).is_err());
}

#[test]
fn cloned_branches_allocate_distinct_new_identities() {
    let original = Model::standalone("original");
    let mut a = original.clone();
    let mut b = original;
    let a_id = a.edit(|draft| draft.add_named_node("a")).unwrap();
    let b_id = b.edit(|draft| draft.add_named_node("b")).unwrap();
    assert_ne!(a_id, b_id);
    assert!(a.node(b_id).is_none());
}

#[test]
fn graph_can_cross_thread_boundaries() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<Model<'static>>();
}

#[test]
fn scalar_and_vector_slots_cannot_alias() {
    let mut values = Values::default();
    let scalar = values.add(1f32);
    let vector = values.add([1f32; 3]);
    assert_ne!(scalar.slot(), vector.slot());
}

#[test]
fn complete_material_looks_resolve_independently() {
    let slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new("looks");
    let node = builder.add_named_node("root").unwrap();
    let mesh = builder
        .add_mesh(Mesh::triangles(
            [[0f32; 3], [1., 0., 0.], [0., 1., 0.]],
            [[0, 1, 2]],
        ))
        .unwrap();
    let base = builder
        .add_material(Material {
            name: "".into(),
            surface: Default::default(),
        })
        .unwrap();
    let alternative = builder
        .add_material(
            Material {
                name: "".into(),
                surface: Default::default(),
            }
            .named("alternate"),
        )
        .unwrap();
    let subset = builder.add_primary_subset(mesh, 0, 1).unwrap();
    let instance = builder
        .add_instance_with_materials(
            MeshInstance {
                controls: None,
                node,
                mesh,
                skin: None,
                draws: vec![Draw::new(subset, None)],
                lod: None,
                selection_group: 0,
            },
            vec![base],
        )
        .unwrap();
    let draw = builder.model().instance(instance).unwrap().draws[0].id();
    let look = builder
        .add_look(Look {
            name: "alternate".into(),
            bindings: vec![LookBinding {
                instance,
                draw,
                material: alternative,
            }],
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    assert_eq!(model.material_for_draw(instance, draw, None).unwrap(), base);
    assert_eq!(
        model.material_for_draw(instance, draw, Some(look)).unwrap(),
        alternative
    );
    assert_eq!(model.instance(instance).unwrap().draws[0].id(), draw);
}

#[test]
fn typed_vocabulary_widening_preserves_ids_and_borrowed_storage() {
    let slot_values = SlotValues::default();
    use crate::model::profiles::{Mdlx, Universal};
    let positions = ZeroVec::alloc_from_slice(&[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
    let faces = [[0, 1, 2]];
    let mut builder = ModelBuilder::<Mdlx>::with_profile("typed triangle");
    let geometry = MdlxGeometry::triangles(positions.as_borrowed(), faces.as_slice()).unwrap();
    let mesh = builder.add_mesh(Mesh::new(geometry)).unwrap();
    let material = builder
        .add_material(Material::<Mdlx> {
            name: "native surface".into(),
            surface: profiles::mdlx::Material::default(),
        })
        .unwrap();
    let narrow = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let universal = narrow.widen::<Universal>();
    assert!(universal
        .material(material)
        .unwrap()
        .surface
        .steps
        .is_empty());
    assert_eq!(
        universal
            .mesh(mesh)
            .unwrap()
            .positions()
            .as_bytes()
            .as_ptr(),
        positions.as_bytes().as_ptr()
    );
}

#[test]
fn canonical_packed_arrays_can_be_exported_without_copying() {
    use zerovec::ZeroVec;
    let packed = ZeroVec::alloc_from_slice(&[1u32, 2, 3]);
    let values = packed.as_borrowed();
    assert_eq!(values.get(1), Some(2));
    assert_eq!(values.as_bytes().as_ptr(), packed.as_bytes().as_ptr());
    assert_eq!(
        values.as_borrowed().as_bytes().as_ptr(),
        packed.as_bytes().as_ptr()
    );
}

#[test]
fn application_vocabularies_participate_in_graph_validation() {
    use crate::model::profiles::{Profile, SurfacePayload, Unavailable};
    #[derive(Clone, Debug)]
    struct Application;
    #[derive(Clone, Debug)]
    struct Surface {
        texture: Texture,
    }
    impl ValidateLinks for Surface {
        fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
            self.texture.visit_links(visitor);
        }
    }
    impl SurfacePayload for Surface {
        fn validate(&self) -> Result<(), Error> {
            Ok(())
        }
    }
    impl Profile for Application {
        type ImageResource<'a> = PathResource<'a>;
        type ModelResource<'a> = crate::model::profiles::Unavailable;
        type EffectResource<'a> = crate::model::profiles::Unavailable;
        type PhysicsResource<'a> = crate::model::profiles::Unavailable;
        type BoneOverrideResource<'a> = crate::model::profiles::Unavailable;

        type Property = profiles::Unavailable;

        type Appearance<'a> = UniversalAppearance<'a>;
        type Binding<'a> = UniversalBinding<'a>;
        type Geometry<'a> = GeometryData<'a>;
        type Surface<'a> = Surface;
        type Feature<'a> = Unavailable;
        type ModelMetadata<'a> = profiles::NoControls;
        type System<'a> = Unavailable;
        type Node<'a> = profiles::NoControls;
        type Mesh<'a> = profiles::NoControls;
        type Instance<'a> = profiles::NoControls;
        type Draw = profiles::NoControls;
        type Clip<'a> = profiles::NoControls;
        type Controller<'a> = profiles::NoControls;
    }
    let mut first = ModelBuilder::<Application>::with_profile("application");
    let texture = first.add_file_texture("custom.dds").unwrap();
    let material = first
        .add_material(Material::<Application> {
            name: "custom".into(),
            surface: Surface { texture },
        })
        .unwrap();
    let graph = first.finish().unwrap();
    assert_eq!(graph.material(material).unwrap().surface.texture, texture);
    let mut foreign = ModelBuilder::<Application>::with_profile("foreign");
    foreign
        .add_material(Material::<Application> {
            name: "invalid".into(),
            surface: Surface { texture },
        })
        .unwrap();
    assert!(foreign.finish().is_err());
}

#[test]
fn native_identity_bindings_widen_without_allocating_matrices() {
    let slot_values = SlotValues::default();
    use profiles::{Mdlx, Universal};
    let mut builder = Model::<Mdlx>::builder("rig");
    let joint = builder.add_named_node("joint").unwrap();
    let skin = builder.add_skin(Skin::identity(vec![joint])).unwrap();
    let universal = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap()
    .widen::<Universal>();
    assert!(matches!(
        universal.skin(skin).unwrap().binding,
        UniversalBinding::Identity(_)
    ));
    assert_eq!(
        universal.skin(skin).unwrap().inverse_bind().get(0).unwrap()[0],
        [1., 0., 0., 0.]
    );
}

#[test]
fn explicit_joint_bindings_check_palette_cardinality() {
    let slot_values = SlotValues::default();
    use profiles::M3;
    let mut builder = Model::<M3>::builder("rig");
    let joint = builder.add_named_node("joint").unwrap();
    let binding = ExplicitBinding::new(Vec::<BindMatrix>::new()).unwrap();
    builder.add_skin(Skin::new(vec![joint], binding)).unwrap();
    assert!({
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .is_err());
}

#[test]
fn absolute_timeline_does_not_require_a_playback_clip() {
    let slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new("timeline");
    let node = builder.add_named_node("animated").unwrap();
    builder
        .add_timeline_channel(Channel::Vector3(Track {
            target: builder.model().node(node).unwrap().transform.translation,
            curve: Curve::from_samples([Time::default()], [[1., 2., 3.]], Interpolation::Step)
                .unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    assert_eq!(model.clips().len(), 0);
    assert_eq!(model.timeline_channels().len(), 1);
}

#[test]
fn native_single_appearance_cannot_publish_alternative_looks() {
    let slot_values = SlotValues::default();
    let mut draft = Model::<profiles::Mdlx>::builder("native");
    assert_eq!(draft.model().looks().len(), 1);
    assert!(draft.add_look(Look::default()).is_err());
    assert_eq!(
        {
            draft.import_values(&slot_values).unwrap();
            draft.finish()
        }
        .unwrap()
        .looks()
        .len(),
        1
    );
}

#[test]
fn tcb_curves_preserve_authoring_parameters_and_check_cardinality() {
    let parameters = ZeroVec::alloc_from_slice(&[TcbParameters {
        tension: 0.3,
        continuity: -0.2,
        bias: 0.1,
    }]);
    let curve = Curve::tcb([Time::default()], [1f32], parameters.as_borrowed()).unwrap();
    assert_eq!(curve.interpolation(), Interpolation::Tcb);
    assert!(!curve.tcb_parameters().unwrap().is_owned());
    assert_eq!(curve.tcb_parameters().unwrap().get(0), parameters.get(0));
    assert!(Curve::tcb([Time::default()], [1f32], Vec::<TcbParameters>::new()).is_err());
    assert!(Curve::from_samples([Time::default()], [1f32], Interpolation::Tcb).is_err());
}
