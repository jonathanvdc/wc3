use wc3::model::error::{ConstantValue, ErrorKind, Owner, ReferenceError, Segment};
use wc3::model::profiles::{mdlx, Mdlx, Universal};
use wc3::model::{FeatureKind, Model, ModelMetadata};

#[test]
fn invalid_rest_constants_identify_the_actual_typed_slot_and_value() {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.values_mut().add(1f32);
    let bad = builder.values_mut().add(f32::NAN);
    let error = builder.finish().unwrap_err();
    assert!(
        matches!(error.location().segments(),[Segment::Owner(Owner::Value(slot))] if *slot==bad.slot())
    );
    assert!(
        matches!(error.kind(),ErrorKind::Reference(ReferenceError::InvalidConstant {value:ConstantValue::Scalar(value)}) if value.is_nan())
    );
}

#[test]
fn foreign_resources_identify_the_actual_property_and_target() {
    let mut foreign = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let texture = foreign.add_file_texture("foreign.blp").unwrap();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.values_mut().add(1f32);
    builder.values_mut().add(false);
    let bad = builder.values_mut().add(Some(texture));
    let error = builder.finish().unwrap_err();
    assert!(
        matches!(error.location().segments(),[Segment::Owner(Owner::Value(slot)),Segment::Field(field)] if *slot==bad.slot() && field=="value")
    );
    assert!(
        matches!(error.kind(),ErrorKind::Reference(ReferenceError::Foreign {target:Owner::ImageResource(id)}) if *id==texture.resource)
    );
}

#[test]
fn removed_resources_are_distinct_from_foreign_references() {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let texture = builder.add_file_texture("removed.blp").unwrap();
    let slot = builder.values_mut().add(Some(texture));
    builder.remove_image_resource(texture.resource).unwrap();
    let error = builder.finish().unwrap_err();
    assert!(
        matches!(error.kind(),ErrorKind::Reference(ReferenceError::Unknown {target:Owner::ImageResource(id)}) if *id==texture.resource)
    );
    assert!(
        matches!(error.location().segments().first(),Some(Segment::Owner(Owner::Value(value))) if *value==slot.slot())
    );
}

#[test]
fn feature_kind_mismatch_retains_owner_target_expected_and_actual_categories() {
    let mut builder = Model::<Mdlx>::builder("wrong feature");
    let node = builder.add_named_node("Bone").unwrap();
    let light = builder.construct::<mdlx::Light>();
    let feature = builder.add_feature(node, mdlx::Node::Light(light)).unwrap();
    let mut definition = builder.model().node(node).unwrap().clone();
    definition.properties.bone = Some(mdlx::Bone {
        mesh: None,
        animation: Some(feature),
    });
    builder.set_node(node, definition).unwrap();
    let error = builder.finish().unwrap_err();
    assert!(
        matches!(error.location().segments().first(),Some(Segment::Owner(Owner::Node(id))) if *id==node)
    );
    assert!(
        matches!(error.kind(),ErrorKind::Reference(ReferenceError::FeatureKindMismatch {target,expected,actual:FeatureKind::Other})
        if *target==feature && expected.as_slice()==[FeatureKind::MdlxGeometryAnimation])
    );
}
