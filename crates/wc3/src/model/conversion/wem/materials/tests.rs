use crate::model::conversion::wem::materials::{read, write};
use crate::model::semantic::profiles::wem::materials::{
    BlendMode, Body, CombinerOp, CombinerStage, CommonSurface, CompositeLayer, CompositeOp,
    CullMode, Feature, Input, LayerFeature, LegacySlot, MaterialFlags, PbrSlot, SlotInput,
    SurfaceChannel, UVMappingMode,
};
use crate::model::semantic::profiles::Universal;
use crate::model::semantic::{AddressMode, ColorSpace, Resources, Sampler, Texture};
use crate::model::wem::RawDocument;
use std::collections::BTreeMap;

#[test]
fn all_common_shader_bodies_keep_texture_identity_and_authored_coefficients() {
    let mut resources = Resources::<Universal>::builder();
    let texture = resources.add_texture(Texture::file("image.blp"));
    let input = Input {
        texture: Some(texture),
        constant: [0.25, 0.5, 0.75, 1.],
        uv_set: 3,
        mapping: UVMappingMode::Unknown(221),
        color_space: ColorSpace::Unknown(222),
        sampler: Sampler {
            u: AddressMode::Mirror,
            v: AddressMode::Unknown(223),
        },
        uv_transform: [[2., 0., 0.25], [0., 3., 0.5]],
        weight: 0.75,
    };
    let bodies = vec![
        Body::Composite {
            layers: vec![CompositeLayer::new(
                input.clone(),
                SurfaceChannel::Coverage,
                CompositeOp::Unknown(224),
            )],
            diffuse: [0.1; 4],
            emissive: [0.2; 4],
            specular: [0.3; 4],
            specular_exponent: 15.,
            environment: 0.8,
            simulate_roughness: true,
        },
        Body::Combiners {
            stages: vec![CombinerStage::new(
                input.clone(),
                CombinerOp::MaskedMod,
                CombinerOp::Unknown(225),
            )],
            diffuse: [0.4; 4],
            emissive: [0.5; 4],
        },
        Body::LegacyDeferred {
            slots: vec![SlotInput::new(LegacySlot::Unknown(226), input.clone())],
            diffuse: [0.6; 4],
            emissive: [0.7; 4],
            specular: [0.8; 4],
            specular_exponent: 32.,
            environment: 0.9,
        },
        Body::PbrDeferred {
            slots: vec![SlotInput::new(PbrSlot::Unknown(227), input.clone())],
            base_color: [0.8; 4],
            metallic: 0.3,
            roughness: 0.4,
            emissive: [0.9; 3],
        },
    ];
    for body in bodies {
        let layer = body.layer_ids().next().unwrap();
        let mut raw = RawDocument::new();
        let surface = CommonSurface {
            blend: BlendMode::Unknown(0xffff_ffff),
            cull: CullMode::Unknown(228),
            depth_test: true,
            depth_write: false,
            depth_bias: -0.25,
            alpha_test_threshold: 0.375,
            priority: -12,
            flags: MaterialFlags::from_bits_retain(0x8000_0005),
            body,
            features: vec![
                LayerFeature {
                    id: 2,
                    layer,
                    operation: Feature::Fresnel {
                        color: [0.1, 0.2, 0.3],
                        exponent: 2.,
                        range: [0.2, 0.8],
                        team_color: 0.5,
                    },
                },
                LayerFeature {
                    id: 8,
                    layer,
                    operation: Feature::UvAnimation {
                        scroll_rate: [0.1, 0.2],
                        rotate_rate: 0.3,
                        scale_rate: [1., 2.],
                    },
                },
                LayerFeature {
                    id: 9,
                    layer,
                    operation: Feature::Shading {
                        unlit: true,
                        two_sided: true,
                        unfogged: false,
                        no_depth_test: false,
                        no_depth_write: true,
                    },
                },
            ],
        };
        let table = BTreeMap::from([(texture, 0)]);
        let encoded = write(&mut raw, &surface, &table).unwrap();
        let decoded = read(&raw, &encoded, &[texture]).unwrap();
        assert_eq!(decoded.blend, surface.blend);
        assert_eq!(decoded.cull, surface.cull);
        assert_eq!(decoded.flags, surface.flags);
        assert_eq!(decoded.priority, -12);
        assert_eq!(decoded.depth_bias, -0.25);
        assert_eq!(decoded.features.len(), 3);
        let decoded_layer = decoded.body.layer_ids().next().unwrap();
        assert!(decoded
            .features
            .iter()
            .all(|feature| feature.layer == decoded_layer));
        assert_eq!(
            decoded
                .features
                .iter()
                .map(|feature| feature.id)
                .collect::<Vec<_>>(),
            [2, 8, 9]
        );
        assert_eq!(
            decoded.body.input(decoded_layer).unwrap().texture,
            Some(texture)
        );
        let mut rebuilt = RawDocument::new();
        assert_eq!(write(&mut rebuilt, &decoded, &table).unwrap(), encoded);
    }
}

#[test]
fn layer_features_follow_identity_when_layers_move_and_reject_foreign_targets() {
    let input = || Input {
        texture: None,
        constant: [1.; 4],
        uv_set: 0,
        mapping: UVMappingMode::default(),
        color_space: ColorSpace::default(),
        sampler: Sampler::default(),
        uv_transform: [[1., 0., 0.], [0., 1., 0.]],
        weight: 1.,
    };
    let layer = CompositeLayer::new(input(), SurfaceChannel::Coverage, CompositeOp::AlphaBlend);
    let target = layer.id();
    let other = CompositeLayer::new(input(), SurfaceChannel::Color, CompositeOp::AlphaBlend);
    let mut surface = CommonSurface {
        body: Body::Composite {
            layers: vec![layer, other],
            diffuse: [1.; 4],
            emissive: [0.; 4],
            specular: [0.; 4],
            specular_exponent: 0.,
            environment: 0.,
            simulate_roughness: false,
        },
        features: vec![LayerFeature {
            id: 7,
            layer: target,
            operation: Feature::Shading {
                unlit: true,
                two_sided: false,
                unfogged: false,
                no_depth_test: false,
                no_depth_write: false,
            },
        }],
        blend: BlendMode::default(),
        cull: CullMode::default(),
        depth_test: true,
        depth_write: true,
        depth_bias: 0.,
        alpha_test_threshold: 0.,
        priority: 0,
        flags: MaterialFlags::default(),
    };
    let Body::Composite { layers, .. } = &mut surface.body else {
        unreachable!()
    };
    layers.swap(0, 1);
    let mut output = RawDocument::new();
    let encoded = write(&mut output, &surface, &BTreeMap::new()).unwrap();
    let decoded = read(&output, &encoded, &[]).unwrap();
    let ids: Vec<_> = decoded.body.layer_ids().collect();
    assert_eq!(decoded.features[0].layer, ids[1]);
    surface.features[0].layer =
        CompositeLayer::new(input(), SurfaceChannel::Coverage, CompositeOp::AlphaBlend).id();

    assert!(write(&mut RawDocument::new(), &surface, &BTreeMap::new()).is_err());
}
