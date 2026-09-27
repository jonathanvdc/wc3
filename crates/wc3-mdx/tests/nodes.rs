use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::scene::{Bone, Node, NodeFlags};
use wc3_mdx::Model;

#[test]
fn bones_and_helpers_round_trip() {
    let mut node = Node::new("Root", 7).unwrap();
    node.set_parent_id(3);
    node.set_flags(NodeFlags(0x100));
    let bone = Bone::new(node.clone(), 2, u32::MAX);
    let mut model = Model::<wc3_mdx::V1800>::new();
    model.set_bones(&[bone]);
    model.set_helpers(&[node]);
    let parsed = Model::<wc3_mdx::V1800>::decode(&model.encode().unwrap()).unwrap();
    let bone = &parsed.bones()[0];
    assert_eq!(bone.node().name(), "Root");
    assert_eq!(bone.node().object_id(), 7);
    assert_eq!(bone.node().parent_id(), 3);
    assert_eq!(bone.node().flags().bits(), 0x100);
    assert_eq!(bone.geoset_id(), 2);
    assert_eq!(bone.geoset_animation_id(), u32::MAX);
    assert_eq!(parsed.helpers()[0].name(), "Root");
}

#[test]
fn node_flags_preserve_unknown_bits() {
    let mut node = Node::new("Flagged", 1).unwrap();
    node.set_flags(NodeFlags(0x8000_0100));
    let mut flags = node.flags();
    assert!(flags.bone());
    flags.set_billboarded(true);
    node.set_flags(flags);
    assert_eq!(node.flags().bits(), 0x8000_0108);
    flags.set_bone(false);
    node.set_flags(flags);
    assert_eq!(node.flags().bits(), 0x8000_0008);
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
                let model = Model::<wc3_mdx::V1800>::decode(&bytes).unwrap();
                for bone in model.bones() {
                    assert!(!bone.node().name().is_empty());
                }
            }
        }
    }
}
