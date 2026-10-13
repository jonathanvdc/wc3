use super::super::records::Character;
use super::*;

#[test]
fn pruning_preserves_live_indices_and_releases_original_container() {
    let mut raw = Document::<V3>::new().unwrap().into_raw();
    let live = raw.append(&[Character { value: b'L' }]).unwrap();
    let root_id = raw.section_id(raw.root_reference().index).unwrap();
    let mut root = raw
        .array::<DocumentRecord<V1>>(root_id)
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    root.name = Ref::from_section(1, live.index());
    raw.replace(root_id, &[root]).unwrap();
    let orphan = raw.append(&[Character { value: b'O' }]).unwrap();
    let bytes = raw.write().unwrap();
    let mut parsed = RawDocument::read(&bytes).unwrap();
    assert!(parsed.source.is_some());
    assert_eq!(parsed.prune_unreachable().unwrap(), 1);
    assert!(parsed.source.is_none());
    assert_eq!(parsed.sections[live.index() as usize].tag, *b"CHAR");
    assert_eq!(parsed.sections[orphan.index() as usize].tag, *b"HOLE");
    let output = parsed.write().unwrap();
    let read = Document::<V3>::read(&output).unwrap();
    assert_eq!(
        read.string(read.root::<DocumentRecord<V1>>().unwrap().name)
            .unwrap(),
        Some("L")
    );
}
