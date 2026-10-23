use bevy::prelude::{Assets, Image};
use bevy_wc3::{compile_model, RenderDiagnosticKind, TextureInput, TextureRole};
use wc3::model::{conversion::ImportOptions, io, profiles::Universal};

#[test]
fn native_crystal_compiles_through_universal_with_only_consumed_role_requests() {
    let text = include_str!("fixtures/hd_capture.mdl")
        .replace("Shader_HD_DefaultUnit", "Shader_HD_Crystal");
    let model = io::mdlx::decode_mdl(&text, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let mut images = Assets::<Image>::default();
    let mut requests = Vec::new();
    let compiled = compile_model(&model, |request| {
        if let Some(TextureInput::Material { slot, .. }) = request.input {
            requests.push((slot, request.role));
        }
        Some(images.add(Image::default()))
    })
    .unwrap();
    for (slot, role) in [
        (0, TextureRole::Color),
        (1, TextureRole::Normal),
        (2, TextureRole::Data),
        (3, TextureRole::Color),
        (4, TextureRole::Color),
    ] {
        assert!(
            requests.contains(&(slot, role)),
            "missing consumed role {slot:?}: {requests:?}"
        );
    }
    assert!(
        !requests.iter().any(|(slot, _)| *slot == 5),
        "retained environment storage must not gate Crystal rendering"
    );
    assert!(!compiled
        .diagnostics()
        .iter()
        .any(|issue| issue.kind == RenderDiagnosticKind::Unsupported));
    assert!(compiled
        .diagnostics()
        .iter()
        .any(|issue| issue.kind == RenderDiagnosticKind::Approximation
            && issue.message.contains("Crystal")));
}
