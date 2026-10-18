use std::borrow::Cow;
use wc3::model::profiles::{mdlx, Mdlx, Universal, M2};
use wc3::model::{Bounds, Model, ModelMetadata};

#[test]
fn metadata_replacement_owns_model_identity_and_bounds() {
    let mut draft = Model::<Mdlx>::builder("old name");
    draft.set_metadata(mdlx::ModelMetadata {
        name: Cow::Borrowed("replacement"),
        bounds: Some(Bounds::default()),
        blend_time: 0.25,
        ..Default::default()
    });
    let model = draft.finish().unwrap();
    assert_eq!(model.name(), "replacement");
    assert_eq!(model.metadata().blend_time, 0.25);
    assert_eq!(model.bounds(), model.metadata().bounds);
}

#[test]
fn invalid_metadata_edits_are_transactional() {
    let mut model = Model::<M2>::builder("checked").finish().unwrap();
    assert!(model
        .edit(|draft| {
            draft.metadata_mut().bounds.minimum[0] = f32::NAN;
            Ok(())
        })
        .is_err());
    assert_eq!(model.metadata().bounds, Bounds::default());
}

#[test]
fn universal_metadata_edits_replace_native_declarations() {
    let mut model = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
    model
        .edit(|draft| {
            draft.set_metadata(ModelMetadata::M2("after".into()));
            Ok(())
        })
        .unwrap();
    assert!(matches!(model.metadata(), ModelMetadata::M2(_)));
    assert_eq!(model.name(), "after");
    assert_eq!(model.bounds(), Some(Bounds::default()));
}
