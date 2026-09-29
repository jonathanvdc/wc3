use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;
use wc3::model::scene::{Bone, Node, NodeFlags};
use wc3::model::Model;

#[test]
fn bones_and_helpers_round_trip() {
    let mut node = Node::new("Root", 7).unwrap();
    node.parent_id = 3;
    node.flags = NodeFlags(0x100);
    let bone = Bone::new(node.clone(), 2, u32::MAX);
    let mut model = Model::<wc3::model::V1800>::new();
    model.set_bones(&[bone]);
    model.set_helpers(&[node]);
    let parsed = Model::<wc3::model::V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let bone = &parsed.bones()[0];
    assert_eq!(bone.node.name.text(), "Root");
    assert_eq!(bone.node.object_id, 7);
    assert_eq!(bone.node.parent_id, 3);
    assert_eq!(bone.node.flags.bits(), 0x100);
    assert_eq!(bone.geoset_id, 2);
    assert_eq!(bone.geoset_animation_id, u32::MAX);
    assert_eq!(parsed.helpers()[0].name.text(), "Root");
}

#[test]
fn node_flags_preserve_unknown_bits() {
    let mut node = Node::new("Flagged", 1).unwrap();
    node.flags = NodeFlags(0x8000_0100);
    let mut flags = node.flags;
    assert!(flags.bone());
    flags.set_billboarded(true);
    node.flags = flags;
    assert_eq!(node.flags.bits(), 0x8000_0108);
    flags.set_bone(false);
    node.flags = flags;
    assert_eq!(node.flags.bits(), 0x8000_0008);
}

#[test]
fn local_bones_are_bounded_when_available() {
    let Ok(directory) = std::env::var("WC3_MDX_FIXTURES") else {
        return;
    };
    let mut pending = vec![std::path::PathBuf::from(directory)];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "mdx") {
                let bytes = std::fs::read(&path).unwrap();
                let model = Model::<wc3::model::V1800>::decode_mdx(&bytes).unwrap();
                for bone in model.bones() {
                    assert!(!bone.node.name.text().is_empty());
                }
            }
        }
    }
}

#[test]
fn typed_node_flags_preserve_the_complete_word_and_node() {
    use wc3::model::animation::{AnimationTrack, NodeTranslation};
    use wc3::model::emitters::{Particle2Flags, ParticleEmitterFlags, PopcornFlags};
    use wc3::model::scene::NodeTrack;

    let bits = 0x8002_1001;
    let mut original = Node::new("Typed", 9).unwrap();
    original.parent_id = 4;
    original.flags = NodeFlags(bits);
    original.tracks.push(NodeTrack::Translation(
        AnimationTrack::<NodeTranslation>::linear(vec![], None).unwrap(),
    ));
    let bytes = original.encode_mdx().unwrap();
    let mut particle = original.clone().cast_flags::<Particle2Flags>();
    assert!(particle.flags.line_emitter());
    assert!(!particle.flags.unfogged());
    assert!(particle.flags.dont_inherit_translation());
    assert_eq!(particle.encode_mdx().unwrap(), bytes);
    assert_eq!(
        Node::<Particle2Flags>::decode_mdx(&bytes).unwrap(),
        particle
    );
    particle.flags.set_billboarded(true);
    assert_eq!(particle.flags.bits(), bits | 8);
    particle.flags.set_billboarded(false);
    let popcorn = particle.cast_flags::<PopcornFlags>();
    assert!(popcorn.flags.unfogged());
    assert_eq!(popcorn.encode_mdx().unwrap(), bytes);
    assert_eq!(Node::<PopcornFlags>::decode_mdx(&bytes).unwrap(), popcorn);
    let classic = popcorn.cast_flags::<ParticleEmitterFlags>();
    assert_eq!(classic.flags.bits(), bits);
    assert_eq!(classic.cast_flags::<NodeFlags>(), original);
    assert_eq!(Particle2Flags::from_bits(bits).bits(), bits);
    assert_eq!(PopcornFlags::from_bits(bits).bits(), bits);
    assert_eq!(ParticleEmitterFlags::from_bits(bits).bits(), bits);
}
