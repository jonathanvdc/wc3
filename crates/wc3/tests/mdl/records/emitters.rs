use wc3::model::animation::{Animatable, Track};

use wc3::model::emitters::{ParticleEmitter, ParticleEmitterFlags, RibbonEmitter};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::{Node, NodeFlags};
use wc3::model::{mdl, mdx};
fn roundtrip<T: mdl::Read + mdl::Write + mdx::Read + mdx::Write>(value: &T) {
    let bytes = value.encode_mdx().unwrap();
    let text = value.encode_mdl().unwrap();
    let decoded = T::decode_mdl(&text).unwrap_or_else(|error| panic!("{error}: {text}"));
    assert_eq!(decoded.encode_mdl().unwrap(), text);
    assert_eq!(T::decode_mdx(&bytes).unwrap().encode_mdx().unwrap(), bytes);
    assert_eq!(value.encode_mdx().unwrap(), bytes);
}
#[test]
fn particle_flags_and_all_channels_roundtrip() {
    for flags in [
        "",
        "EmitterUsesMdl,",
        "EmitterUsesTga,",
        "EmitterUsesMdl, EmitterUsesTga,",
    ] {
        let text = format!("ParticleEmitter \"a\" {{ ObjectId 0, {flags} Translation 0 {{ Linear, }} Visibility 0 {{ DontInterp, }} EmissionRate 0 {{ Linear, }} Gravity 0 {{ Hermite, }} Longitude 0 {{ Bezier, }} Latitude 0 {{ Linear, }} LifeSpan 0 {{ Linear, }} InitVelocity 0 {{ Linear, }} Path \"a\\b.mdl\", }}");
        let emitter = ParticleEmitter::decode_mdl(&text).unwrap();
        let expected = 0x1000
            | if flags.contains("Mdl") { 0x8000 } else { 0 }
            | if flags.contains("Tga") { 0x10000 } else { 0 };
        assert_eq!(emitter.node.flags.bits(), expected);
        assert_eq!(emitter.path.text(), "a\\b.mdl");
        if !flags.is_empty() {
            let output = emitter.encode_mdl().unwrap();
            assert!(output.find("ObjectId").unwrap() < output.find("EmitterUses").unwrap());
        }
        roundtrip(&emitter);
    }
    roundtrip(&ParticleEmitter::decode_mdl("ParticleEmitter \"a\" { ObjectId 0, static EmissionRate 8.0, static Gravity -0.0, static Longitude 90.0, static Latitude 2.0, static LifeSpan 3.0, static InitVelocity 4.0, }").unwrap());
}
#[test]
fn independent_emitter_fixtures_roundtrip() {
    let mut node = Node::new("particle", 7).unwrap();
    node.flags = NodeFlags(0x19008);
    let mut particle = ParticleEmitter::new(node.clone(), "a.mdl").unwrap();
    particle.gravity = Animatable::Static(3.0);
    particle.longitude = Animatable::Static(4.0);
    particle.latitude = Animatable::Static(5.0);
    particle.life_span = Animatable::Static(6.0);
    particle.initial_velocity = Animatable::Static(7.0);
    particle
        .emission_rate
        .set_track(Track::linear(vec![], None).unwrap());
    roundtrip(&particle);
    node.flags = NodeFlags(0x4000);
    let mut ribbon = RibbonEmitter::new(node);
    ribbon.height_above = Animatable::Static(2.0);
    ribbon.height_below = Animatable::Static(3.0);
    ribbon.alpha = Animatable::Static(0.0);
    ribbon.color = Animatable::Static([0.1, 0.2, 0.3]);
    ribbon.life_span = 5.0;
    ribbon.texture_slot = Animatable::Static(0);
    ribbon.emission_rate = u32::MAX;
    ribbon.rows = 2;
    ribbon.columns = 3;
    ribbon.material_id = 4;
    ribbon.gravity = -0.0;
    ribbon
        .texture_slot
        .set_track(Track::linear(vec![], None).unwrap());
    ribbon.alpha.set_track(Track::linear(vec![], None).unwrap());
    roundtrip(&ribbon);
    assert!(ribbon.encode_mdl().unwrap().contains("Gravity -0.0,"));
}
#[test]
fn ribbon_static_alias_and_all_channels_roundtrip() {
    let ribbon = RibbonEmitter::decode_mdl("RibbonEmitter \"Trail\" { ObjectId 22, Parent 6, static HeightAbove 4.0, static HeightBelow 4.0, static Alpha 1.0, static Color { 1.0, 1.0, 1.0 }, LifeSpan 0.5, TextureSlot 0, EmissionRate 30, Rows 1, Columns 1, MaterialID 1, }").unwrap();
    assert_eq!(ribbon.node.flags.bits(), 0x4000);
    assert_eq!(ribbon.emission_rate, 30);
    let text = ribbon.encode_mdl().unwrap();
    assert!(text.contains("static TextureSlot 0,"));
    assert!(!text.contains("Gravity"));
    roundtrip(&ribbon);
    roundtrip(&RibbonEmitter::decode_mdl("RibbonEmitter \"a\" { ObjectId 0, Visibility 0 { Linear, } HeightAbove 0 { Linear, } HeightBelow 0 { Linear, } Alpha 0 { Linear, } Color 0 { Linear, } TextureSlot 0 { Linear, } }").unwrap());
}
#[test]
fn duplicate_channels_and_scalar_ranges_are_strict() {
    for body in [
        "EmitterUsesMdl, EmitterUsesMdl,",
        "static Gravity 1.0, Gravity 0 { Linear, }",
        "Visibility 0 { Linear, } Visibility 0 { Linear, }",
        "static Visibility 0.0,",
        "EmissionRate 3.0,",
        "Reserved 0,",
    ] {
        let text = format!("ParticleEmitter \"a\" {{ ObjectId 0, {body} }}");
        assert!(ParticleEmitter::decode_mdl(&text).is_err(), "{body}");
    }
    for body in [
        "TextureSlot 0, static TextureSlot 0,",
        "static TextureSlot 0, TextureSlot 0 { Linear, }",
        "TextureSlot 0 { Linear, } TextureSlot 0,",
        "TextureSlot -1,",
        "TextureSlot 4294967296,",
        "EmissionRate 1.5,",
        "EmissionRate -1,",
        "Rows 4294967296,",
        "static Visibility 0.0,",
        "static LifeSpan 0.0,",
    ] {
        let text = format!("RibbonEmitter \"a\" {{ ObjectId 0, {body} }}");
        assert!(RibbonEmitter::decode_mdl(&text).is_err(), "{body}");
    }
    let error = RibbonEmitter::decode_mdl(
        "RibbonEmitter \"a\" { ObjectId 0, TextureSlot 1, TextureSlot 0 { Linear, } }",
    )
    .unwrap_err();
    assert_eq!(error.kind, mdl::ReadErrorKind::DuplicateField);
}
#[test]
fn writers_export_animation_over_base_and_reject_unknown_flags_and_padding() {
    let mut particle = ParticleEmitter::decode_mdl(
        "ParticleEmitter \"a\" { ObjectId 0, EmissionRate 0 { Linear, } }",
    )
    .unwrap();
    particle.emission_rate.set_value(1.0);
    let decoded = ParticleEmitter::decode_mdl(&particle.encode_mdl().unwrap()).unwrap();
    assert_eq!(decoded.emission_rate.value(), Some(&0.0));
    assert_eq!(
        decoded.emission_rate.track(),
        particle.emission_rate.track()
    );
    particle.emission_rate = Animatable::Static(0.0);
    particle.node.flags = ParticleEmitterFlags(0x21000);
    assert!(particle.encode_mdl().is_err());
    particle.node.flags = ParticleEmitterFlags(0x1000);
    let mut bytes = particle.encode_mdx().unwrap();
    let node_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let padding = 4 + node_size + 16 + 256;
    bytes[padding..padding + 4].copy_from_slice(&1u32.to_le_bytes());
    assert!(ParticleEmitter::decode_mdx(&bytes)
        .unwrap()
        .encode_mdl()
        .is_err());
    let mut ribbon =
        RibbonEmitter::decode_mdl("RibbonEmitter \"a\" { ObjectId 0, TextureSlot 0 { Linear, } }")
            .unwrap();
    ribbon.texture_slot.set_value(3);
    let decoded = RibbonEmitter::decode_mdl(&ribbon.encode_mdl().unwrap()).unwrap();
    assert_eq!(decoded.texture_slot.value(), Some(&0));
    assert_eq!(decoded.texture_slot.track(), ribbon.texture_slot.track());
    ribbon.texture_slot = Animatable::Static(0);
    ribbon.node.flags = NodeFlags(0);
    assert!(ribbon.encode_mdl().is_err());
}

#[test]
fn particle_path_is_a_single_260_byte_field() {
    let path = "a".repeat(259);
    let mut node = Node::new("long path", 0).unwrap();
    node.flags = NodeFlags(0x1000);
    let mut emitter = ParticleEmitter::new(node, &path).unwrap();
    emitter.life_span = Animatable::Static(123.0);
    emitter.initial_velocity = Animatable::Static(456.0);
    let bytes = emitter.encode_mdx().unwrap();
    let node_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let start = 4 + node_size + 16;
    assert_eq!(&bytes[start..start + 259], path.as_bytes());
    assert_eq!(bytes[start + 259], 0);
    assert_eq!(&bytes[start + 260..start + 264], &123.0f32.to_le_bytes());
    assert_eq!(&bytes[start + 264..start + 268], &456.0f32.to_le_bytes());
    assert_eq!(
        ParticleEmitter::decode_mdx(&bytes).unwrap().path.text(),
        path
    );
    roundtrip(&emitter);
    assert!(emitter.path.set_text(&"a".repeat(260)).is_err());
}

#[test]
fn typed_emitter_nodes_share_common_mdl_flags() {
    use wc3::model::emitters::{ParticleEmitter2, PopcornEmitter};

    let mut particle = ParticleEmitter2::decode_mdl(
        "ParticleEmitter2 \"typed\" { ObjectId 1, DontInheritRotation, LineEmitter, }",
    )
    .unwrap();
    assert!(particle.node.flags.dont_inherit_rotation());
    assert!(particle.node.flags.line_emitter());
    particle.node.flags.set_camera_anchored(true);
    particle.node.flags.set_unfogged(true);
    roundtrip(&particle);
    let mut popcorn = PopcornEmitter::decode_mdl(
        "ParticleEmitterPopcorn \"typed\" { ObjectId 1, Billboarded, Unfogged, }",
    )
    .unwrap();
    assert!(popcorn.node.flags.billboarded());
    assert!(popcorn.node.flags.unfogged());
    popcorn.node.flags.set_dont_inherit_scaling(true);
    popcorn.node.flags.set_popcorn_scaling(true);
    roundtrip(&popcorn);
}
