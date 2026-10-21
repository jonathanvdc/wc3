use super::*;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};

fn property_value(name: &[u8], numeric: [u32; 4], text: Option<&[u8]>) -> EffectRendererProperty {
    EffectRendererProperty {
        name: name.into(),
        numeric,
        text: text.map(Into::into),
        property_type: 0,
    }
}
#[test]
fn sampled_appearance_resolves_color_and_lut_as_distinct_interpretations() {
    let document = Document::read(include_bytes!(
        "../../../../wc3/tests/fixtures/pkb/sprite.pkb"
    ))
    .unwrap();
    let program = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    let mut renderer = program.renderers()[0].clone();
    let mut properties = renderer.properties.to_vec();
    properties.extend([
        property_value(b"AlphaRemap", [1, 0, 0, 0], None),
        property_value(b"AlphaRemap.AlphaMap", [0; 4], Some(b"sprite.png")),
        property_value(b"Atlas", [1, 0, 0, 0], None),
        property_value(b"Atlas.SubDiv", [4, 2, 0, 0], None),
        property_value(b"Atlas.Blending", [1, 0, 0, 0], None),
        property_value(b"RotateTexture", [1, 0, 0, 0], None),
        property_value(b"FlipUVs", [1, 0, 0, 0], None),
    ]);
    renderer.properties = properties.into();
    let requests: Vec<_> = image_requests(7, &renderer).collect();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].resource, requests[1].resource);
    assert_ne!(
        (requests[0].renderer, requests[0].binding),
        (requests[1].renderer, requests[1].binding)
    );
    assert_eq!(
        (requests[0].role, requests[0].color_space),
        (TextureRole::Color, TextureColorSpace::Srgb)
    );
    assert_eq!(
        (requests[1].role, requests[1].color_space),
        (TextureRole::Data, TextureColorSpace::Linear)
    );
    assert_eq!(
        requests[1].sampler,
        Sampler {
            u: AddressMode::Clamp,
            v: AddressMode::Clamp
        }
    );
    let prepared = prepare(7, &renderer, &program).unwrap().unwrap();
    assert_eq!(prepared.atlas, [2, 4]);
    assert_eq!(
        prepared.style.composition,
        AgeSpriteComposition::Sampled {
            uv: SpriteUv {
                rotate: true,
                flip_u: true,
                flip_v: true
            },
            crossfade: true,
            alpha_remap: true,
        }
    );
}

#[test]
fn facing_axes_bind_native_and_custom_semantics_with_checked_vector_layouts() {
    use wc3::model::scene::popcorn::EffectRendererInput;
    let document = Document::read(include_bytes!(
        "../../../../wc3/tests/fixtures/pkb/sprite.pkb"
    ))
    .unwrap();
    let program = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    for (mode, facing) in [
        (0, SpriteFacing::Camera),
        (2, SpriteFacing::Axial),
        (3, SpriteFacing::Spheroid),
        (5, SpriteFacing::Planar),
    ] {
        for (semantic, name) in [(4, None), (8, None), (101, Some(b"Axis".as_slice()))] {
            let mut renderer = program.renderers()[0].clone();
            let mut properties = renderer.properties.to_vec();
            properties.push(property_value(b"BillboardingMode", [mode, 0, 0, 0], None));
            renderer.properties = properties.into();
            let position = renderer
                .inputs
                .iter()
                .find(|input| input.semantic == 0)
                .unwrap()
                .field;
            let mut inputs = renderer.inputs.to_vec();
            inputs.extend([
                EffectRendererInput {
                    semantic,
                    field: position,
                    additional_name: name.map(Into::into),
                    additional_type: None,
                },
                EffectRendererInput {
                    semantic: 102,
                    field: position,
                    additional_name: Some(b"NormalAxis".as_slice().into()),
                    additional_type: None,
                },
            ]);
            renderer.inputs = inputs.into();
            let prepared = prepare(0, &renderer, &program).unwrap().unwrap();
            assert_eq!(prepared.facing, facing);
            assert_eq!(prepared.axes, [Some(position), Some(position)]);
            let mut inputs = renderer.inputs.to_vec();
            let scalar = inputs
                .iter()
                .find(|input| input.semantic == 1)
                .unwrap()
                .field;
            inputs.last_mut().unwrap().field = scalar;
            renderer.inputs = inputs.into();
            assert!(prepare(0, &renderer, &program).is_err());
        }
    }
}

#[test]
fn soft_intersections_do_not_discard_supported_billboard_geometry() {
    let document = Document::read(include_bytes!(
        "../../../../wc3/tests/fixtures/pkb/sprite.pkb"
    ))
    .unwrap();
    let program = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    let mut renderer = program.renderers()[0].clone();
    let mut properties = renderer.properties.to_vec();
    properties.push(property_value(b"SoftParticles", [u32::MAX, 0, 0, 0], None));
    renderer.properties = properties.into();
    assert!(prepare(0, &renderer, &program).unwrap().is_some());
    assert_eq!(image_requests(0, &renderer).count(), 1);
}

#[test]
fn capsule_geometry_requires_longitudinal_axis_and_ignores_optional_second_axis() {
    let document = Document::read(include_bytes!(
        "../../../../wc3/tests/fixtures/pkb/sprite.pkb"
    ))
    .unwrap();
    let program = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    let mut renderer = program.renderers()[0].clone();
    let mut properties = renderer.properties.to_vec();
    properties.push(property_value(b"BillboardingMode", [4, 0, 0, 0], None));
    renderer.properties = properties.into();
    let fallback = prepare(0, &renderer, &program).unwrap().unwrap();
    assert_eq!(fallback.facing, SpriteFacing::Capsule);
    assert!(!fallback.capsule_geometry());
    let mut axis = renderer
        .inputs
        .iter()
        .find(|input| input.semantic == 0)
        .unwrap()
        .clone();
    axis.semantic = 4;
    axis.additional_name = None;
    let mut inputs = renderer.inputs.to_vec();
    inputs.push(axis.clone());
    renderer.inputs = inputs.clone().into();
    let capsule = prepare(0, &renderer, &program).unwrap().unwrap();
    assert!(capsule.capsule_geometry());
    assert!(capsule.facing.longitudinal_axis());
    assert_eq!(capsule.facing.code(), 4);
    axis.semantic = 5;
    inputs.push(axis);
    renderer.inputs = inputs.into();
    assert!(prepare(0, &renderer, &program)
        .unwrap()
        .unwrap()
        .capsule_geometry());
}
