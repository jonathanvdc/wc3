use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::profiles::{Mdlx, Narrows, Universal};
use wc3::model::ValueGroups;
use wc3::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivitySets, GeometryData,
    GeometryEdit, GeometryInfluences, GeometrySchema, Influence, Topology,
};

#[test]
fn general_geometry_projects_only_when_its_full_domain_is_supported() {
    let mut general = GeometryData::triangles(vec![[0.; 3]], vec![[0, 0, 0]]).unwrap();
    assert!(<Mdlx as Narrows<Universal>>::geometry(general.clone()).is_ok());
    general
        .attributes
        .insert(Attribute {
            semantic: AttributeSemantic::Named("custom".into()),
            domain: AttributeDomain::Vertex,
            values: AttributeValues::Unsigned(vec![23].into_zero_vec()),
        })
        .unwrap();
    assert!(<Mdlx as Narrows<Universal>>::geometry(general).is_err());
    let mut general = GeometryData::triangles(vec![[0.; 3]], vec![[0, 0, 0]]).unwrap();
    general.influences = GeometryInfluences::Variable(
        vec![vec![Influence {
            joint: 2,
            weight: 1.,
        }]]
        .into(),
    );
    assert!(<Mdlx as Narrows<Universal>>::geometry(general).is_err());
}

#[test]
fn remapping_keeps_each_influence_domain_and_custom_attribute_correspondence() {
    use std::mem::discriminant;
    use wc3::model::VertexRemap;
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let weighted = Influence {
        joint: 2,
        weight: 1.,
    };
    let zero = Influence {
        joint: 9,
        weight: 0.,
    };
    let variable = [vec![weighted], vec![weighted], vec![weighted]];
    let three = [[weighted, zero, zero]; 3];
    let four = [[weighted, zero, zero, zero]; 3];
    let groups = [vec![2, 7], vec![7, 2], vec![9, 7, 2]];
    let custom = [10u32, 20, 30];
    for influences in [
        GeometryInfluences::None,
        GeometryInfluences::Variable(ValueGroups::from_rows(variable)),
        GeometryInfluences::Three(ZeroVec::alloc_from_slice(&three)),
        GeometryInfluences::Four(ZeroVec::alloc_from_slice(&four)),
        GeometryInfluences::MatrixGroups(ValueGroups::from_rows(groups)),
    ] {
        let data = GeometryData {
            positions: ZeroVec::alloc_from_slice(&positions),
            connectivity: ConnectivitySets::new(
                Topology::Triangles(vec![[0, 1, 2]].into_zero_vec()),
                Vec::new(),
            ),
            attributes: vec![Attribute {
                semantic: AttributeSemantic::Named("custom".into()),
                domain: AttributeDomain::Vertex,
                values: AttributeValues::Unsigned(ZeroVec::alloc_from_slice(&custom)),
            }]
            .try_into()
            .unwrap(),
            influences,
            ..Default::default()
        };
        let mapped = data
            .edited(GeometryEdit {
                remap: Some(VertexRemap {
                    new_to_old: vec![2, 0, 1, 0],
                    topology: Topology::Triangles(vec![[3, 2, 0]].into_zero_vec()),
                }),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            discriminant(&mapped.influences),
            discriminant(&data.influences)
        );
        for (new, old) in [2usize, 0, 1, 0].into_iter().enumerate() {
            assert_eq!(
                mapped.influences.rows().joint_weights(new),
                data.influences.rows().joint_weights(old)
            );
        }
        let AttributeValues::Unsigned(values) = &mapped.attributes[0].values else {
            unreachable!()
        };
        assert_eq!(values.to_vec(), vec![30, 10, 20, 10]);
    }
}
