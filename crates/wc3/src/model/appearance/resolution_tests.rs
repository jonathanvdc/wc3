use super::*;
use crate::model::controls::{diablo3 as d3_controls, diablo4 as d4_controls};
use crate::model::{Material, MaterialSurface, Mesh, MeshInstance, ModelBuilder, Node};

fn fixture(
    appearance: UniversalAppearance<'static>,
    controls: MeshControls,
    mut builder: ModelBuilder<'static>,
) -> (Model<'static>, InstanceId) {
    builder.set_appearance(appearance);
    let node = builder.add_node(Node::named("root")).unwrap();
    let mesh = builder
        .add_mesh(
            Mesh::triangles([[0.; 3], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]])
                .with_controls(Some(controls)),
        )
        .unwrap();
    let instance = builder.add_instance(MeshInstance::new(node, mesh)).unwrap();
    (builder.finish().unwrap(), instance)
}

fn material(builder: &mut ModelBuilder<'static>) -> MaterialId {
    builder
        .add_material(Material {
            name: "".into(),
            surface: MaterialSurface::Layered(Box::default()),
        })
        .unwrap()
}

#[test]
fn diablo3_resolution_preserves_alternative_and_variant_identity() {
    let mut builder = ModelBuilder::new("d3 assignment");
    let first = material(&mut builder);
    let second = material(&mut builder);
    let mut appearance = diablo3::Appearance::default();
    let slot = appearance.add_slot(diablo3::Slot::named("body"));
    let a = appearance.add_variant(slot, first).unwrap();
    let b = appearance.add_variant(slot, second).unwrap();
    let look_a = appearance.add_look("a", vec![(slot, a)]).unwrap();
    let look_b = appearance.add_look("b", vec![(slot, b)]).unwrap();
    let (model, instance) = fixture(
        UniversalAppearance::D3(appearance),
        MeshControls::D3(d3_controls::MeshControls {
            material_slot: Some(slot),
            ..Default::default()
        }),
        builder,
    );
    assert!(model
        .resolve_instance_material(instance, MaterialSelection::Default)
        .is_err());
    assert_eq!(
        model
            .resolve_instance_material(instance, MaterialSelection::Diablo3(look_a))
            .unwrap(),
        first
    );
    assert_eq!(
        model
            .resolve_instance_material(instance, MaterialSelection::Diablo3(look_b))
            .unwrap(),
        second
    );
    assert_eq!(
        model
            .resolve_instance_material(
                instance,
                MaterialSelection::Diablo3Variant { slot, variant: b }
            )
            .unwrap(),
        second
    );
    let mut foreign = diablo3::Appearance::default();
    let foreign_slot = foreign.add_slot(diablo3::Slot::named("body"));
    let variant = foreign.add_variant(foreign_slot, first).unwrap();
    assert!(model
        .resolve_instance_material(
            instance,
            MaterialSelection::Diablo3Variant { slot, variant }
        )
        .is_err());
}

#[test]
fn diablo4_single_variant_default_and_explicit_family_checks() {
    let mut builder = ModelBuilder::new("d4 assignment");
    let expected = material(&mut builder);
    let mut appearance = diablo4::Appearance::default();
    let slot = appearance.add_slot(diablo4::Slot::new(42, -1, 0));
    let variant = appearance.add_variant(slot, expected).unwrap();
    let (model, instance) = fixture(
        UniversalAppearance::D4(appearance),
        MeshControls::D4(d4_controls::MeshControls {
            material_slot: Some(slot),
            ..Default::default()
        }),
        builder,
    );
    assert_eq!(
        model
            .resolve_instance_material(instance, MaterialSelection::Default)
            .unwrap(),
        expected
    );
    assert_eq!(
        model
            .resolve_instance_material(
                instance,
                MaterialSelection::Diablo4Variant { slot, variant }
            )
            .unwrap(),
        expected
    );
    let mut other = diablo3::Appearance::default();
    let look = other.add_look("foreign family", vec![]).unwrap();
    assert!(model
        .resolve_instance_material(instance, MaterialSelection::Diablo3(look))
        .is_err());
}

#[test]
fn sole_alternative_default_uses_its_binding_instead_of_first_variant() {
    let mut builder = ModelBuilder::new("sole authored alternative");
    let first = material(&mut builder);
    let selected = material(&mut builder);
    let mut appearance = diablo3::Appearance::default();
    let slot = appearance.add_slot(diablo3::Slot::named("body"));
    appearance.add_variant(slot, first).unwrap();
    let variant = appearance.add_variant(slot, selected).unwrap();
    appearance
        .add_look("only look", vec![(slot, variant)])
        .unwrap();
    let (model, instance) = fixture(
        UniversalAppearance::D3(appearance),
        MeshControls::D3(d3_controls::MeshControls {
            material_slot: Some(slot),
            ..Default::default()
        }),
        builder,
    );
    assert_eq!(
        model
            .resolve_instance_material(instance, MaterialSelection::Default)
            .unwrap(),
        selected
    );
}
