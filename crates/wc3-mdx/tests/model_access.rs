use wc3_mdx::geometry::BindPoseMatrix;
use wc3_mdx::{AnyVersionModel, Model, ModelAccess, TryModelAccess, ValueError, V1800, V800, V900};

fn replace_durations(model: &mut impl ModelAccess) {
    model.set_global_sequences(&[100, 250]);
    assert_eq!(model.global_sequences(), [100, 250]);
}

#[test]
fn shared_accessors_work_for_typed_and_runtime_models() {
    let mut typed = Model::<V800>::new();
    replace_durations(&mut typed);

    let mut runtime = AnyVersionModel::V1800(Model::<V1800>::new());
    replace_durations(&mut runtime);
    assert_eq!(runtime.global_sequences(), [100, 250]);
    assert_eq!(runtime.model_info(), None);
}

fn check_bind_poses(model: &mut impl TryModelAccess, supported: bool) {
    let poses = [BindPoseMatrix([0.0; 12])];
    if supported {
        assert!(model.try_bind_poses().unwrap().is_empty());
        model.try_set_bind_poses(&poses).unwrap();
        assert_eq!(model.try_bind_poses().unwrap(), poses);
    } else {
        assert!(
            matches!(model.try_bind_poses(), Err(ValueError::UnsupportedVersion { tag, minimum: 900, actual: 800 }) if tag == *b"BPOS")
        );
        assert!(model.try_set_bind_poses(&poses).is_err());
    }
}

#[test]
fn checked_accessors_distinguish_unsupported_and_empty() {
    let mut old = Model::<V800>::new();
    check_bind_poses(&mut old, false);
    assert_eq!(old.chunks().len(), 1);

    let mut modern = Model::<V1800>::new();
    check_bind_poses(&mut modern, true);
    assert_eq!(modern.bind_poses(), [BindPoseMatrix([0.0; 12])]);
    assert!(Model::<V900>::new().bind_poses().is_empty());

    let mut runtime = AnyVersionModel::V800(Model::new());
    check_bind_poses(&mut runtime, false);
    assert!(runtime.try_bind_poses().is_err());
    assert!(
        matches!(runtime.try_face_fx(), Err(ValueError::UnsupportedVersion { tag, .. }) if tag == *b"FAFX")
    );
    assert!(
        matches!(runtime.try_popcorn_emitters(), Err(ValueError::UnsupportedVersion { tag, .. }) if tag == *b"CORN")
    );
    assert!(runtime.try_set_face_fx(&[]).is_err());
    assert!(runtime.try_set_popcorn_emitters(&[]).is_err());

    let mut runtime = AnyVersionModel::V1800(Model::new());
    assert!(runtime.try_face_fx().unwrap().is_empty());
    assert!(runtime.try_popcorn_emitters().unwrap().is_empty());
    runtime.try_set_face_fx(&[]).unwrap();
    runtime.try_set_popcorn_emitters(&[]).unwrap();
}
