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
    let mut model = draft.finish().unwrap();
    assert_eq!(model.name(), "replacement");
    assert_eq!(model.metadata().blend_time, 0.25);
    assert_eq!(model.bounds(), model.metadata().bounds);
    assert!(model
        .edit(|draft| {
            draft.metadata_mut().blend_time = f64::NAN;
            Ok(())
        })
        .is_err());
    assert_eq!(model.metadata().blend_time, 0.25);
    model
        .edit(|draft| {
            draft.metadata_mut().blend_time = 0.5;
            Ok(())
        })
        .unwrap();
    let restored = model.widen::<Universal>().try_narrow::<Mdlx>().unwrap();
    assert_eq!(restored.metadata().blend_time, 0.5);
}

#[test]
fn invalid_metadata_edits_are_transactional() {
    let mut model = Model::<M2>::builder("checked").finish().unwrap();
    assert!(model
        .edit(|draft| {
            let mut bounds = Bounds::default();
            bounds.minimum[0] = f32::NAN;
            draft.metadata_mut().bounds = Some(bounds);
            Ok(())
        })
        .is_err());
    assert_eq!(model.metadata().bounds, None);
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
    assert_eq!(model.bounds(), None);
}
