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
