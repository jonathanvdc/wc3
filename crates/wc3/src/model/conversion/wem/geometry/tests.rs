use super::*;
use crate::model::semantic::profiles::wem::{
    Appearance, Availability, Geometry, Section, SectionFlags, Wem,
};
use crate::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, GeometryData, Mesh, Model,
    Node, NodeInfluence, Topology, Values,
};
use crate::model::wem::RawDocument;
use std::collections::HashMap;

#[test]
fn polygon_groups_corner_layers_and_node_weights_remain_independent() {
    let mut builder = Model::<Wem>::builder("mesh owner");
    let node = builder.add_node(Node::named("bone")).unwrap();
    let mut appearance = Appearance::default();
    let slot = appearance.add_slot("surface");
    let geometry = Geometry::new(GeometryData {
        positions: Values::owned(vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]),
        topology: Topology::Polygons {
            offsets: Values::owned(vec![0, 4]),
            corners: Values::owned(vec![0, 1, 2, 3]),
        },
        attributes: vec![Attribute {
            semantic: AttributeSemantic::Named("normal".into()),
            domain: AttributeDomain::Corner,
            values: AttributeValues::Vector3(Values::owned(vec![[0., 0., 1.]; 4])),
        }],
        ..Default::default()
    })
    .unwrap();
    let mut mesh = Mesh::<Wem>::new(geometry);
    mesh.geometry.influences = Values::owned(vec![vec![NodeInfluence { node, weight: 1. }]; 4]);
    mesh.geometry
        .add_section(Section {
            name: "surface".into(),
            availability: Availability::from_bits_retain(0x8000_0000),
            slot: Some(slot),
            rigid_node: None,
            selection_group: 3,
            flags: SectionFlags::default(),
            bounds: Default::default(),
            metadata: Default::default(),
        })
        .unwrap();
    mesh.geometry.triangulation = Values::owned(vec![vec![[0, 1, 2], [0, 2, 3]]]);
    let mut document = RawDocument::new();
    let source = write(
        &mut document,
        &mesh,
        &HashMap::from([(node, 0)]),
        &HashMap::from([(slot, 0)]),
    )
    .unwrap();
    let restored = read(&document, &source, &[node], &[slot]).unwrap();
    assert_eq!(restored.positions().len(), 4);
    assert!(restored.positions().is_borrowed());
    assert_eq!(
        restored.geometry.triangulation.get(0).unwrap(),
        vec![[0, 1, 2], [0, 2, 3]]
    );
    assert!(restored.geometry.triangulation.is_borrowed());
    let (_, section) = restored.geometry.sections().next().unwrap();
    assert_eq!(section.availability.bits(), 0x8000_0000);
    assert_eq!(section.slot, Some(slot));
    assert_eq!(restored.geometry.influences.get(0).unwrap()[0].node, node);
    let layer = restored.geometry.layers().next().unwrap();
    let crate::model::semantic::profiles::wem::LayerData::Standard(attribute) = &layer.data else {
        panic!("missing corner layer")
    };
    assert_eq!(attribute.domain, AttributeDomain::Corner);
    assert_eq!(attribute.values.len(), 4);
}
