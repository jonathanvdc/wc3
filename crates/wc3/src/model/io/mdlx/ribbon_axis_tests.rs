use super::*;
use crate::model::formats::mdlx::{
    emitters::RibbonEmitter, materials::Material, scene::Node, Document, V800,
};
use crate::model::{profiles::mdlx::Node as Feature, Axis};

#[test]
fn ribbon_axis_import_roundtrip_and_strict_export_loss() {
    let mut source = Document::<V800>::new();
    source.set_materials([Material::new()]);
    let mut ribbon = RibbonEmitter::new(Node::new("ribbon", 0).unwrap());
    ribbon.rows = 1;
    ribbon.columns = 1;
    ribbon.life_span = 1.;
    ribbon.emission_rate = 1;
    source.set_ribbon_emitters([ribbon]);
    source.set_pivot_points(&[[0.; 3]]);
    let mut model = import(&source, &ImportOptions::default()).unwrap().value;
    let (id, feature) = model.features().iter().next().unwrap();
    let Feature::Ribbon(ribbon) = &feature.payload else {
        panic!("ribbon lost")
    };
    assert_eq!(ribbon.geometry.axis, Axis::PositiveY);
    let output = plan::<V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let roundtrip = import(&output, &ImportOptions::default()).unwrap().value;
    let Feature::Ribbon(ribbon) = &roundtrip.features().iter().next().unwrap().1.payload else {
        panic!("ribbon lost")
    };
    assert_eq!(ribbon.geometry.axis, Axis::PositiveY);
    let mut changed = feature.payload.clone();
    let Feature::Ribbon(ribbon) = &mut changed else {
        unreachable!()
    };
    ribbon.geometry.axis = Axis::PositiveZ;
    model
        .edit(|builder| builder.set_feature(id, changed))
        .unwrap();
    assert!(plan::<V800>(&model, &ExportOptions::default()).is_err());
}
