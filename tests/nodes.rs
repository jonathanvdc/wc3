use wc3_mdx::{Bone, Model, Node};

#[test]
fn bones_and_helpers_round_trip() {
    let mut node = Node::new("Root", 7).unwrap();
    node.set_parent_id(3);
    node.set_flags(0x100);
    let bone = Bone::new(node.clone(), 2, u32::MAX);
    let mut model = Model::new(1800);
    model.set_bones(&[bone]).unwrap();
    model.set_helpers(&[node]).unwrap();
    let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let bone = &parsed.bones().unwrap()[0];
    assert_eq!(bone.node().name(), "Root");
    assert_eq!(bone.node().object_id(), 7);
    assert_eq!(bone.node().parent_id(), 3);
    assert_eq!(bone.node().flags(), 0x100);
    assert_eq!(bone.geoset_id(), 2);
    assert_eq!(bone.geoset_animation_id(), u32::MAX);
    assert_eq!(parsed.helpers().unwrap()[0].name(), "Root");
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
                let model = Model::from_bytes(&bytes).unwrap();
                for bone in model.bones().unwrap() {
                    assert!(!bone.node().name().is_empty());
                }
            }
        }
    }
}
