use wc3::model::animation::AnimationTrack;
use wc3::model::materials::{
    Layer, LayerShadingFlags, LayerTextureSlot, Material, MaterialRenderFlags, ShaderType,
};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::scene::{Attachment, Bone, Node, NodeFlags};
use wc3::model::{mdl, mdx, V1000, V1100, V1800, V800, V900};

fn roundtrip<T: mdl::Read + mdl::Write + mdx::Read + mdx::Write>(value: &T) {
    let binary = value.encode_mdx().unwrap();
    let text = value.encode_mdl().unwrap();
    let decoded = T::decode_mdl(&text).unwrap_or_else(|error| panic!("{error}: {text}"));
    assert_eq!(decoded.encode_mdx().unwrap(), binary, "{text}");
}
#[test]
fn nodes_and_owned_kind_bits() {
    let helper =
        Node::decode_mdl("Helper \"a\" { ObjectId 3, Billboarded, Translation 0 { Linear, } }")
            .unwrap();
    assert_eq!(helper.parent_id(), u32::MAX);
    roundtrip(&helper);
    let bone = Bone::decode_mdl(
        "Bone \"b\" { GeosetAnimId None, ObjectId 4, GeosetId Multiple, Parent 3, }",
    )
    .unwrap();
    assert_eq!(bone.node().flags().bits(), 0x100);
    assert_eq!(bone.geoset_id(), u32::MAX);
    roundtrip(&bone);
    let attachment = Attachment::decode_mdl("Attachment \"c\" { Visibility 0 { DontInterp, } ObjectId 5, Path \"a\\b.mdx\", AttachmentID 2, }").unwrap();
    assert_eq!(attachment.node().flags().bits(), 0x800);
    roundtrip(&attachment);
    assert!(attachment.node().encode_mdl().is_err());
    let mut bone = bone;
    bone.node_mut().set_flags(NodeFlags(0));
    assert!(bone.encode_mdl().is_err());
    for text in [
        "Helper \"a\" { }",
        "Helper \"a\" { ObjectId 0, Translation 0 { Linear, } Translation 0 { Linear, } }",
        "Attachment \"a\" { ObjectId 0, Visibility 0 { Linear, } Visibility 0 { Linear, } }",
    ] {
        assert!(if text.starts_with("Helper") {
            Node::decode_mdl(text).is_err()
        } else {
            Attachment::decode_mdl(text).is_err()
        });
    }
}
#[test]
fn material_directives_and_classic_layers() {
    let text = "Material { Layer { FilterMode Blend, static TextureID 2 <= 0, static Alpha 0.5, BackFacesForShadows, AmbientOcclusion, } TwoSided, Unfogged, ConstantColor, PriorityPlane -3, }";
    let material = Material::<V800>::decode_mdl(text).unwrap();
    assert_eq!(material.priority_plane(), -3);
    assert_eq!(material.render_mode().bits(), 3);
    assert!(material.layers()[0].shading_flags().two_sided());
    roundtrip(&material);
    let layer =
        Layer::<V800>::decode_mdl("Layer { TextureID 0 { DontInterp, } static Alpha 1.0, }")
            .unwrap();
    roundtrip(&layer);
    let mut layer = layer;
    layer.set_texture_id(5);
    assert!(layer.encode_mdl().is_err());
    let mut material = Material::<V800>::new();
    material.set_render_mode(MaterialRenderFlags(2));
    material.set_layers(&[Layer::new()]);
    assert!(material.encode_mdl().is_err());
}
#[test]
fn version_selected_pbr_and_shaders() {
    assert!(Layer::<V800>::decode_mdl("Layer { static EmissiveGain 1.0, }").is_err());
    assert!(Layer::<V900>::decode_mdl("Layer { static FresnelOpacity 0.0, }").is_err());
    roundtrip(&Layer::<V900>::decode_mdl("Layer { static EmissiveGain 2.0, }").unwrap());
    roundtrip(
        &Layer::<V1000>::decode_mdl(
            "Layer { static FresnelColor { 0.1, 0.2, 0.3 }, FresnelOpacity 0 { Linear, } }",
        )
        .unwrap(),
    );
    roundtrip(
        &Material::<V900>::decode_mdl("Material { Shader \"Shader_HD_DefaultUnit\", Layer {} }")
            .unwrap(),
    );
    assert!(
        Material::<V1100>::decode_mdl("Material { Shader \"Shader_HD_DefaultUnit\", }").is_err()
    );
    assert!(Layer::<V1000>::decode_mdl("Layer { Shader \"Shader_SD_Legacy\", }").is_err());
    for name in ["shader_hd_defaultunit", "Shader_HD_Crystal"] {
        let text = format!("Layer {{ Shader \"{name}\", static TextureID 7 <= 4, TextureID 0 {{ Linear, }} static EmissiveGain 3.0, }}");
        roundtrip(&Layer::<V1100>::decode_mdl(&text).unwrap());
        roundtrip(&Layer::<V1800>::decode_mdl(&text).unwrap());
    }
    roundtrip(&Layer::<V1100>::new());
}
#[test]
fn malformed_and_unrepresentable_layers() {
    for text in [
        "Layer { FilterMode Bogus, }",
        "Layer { Shader \"typo\", }",
        "Layer { static TextureID 1 <= 6, }",
        "Layer { static TextureID 1 <= 1, }",
        "Layer { static TextureID 1 <= 0, TextureID 0 { Linear, } }",
        "Layer { static Alpha 1.0, Alpha 0 { Linear, } }",
        "Layer { CoordId 0, CoordId 1, }",
    ] {
        assert!(Layer::<V1100>::decode_mdl(text).is_err(), "{text}");
    }
    let mut layer = Layer::<V1100>::new();
    layer.set_shader_type(ShaderType::new(99));
    assert!(layer.encode_mdl().is_err());
    layer.set_shader_type(ShaderType::HD_DEFAULT_UNIT);
    layer.set_texture_slots(&[LayerTextureSlot {
        texture_id: 0,
        texture_type: 1,
        track: Some(AnimationTrack::linear(vec![], None).unwrap()),
    }]);
    assert!(layer.encode_mdl().is_err());
    layer.set_texture_slots(&[]);
    layer.set_shading_flags(LayerShadingFlags(0x800));
    assert!(layer.encode_mdl().is_err());
}

#[test]
fn rejects_hidden_binary_storage_and_noncanonical_channel_order() {
    use wc3::model::animation::{LayerAlpha, LayerTextureId};
    use wc3::model::materials::LayerTrack;
    use wc3::model::mdx::Write as _;
    let layer = Layer::<V800>::decode_mdl("Layer { static TextureID 0 <= 0, }").unwrap();
    let mut layer = layer;
    layer.set_tracks(&[
        LayerTrack::Alpha(AnimationTrack::<LayerAlpha>::linear(vec![], None).unwrap()),
        LayerTrack::TextureId(AnimationTrack::<LayerTextureId>::linear(vec![], None).unwrap()),
    ]);
    assert!(layer.encode_mdl().is_err());
    let attachment = Attachment::decode_mdl("Attachment \"a\" { ObjectId 0, }").unwrap();
    let mut bytes = attachment.encode_mdx().unwrap();
    let node_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let reserved = 4 + node_size + 256;
    bytes[reserved..reserved + 4].copy_from_slice(&7u32.to_le_bytes());
    let attachment = <Attachment as mdx::Read>::decode_mdx(&bytes).unwrap();
    assert!(attachment.encode_mdl().is_err());
    let bone = Bone::decode_mdl("Bone \"a\" { ObjectId 0, }").unwrap();
    let mut bytes = bone.encode_mdx().unwrap();
    bytes[6] = 0xff; // nonzero padding after the name's terminator
    let bone = <Bone as mdx::Read>::decode_mdx(&bytes).unwrap();
    assert!(bone.encode_mdl().is_err());
}

#[test]
fn default_materials_roundtrip_at_every_supported_version() {
    use wc3::model::{V1200, V1300, V1400, V1600};
    macro_rules! check {
        ($($version:ty),*) => { $( {
            let mut material = Material::<$version>::new();
            material.set_layers(&[Layer::new()]);
            roundtrip(&material);
        } )* };
    }
    check!(V800, V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800);
}
