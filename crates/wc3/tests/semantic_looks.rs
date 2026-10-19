use glam::Vec3;
use wc3::model::common::PackedArray;
use wc3::model::io::geometry;
use wc3::model::profiles::{Mdlx, Universal};
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{
    Draw, DrawId, InstanceId, Look, LookBinding, LookId, Material, MaterialId, Mesh, MeshInstance,
    Model, ModelBuilder,
};

fn fixture() -> (Model<'static>, InstanceId, [DrawId; 2], [MaterialId; 2]) {
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("placement").unwrap();
    let mut mesh = Mesh::triangles(
        (vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])
            .into_iter()
            .map(Vec3::from_array)
            .collect::<PackedArray<_>>(),
        vec![[0, 1, 2]],
    );
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let materials = [
        builder
            .add_material(
                Material {
                    name: "".into(),
                    surface: Default::default(),
                }
                .named("first"),
            )
            .unwrap(),
        builder
            .add_material(
                Material {
                    name: "".into(),
                    surface: Default::default(),
                }
                .named("second"),
            )
            .unwrap(),
    ];
    let mut instance = MeshInstance::new(node, mesh);
    instance.draws = vec![Draw::new(subset, None), Draw::new(subset, None)];
    let draws = [instance.draws[0].id(), instance.draws[1].id()];
    let instance = builder
        .add_instance_with_materials(instance, materials.to_vec())
        .unwrap();
    (builder.finish().unwrap(), instance, draws, materials)
}

#[test]
fn reordering_draws_retains_material_and_projection_identity() {
    use wc3::model::conversion::{Context, ExportOptions};
    let (mut model, instance, draws, materials) = fixture();
    model
        .edit(|draft| {
            let mut value = draft.model().instance(instance).unwrap().clone();
            value.draws.reverse();
            draft.set_instance(instance, value)
        })
        .unwrap();
    for (draw, material) in draws.into_iter().zip(materials) {
        assert_eq!(
            model.material_for_draw(instance, draw, None).unwrap(),
            material
        );
    }
    let projected = geometry::project(
        &model,
        instance,
        None,
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap();
    assert_eq!(projected[0].source_draw, draws[1]);
    assert_eq!(projected[0].material, materials[1]);
}

#[test]
fn incomplete_alternative_is_rejected_without_default_fallback() {
    let (mut model, instance, draws, materials) = fixture();
    let original = model.default_look();
    assert!(model
        .edit(|draft| {
            draft.add_look(Look {
                name: "incomplete".into(),
                bindings: vec![LookBinding {
                    instance,
                    draw: draws[0],
                    material: materials[1],
                }],
            })?;
            Ok(())
        })
        .is_err());
    assert_eq!(model.default_look(), original);
    assert_eq!(model.looks().len(), 1);
}

#[test]
fn duplicate_and_foreign_draw_assignments_are_rejected() {
    let mut slot_values = SlotValues::default();
    let (mut model, instance, draws, materials) = fixture();
    let look_id = model.default_look().unwrap();
    let original = model.look(look_id).unwrap().clone();
    let mut duplicate = original.clone();
    duplicate.bindings.push(duplicate.bindings[0].clone());
    assert!(model
        .edit(|draft| draft.set_look(look_id, duplicate))
        .is_err());
    let mut foreign = original;
    foreign.bindings[0].draw = Draw::<Mdlx>::new(
        model.instance(instance).unwrap().draws[0].subset,
        SlotConstruct::construct(&mut slot_values),
    )
    .id();
    assert!(model
        .edit(|draft| draft.set_look(look_id, foreign))
        .is_err());
    assert_eq!(
        model.material_for_draw(instance, draws[0], None).unwrap(),
        materials[0]
    );
}

#[test]
fn removal_requires_updating_every_look_in_the_same_edit() {
    let (mut model, instance, draws, _) = fixture();
    let look_id = model.default_look().unwrap();
    assert!(model
        .edit(|draft| {
            let mut value = draft.model().instance(instance).unwrap().clone();
            value.draws.remove(0);
            draft.set_instance(instance, value)
        })
        .is_err());
    model
        .edit(|draft| {
            let mut value = draft.model().instance(instance).unwrap().clone();
            value.draws.remove(0);
            draft.set_instance(instance, value)?;
            let mut look = draft.model().look(look_id).unwrap().clone();
            look.bindings.retain(|binding| binding.draw != draws[0]);
            draft.set_look(look_id, look)
        })
        .unwrap();
    assert!(model.material_for_draw(instance, draws[0], None).is_err());
    assert_eq!(model.look(look_id).unwrap().bindings.len(), 1);
}

#[test]
fn explicit_look_works_without_a_default_and_is_authoritative() {
    let (mut model, instance, draws, materials) = fixture();
    let selected: LookId = model.default_look().unwrap();
    model
        .edit(|draft| {
            let mut look = draft.model().look(selected).unwrap().clone();
            look.bindings[0].material = materials[1];
            look.bindings[1].material = materials[0];
            draft.set_look(selected, look)?;
            draft.set_default_look(None)
        })
        .unwrap();
    assert!(model.material_for_draw(instance, draws[0], None).is_err());
    assert_eq!(
        model
            .material_for_draw(instance, draws[0], Some(selected))
            .unwrap(),
        materials[1]
    );
}

#[test]
fn draws_cannot_publish_without_a_complete_appearance() {
    let (mut model, _, _, _) = fixture();
    let selected = model.default_look().unwrap();
    assert!(model
        .edit(|draft| {
            draft.remove_look(selected);
            draft.set_default_look(None)
        })
        .is_err());
}

#[test]
fn sole_look_is_automatically_default_and_survives_profile_changes() {
    let slot_values = SlotValues::default();
    let mut builder = Model::<Mdlx>::builder("single");
    let selected = builder.model().default_look().unwrap();
    assert_eq!(builder.model().looks().len(), 1);
    assert!(builder.model().look(selected).unwrap().bindings.is_empty());
    assert!(builder.remove_look(selected).is_none());
    assert!(builder.add_look(Look::default()).is_err());
    builder
        .set_look(
            selected,
            Look {
                name: "sole".into(),
                bindings: Vec::new(),
            },
        )
        .unwrap();
    assert_eq!(builder.model().default_look(), Some(selected));
    assert!(builder.set_default_look(None).is_err());
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap()
    .widen::<Universal>()
    .try_narrow::<Mdlx>()
    .unwrap();
    assert_eq!(model.default_look(), Some(selected));
    assert_eq!(model.look(selected).unwrap().name, "sole");
}

#[test]
fn new_bindings_need_new_draw_identities_even_when_the_subset_is_shared() {
    let (mut model, instance, draws, materials) = fixture();
    assert!(model
        .edit(|draft| {
            let mut copy = draft.model().instance(instance).unwrap().clone();
            copy.draws.truncate(1);
            draft.add_instance_with_materials(copy, vec![materials[0]])?;
            Ok(())
        })
        .is_err());
    let mut other = None;
    model
        .edit(|draft| {
            let mut copy = draft.model().instance(instance).unwrap().clone();
            copy.draws = vec![Draw::new(copy.draws[0].subset, None)];
            other = Some(draft.add_instance_with_materials(copy, vec![materials[1]])?);
            Ok(())
        })
        .unwrap();
    let other = other.unwrap();
    assert!(model.material_for_draw(other, draws[0], None).is_err());
    let draw = model.instance(other).unwrap().draws[0].id();
    assert_eq!(
        model.material_for_draw(other, draw, None).unwrap(),
        materials[1]
    );
}

#[test]
fn mismatched_material_count_leaves_the_builder_unchanged() {
    let slot_values = SlotValues::default();
    let mut draft = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = draft.add_named_node("placement").unwrap();
    let mesh = draft
        .add_mesh(Mesh::triangles(
            (vec![[0.; 3]; 3])
                .into_iter()
                .map(Vec3::from_array)
                .collect::<PackedArray<_>>(),
            vec![[0, 1, 2]],
        ))
        .unwrap();
    let subset = draft.add_primary_subset(mesh, 0, 1).unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.draws.push(Draw::new(subset, None));
    assert!(draft
        .add_instance_with_materials(instance, Vec::new())
        .is_err());
    assert_eq!(draft.model().instances().len(), 0);
    assert_eq!(draft.model().looks().len(), 0);
    {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap();
}

#[test]
fn draw_and_look_identities_survive_widening_and_exact_narrowing() {
    let mut slot_values = SlotValues::default();
    use wc3::model::profiles::mdlx::Material as MdlxMaterial;
    use wc3::model::MdlxGeometry;
    let mut draft = Model::<Mdlx>::builder("typed draw");
    let node = draft.add_named_node("placement").unwrap();
    let mut mesh = Mesh::new(
        MdlxGeometry::triangles(
            (vec![[0.; 3]; 3])
                .into_iter()
                .map(Vec3::from_array)
                .collect::<PackedArray<_>>(),
            vec![[0, 1, 2]],
        )
        .unwrap(),
    );
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = draft.add_mesh(mesh).unwrap();
    let material = draft
        .add_material(Material {
            name: "surface".into(),
            surface: MdlxMaterial::default(),
        })
        .unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    let draw = Draw::new(subset, SlotConstruct::construct(&mut slot_values));
    let draw_id = draw.id();
    instance.draws.push(draw);
    let instance = draft
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    let model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap();
    let look = model.default_look();
    let broad = model.widen::<Universal>();
    assert_eq!(
        broad.material_for_draw(instance, draw_id, look).unwrap(),
        material
    );
    let narrow = broad.try_narrow::<Mdlx>().unwrap();
    assert_eq!(narrow.default_look(), look);
    assert_eq!(
        narrow
            .instance(instance)
            .unwrap()
            .draw(draw_id)
            .unwrap()
            .subset,
        subset
    );
    assert_eq!(
        narrow.material_for_draw(instance, draw_id, look).unwrap(),
        material
    );
}
