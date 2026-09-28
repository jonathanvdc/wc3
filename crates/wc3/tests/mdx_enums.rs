use mdl::{Read as _, Write as _};
use mdx::{Read as _, Write as _};
use wc3::model::emitters::{Particle2FilterMode, Particle2Frames, ParticleEmitter2};
use wc3::model::materials::{Layer, LayerFilterMode, LayerShadingFlags};
use wc3::model::V800;
use wc3::model::{mdl, mdx};

#[derive(Debug, PartialEq, mdx::Read, mdx::Write, mdx::Value)]
#[mdx(value = u32)]
enum Strict {
    #[mdx(value = 7)]
    Seven,
}

#[test]
fn numeric_choices_preserve_binary_values_and_report_offsets() {
    for raw in [0u32, 1, 2, 3, 4, 5, 6, 99, u32::MAX] {
        let bytes = raw.to_le_bytes();
        let mode = LayerFilterMode::decode_mdx(&bytes).unwrap();
        assert_eq!(mode, LayerFilterMode::from_raw(raw));
        assert_eq!(mode.raw(), raw);
        assert_eq!(mode.encode_mdx().unwrap(), bytes);
    }
    assert_eq!(Strict::from_raw(7), Some(Strict::Seven));
    assert_eq!(Strict::from_raw(8), None);
    assert_eq!(Strict::Seven.raw(), 7);
    assert_eq!(Strict::Seven.encode_mdx().unwrap(), 7u32.to_le_bytes());
    let bytes = [0, 0, 0, 0, 8, 0, 0, 0];
    let mut cursor = mdx::Cursor::new(&bytes);
    cursor.read::<u32>().unwrap();
    assert_eq!(
        cursor.read::<Strict>(),
        Err(mdx::ReadError::UnknownEnumValue {
            enum_name: "Strict",
            value: 8,
            offset: 4,
        })
    );
    assert!(matches!(
        Strict::decode_mdx(&[7, 0]),
        Err(mdx::ReadError::UnexpectedEnd { .. })
    ));
}

#[test]
fn typed_storage_retains_unknown_data_during_edits() {
    let mut layer = Layer::<V800>::new();
    layer.set_filter_mode(LayerFilterMode::Unknown(99));
    let mut flags = LayerShadingFlags::from_bits_retain(0x8000_0000);
    flags.set_two_sided(true);
    layer.set_shading_flags(flags);
    let decoded = Layer::<V800>::decode_mdx(&layer.encode_mdx().unwrap()).unwrap();
    assert_eq!(decoded, layer);
    assert_eq!(decoded.shading_flags().bits(), 0x8000_0010);
    assert!(layer.encode_mdl().is_err());
    let mut emitter = ParticleEmitter2::new(wc3::model::scene::Node::new("p", 0).unwrap());
    emitter.filter_mode = Particle2FilterMode::Unknown(99);
    emitter.frames = Particle2Frames::Unknown(u32::MAX);
    assert_eq!(
        ParticleEmitter2::decode_mdx(&emitter.encode_mdx().unwrap()).unwrap(),
        emitter
    );
}

#[test]
fn layer_filter_keywords_and_unknown_rejection() {
    for (raw, keyword) in [
        "None",
        "Transparent",
        "Blend",
        "Additive",
        "AddAlpha",
        "Modulate",
        "Modulate2x",
    ]
    .iter()
    .enumerate()
    {
        let mode = LayerFilterMode::from_raw(raw as u32);
        assert_eq!(mode.encode_mdl().unwrap(), *keyword);
        assert_eq!(LayerFilterMode::decode_mdl(keyword).unwrap(), mode);
    }
    assert!(LayerFilterMode::Unknown(99).encode_mdl().is_err());
    assert!(LayerFilterMode::decode_mdl("Unknown").is_err());
}

#[test]
fn emitter_flags_interpret_overlapping_bits_and_preserve_other_bits() {
    use wc3::model::emitters::{ParticleEmitter, ParticleEmitter2, PopcornEmitter};
    use wc3::model::scene::{Node, NodeFlags};
    let mut node = Node::new("flags", 0).unwrap();
    node.set_flags(NodeFlags(0x8002_0001));
    let mut particles = ParticleEmitter2::new(node.clone());
    assert!(particles.flags().line_emitter());
    assert!(!particles.flags().unfogged());
    let mut popcorn = PopcornEmitter::new(node.clone(), "", "").unwrap();
    assert!(popcorn.flags().unfogged());
    let mut flags = popcorn.flags();
    flags.set_unfogged(false);
    flags.set_popcorn_scaling(true);
    popcorn.set_flags(flags);
    assert_eq!(popcorn.node().flags().bits(), 0x8004_0001);
    let mut flags = particles.flags();
    flags.set_unfogged(true);
    particles.set_flags(flags);
    assert_eq!(particles.node().flags().bits(), 0x8006_0001);
    let mut classic = ParticleEmitter::new(node, "").unwrap();
    let mut flags = classic.flags();
    flags.set_emitter_uses_tga(true);
    classic.set_flags(flags);
    assert_eq!(classic.node().flags().bits(), 0x8003_0001);
}
