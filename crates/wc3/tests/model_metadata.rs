use std::borrow::Cow;
use wc3::model::profiles::{mdlx, Mdlx, Universal, M2, M3};
use wc3::model::{Bounds, Model, ModelMetadata};

#[test]
fn native_names_and_bounds_survive_metadata_projection() {
    let bounds = Bounds {
        minimum: [-1., -2., -3.],
        maximum: [4., 5., 6.],
        sphere_radius: 9.,
    };
    let mut draft = Model::<M2>::builder("authored name");
    draft.metadata_mut().bounds = bounds;
    let native = draft.finish().unwrap();
    assert!(matches!(
        native.metadata().name,
        Cow::Borrowed("authored name")
    ));
    let universal = native.widen::<Universal>();
    assert_eq!(universal.name(), "authored name");
    assert_eq!(universal.bounds(), Some(bounds));
    let restored = universal.try_narrow::<M2>().unwrap();
    assert_eq!(restored.metadata().name, "authored name");
    assert_eq!(restored.metadata().bounds, bounds);

    let native = Model::<M3>::builder("borrowed M3 name").finish().unwrap();
    assert!(matches!(
        native.metadata().name,
        Cow::Borrowed("borrowed M3 name")
    ));
    let restored = native.widen::<Universal>().try_narrow::<M3>().unwrap();
    assert_eq!(restored.name(), "borrowed M3 name");
}

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
