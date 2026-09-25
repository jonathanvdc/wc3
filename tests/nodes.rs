use wc3_mdx::Record;
use wc3_mdx::{Bone, Model, Node, NodeFlags};

#[test]
fn bones_and_helpers_round_trip() {
    let mut node = Node::new("Root", 7).unwrap();
    node.set_parent_id(3);
    node.set_raw_flags(0x100);
    let bone = Bone::new(node.clone(), 2, u32::MAX);
    let mut model = Model::new(1800);
    model.set_bones(&[bone]).unwrap();
    model.set_helpers(&[node]).unwrap();
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let bone = &parsed.bones().unwrap()[0];
    assert_eq!(bone.node().name(), "Root");
    assert_eq!(bone.node().object_id(), 7);
    assert_eq!(bone.node().parent_id(), 3);
    assert_eq!(bone.node().raw_flags(), 0x100);
    assert_eq!(bone.geoset_id(), 2);
    assert_eq!(bone.geoset_animation_id(), u32::MAX);
    assert_eq!(parsed.helpers().unwrap()[0].name(), "Root");
}

#[test]
fn node_flags_preserve_unknown_bits() {
    let mut node = Node::new("Flagged", 1).unwrap();
    node.set_raw_flags(0x8000_0100);
    let mut flags = node.flags();
    assert!(flags.contains(NodeFlags::BONE));
    flags.set(NodeFlags::BILLBOARDED, true);
    node.set_flags(flags);
    assert_eq!(node.raw_flags(), 0x8000_0108);
    flags.set(NodeFlags::BONE, false);
    node.set_flags(flags);
    assert_eq!(node.raw_flags(), 0x8000_0008);
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
                let model = Model::decode_latest(&bytes).unwrap();
                for bone in model.bones().unwrap() {
                    assert!(!bone.node().name().is_empty());
                }
            }
        }
    }
}
