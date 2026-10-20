use wc3::model::profiles::Universal;
use wc3::model::{Model, ModelMetadata};

#[test]
fn invalid_rest_constants_identify_the_typed_slot_and_value() {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.values_mut().add(f32::NAN);
    let error = builder
        .finish()
        .map(|_| ())
        .expect_err("NaN must remain invalid");
    assert!(error.path().starts_with("values.Scalar["));
    assert!(error.message().contains("NaN"));
}

#[test]
fn foreign_resources_identify_the_relationship_table_and_reference() {
    let mut foreign = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let texture = foreign.add_file_texture("foreign.blp").unwrap();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.values_mut().add(Some(texture));
    let error = builder
        .finish()
        .map(|_| ())
        .expect_err("foreign resource must remain invalid");
    assert_eq!(error.path(), "relationships.values[0]");
    assert!(error.message().contains("ImageResource"));
}
