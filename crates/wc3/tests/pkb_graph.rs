use wc3::model::formats::pkb::{Document, EventIndex, LayerIndex};

struct Object {
    kind: usize,
    fields: Vec<(u16, Vec<u8>)>,
}
fn word(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}
fn array(values: &[u32]) -> Vec<u8> {
    (values.len() as u32)
        .to_le_bytes()
        .into_iter()
        .chain(values.iter().copied().flat_map(u32::to_le_bytes))
        .collect()
}
fn objects() -> Vec<Object> {
    vec![
        Object {
            kind: 0,
            fields: vec![(1, word(2)), (3, word(9))],
        },
        Object {
            kind: 1,
            fields: vec![
                (1, array(&[3, 4])),
                (2, array(&[5])),
                (3, array(&[6, 7])),
                (
                    5,
                    [1u32, 5, 3, 1]
                        .into_iter()
                        .flat_map(u32::to_le_bytes)
                        .collect(),
                ),
            ],
        },
        Object {
            kind: 2,
            fields: vec![],
        },
        Object {
            kind: 2,
            fields: vec![(0, word(8)), (1, word(1))],
        },
        Object {
            kind: 3,
            fields: vec![(0, word(8)), (1, array(&[0])), (2, array(&[1]))],
        },
        Object {
            kind: 4,
            fields: vec![(1, word(u32::MAX)), (2, array(&[0]))],
        },
        Object {
            kind: 4,
            fields: vec![(0, word(9))],
        },
        Object {
            kind: 5,
            fields: vec![],
        },
        Object {
            kind: 6,
            fields: vec![],
        },
    ]
}
fn encode(objects: &[Object]) -> Vec<u8> {
    let strings: &[&[u8]] = &[
        b"CParticleEffect",
        b"CLayerGraphCompileCache",
        b"CLayerGraphCompileCache_EntrySlot",
        b"CLayerGraphCompileCache_LayerSlot",
        b"CLayerGraphCompileCache_EventSlot",
        b"CLayerCompileCache",
        b"CParticleAttributeList",
        b"",
        b"Root",
        b"Signal",
    ];
    let mut bytes = vec![0x11, 0x0b, 0, 0xca, 2, 5, 1, 1];
    for value in [0, objects.len() as u32, 7, 0, 0] {
        bytes.extend(word(value));
    }
    for index in 0..7 {
        bytes.extend(word(index));
        bytes.extend(word(0));
    }
    for object in objects {
        let mut body = vec![0x20];
        body.extend(word(object.kind as u32));
        body.extend((object.fields.len() as u16).to_le_bytes());
        for (field, value) in &object.fields {
            body.extend(field.to_le_bytes());
            body.extend(value);
        }
        bytes.extend(word(body.len() as u32));
        bytes.extend(body);
    }
    let offset = bytes.len() as u32;
    bytes[20..24].copy_from_slice(&offset.to_le_bytes());
    bytes.extend(word(strings.len() as u32));
    for string in strings {
        bytes.push(string.len() as u8);
        bytes.extend(*string);
    }
    bytes
}

#[test]
fn graph_preserves_anonymous_default_entry_and_distinct_object_slot_domains() {
    let bytes = encode(&objects());
    let document = Document::read(&bytes).unwrap();
    let effect = document.effects().next().unwrap().unwrap();
    assert_eq!(effect.graph, 2);
    assert_eq!(effect.attributes, 9);
    let graph = document.layer_graph(effect.graph).unwrap();
    assert_eq!(graph.default_entry, 0);
    assert_eq!(graph.coordinates, [1, 5, 3, 1]);
    assert_eq!(graph.entries[0].name, b"");
    assert_eq!(graph.entries[0].event, EventIndex(0));
    assert_eq!(graph.entries[1].name, b"Root");
    assert_eq!(graph.entries[1].event, EventIndex(1));
    assert_eq!(graph.layers[0].cache, 8);
    assert_eq!(graph.layers[0].parents, [EventIndex(0)]);
    assert_eq!(graph.events[0].parent, None);
    assert_eq!(graph.events[0].targets, [LayerIndex(0)]);
    assert_eq!(graph.events[1].parent, Some(LayerIndex(0)));
    assert_eq!(graph.events[1].name, b"Signal");
}

#[test]
fn invalid_routes_and_ownership_are_rejected_without_guessing_index_domains() {
    for (object, field, payload) in [
        (1, 0, word(2)),
        (3, 1, word(2)),
        (3, 1, word(u32::MAX)),
        (4, 1, array(&[2])),
        (4, 2, array(&[0])),
        (4, 0, word(9)),
        (5, 2, array(&[1])),
        (5, 2, array(&[u32::MAX])),
        (6, 1, word(1)),
        (6, 1, word(u32::MAX - 1)),
        (6, 3, array(&[0])),
    ] {
        let mut native = objects();
        native[object].fields.retain(|(id, _)| *id != field);
        native[object].fields.push((field, payload));
        let bytes = encode(&native);
        let document = Document::read(&bytes).unwrap();
        assert!(
            document.layer_graph(2).is_err(),
            "object {object} field {field}"
        );
        assert_eq!(document.as_bytes(), bytes);
    }
}

#[test]
fn null_links_missing_fields_and_unknown_schemas_do_not_become_empty_graphs() {
    for (object, field, payload) in [
        (0, 1, word(0)),
        (1, 1, array(&[0])),
        (1, 6, word(0)),
        (4, 0, word(0)),
    ] {
        let mut native = objects();
        native[object].fields.retain(|(id, _)| *id != field);
        native[object].fields.push((field, payload));
        let bytes = encode(&native);
        let document = Document::read(&bytes).unwrap();
        assert!(document
            .effect(1)
            .and_then(|effect| document.layer_graph(effect.graph))
            .is_err());
    }
    let mut native = objects();
    native[1].fields.retain(|(id, _)| *id != 5);
    let bytes = encode(&native);
    assert!(Document::read(&bytes).unwrap().layer_graph(2).is_err());
    let mut native = objects();
    native[2].fields.extend([(1, word(0)), (1, word(0))]);
    let bytes = encode(&native);
    assert!(Document::read(&bytes).unwrap().layer_graph(2).is_err());
}
