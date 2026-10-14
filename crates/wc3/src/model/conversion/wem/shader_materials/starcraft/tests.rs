use crate::model::conversion::wem::shader_materials::starcraft::{read, write};
use crate::model::conversion::wem::text;
use crate::model::semantic::profiles::wem::materials::starcraft::Body;
use crate::model::semantic::profiles::Wem;
use crate::model::wem::records::M3MaterialKind;
use crate::model::wem::records::{
    M3AnimRef, M3AnimationFlags, M3Body, M3Material, M3Standard, M3TextureLayer,
};

use crate::model::semantic::{Resource, Resources, Texture};
use crate::model::wem::{records, RawDocument, Ref};
use std::borrow::Cow;
use std::marker::PhantomData;

#[test]
fn every_embedded_starcraft_shader_keeps_its_actual_body_and_declared_family() {
    let bodies = vec![
        M3Body::Standard(Box::default()),
        M3Body::Displacement(Box::default()),
        M3Body::Composite(Box::default()),
        M3Body::Terrain(Box::default()),
        M3Body::Volume(Box::default()),
        M3Body::VolumeNoise(Box::default()),
        M3Body::Creep(Box::default()),
        M3Body::Stb(Box::default()),
        M3Body::Reflection(Box::default()),
        M3Body::LensFlare(Box::default()),
        M3Body::DataDriven(Box::default()),
    ];
    for body in bodies {
        let document = RawDocument::new();
        let source = M3Material::<records::V19, records::Abi64> {
            source_version: 29,
            kind: M3MaterialKind::Unknown(255),
            authored_data_driven: true.into(),
            body: records::NativeM3Body {
                body,
                abi: PhantomData,
            },
            ..Default::default()
        };
        let graph = read(&document, &source, &mut |_| {
            unreachable!("empty shader has no texture")
        })
        .unwrap();
        graph.validate().unwrap();
        let mut destination = RawDocument::new();
        let output = write(&mut destination, &graph, source.source_version, &|_| {
            Ok(Ref::default())
        })
        .unwrap();
        assert_eq!(output.body.body, source.body.body);
        assert_eq!(output.kind, source.kind);
        assert_eq!(output.source_version, 29);
        assert!(output.authored_data_driven.get());
    }
}

#[test]
fn shader_text_borrows_and_external_animation_controls_remain_independently_editable() {
    let mut document = RawDocument::new();
    let name = text::write_bytes(&mut document, b"independent").unwrap();
    let image = text::write_bytes(&mut document, b"textures/\x80image.dds").unwrap();
    let mut standard = M3Standard {
        name,
        ..Default::default()
    };
    standard.diffuse_layer = Some(M3TextureLayer {
        texture_path: image,
        rgb_multiply: M3AnimRef {
            anim_id: 0xffff_fffe,
            init_value: 0.25,
            null_value: 0.75,
            flags: M3AnimationFlags::from_bits_retain(0x8001),
            interp_type: records::NativeInterpolation16::Unknown(65534),
            unused: -77,
        },
        ..Default::default()
    })
    .into();
    let source = M3Material::<records::V19, records::Abi64> {
        body: records::NativeM3Body {
            body: M3Body::Standard(Box::new(standard)),
            abi: PhantomData,
        },
        ..Default::default()
    };
    let mut textures = Resources::<Wem>::builder();
    let mut graph = read(&document, &source, &mut |path| {
        let mut texture = Texture::file("");
        texture.resource = Resource::Path(path);
        Ok(textures.add_texture(texture))
    })
    .unwrap();
    let textures = textures.finish();
    let Body::Standard(body) = &mut graph.body else {
        unreachable!()
    };
    assert!(matches!(body.name, Cow::Borrowed("independent")));
    let layer = body.diffuse_layer.as_mut().unwrap();
    let id = layer.texture.unwrap();
    let Resource::Path(path) = &textures.texture(id).unwrap().resource else {
        unreachable!()
    };
    assert!(matches!(path, Cow::Borrowed(_)));
    assert_eq!(path.as_ref(), b"textures/\x80image.dds");
    assert_eq!(layer.rgb_multiply.control.source_id, 0xffff_fffe);
    layer.rgb_multiply.initial = 0.5;
    let mut destination = RawDocument::new();
    let shared_path = text::write_bytes(&mut destination, path).unwrap();
    let output = write(&mut destination, &graph, 0, &|texture| {
        assert_eq!(texture, id);
        Ok(shared_path)
    })
    .unwrap();
    let M3Body::Standard(body) = output.body.body else {
        unreachable!()
    };
    let layer = body.diffuse_layer.value.unwrap();
    assert_eq!(layer.rgb_multiply.init_value, 0.5);
    assert_eq!(layer.rgb_multiply.null_value, 0.75);
    assert_eq!(layer.rgb_multiply.anim_id, 0xffff_fffe);
    assert_eq!(layer.rgb_multiply.unused, -77);
    assert_eq!(layer.rgb_multiply.flags.bits(), 0x8001);
    assert_eq!(
        layer.rgb_multiply.interp_type,
        records::NativeInterpolation16::Unknown(65534)
    );
    assert_eq!(
        text::read_bytes(&destination, layer.texture_path)
            .unwrap()
            .as_ref(),
        b"textures/\x80image.dds"
    );
}
