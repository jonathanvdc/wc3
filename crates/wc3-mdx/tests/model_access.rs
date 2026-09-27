use wc3_mdx::{AnyVersionModel, Model, ModelAccess, V1800, V800};

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
