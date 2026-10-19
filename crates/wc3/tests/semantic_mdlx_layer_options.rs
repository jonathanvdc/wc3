use mdl::Read as _;
use mdx::{Read as _, Write as _};
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::mdlx::{self as native, mdl, mdx};
use wc3::model::{io, AddressMode};

#[test]
fn known_layer_declarations_roundtrip_independently_of_common_lighting_and_sampler() {
    let mut source =
        native::Document::<native::V800>::decode_mdl(include_str!("fixtures/mdl/quad_model.mdl"))
            .unwrap();
    let flags = &mut source.materials_mut().next().unwrap().layers[0].shading_flags;
    flags.set_wrap_width(true);
    flags.set_wrap_height(true);
    flags.set_unlit(true);
    flags.set_back_faces_for_shadows(true);
    flags.set_ambient_occlusion(true);
    let expected = *flags;
    let model = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let material = model.materials().values().next().unwrap();
    let step = &material.surface.steps[0];
    let controls = step.state.rasterization.controls;
    assert!(controls.wrap_u && controls.wrap_v && controls.unlit);
    assert!(controls.back_faces_for_shadows && controls.ambient_occlusion);
    assert!(!step.state.rasterization.lighting.unlit);
    let image = model.values()[step.shader.inputs().binding.image].unwrap();
    assert_eq!(image.sampler.u, AddressMode::Clamp);
    assert_eq!(image.sampler.v, AddressMode::Clamp);
    let output = io::mdlx::plan::<native::V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let bytes = output.encode_mdx().unwrap();
    let decoded = native::Document::<native::V800>::decode_mdx(&bytes).unwrap();
    assert_eq!(
        decoded.materials().next().unwrap().layers[0].shading_flags,
        expected
    );
}
