use super::*;
use bevy::math::Vec3;
use wc3::model::ModelBuilder;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
fn catalog(records: &[(u32, i32, [f32; 3])]) -> CharacterCatalog {
    let mut slot_values = SlotValues::default();
    let mut builder: ModelBuilder<'static> =
        ModelBuilder::new(ModelMetadata::M2(Default::default()));
    let bones = records
        .iter()
        .enumerate()
        .map(|(i, (crc, key, pivot))| {
            let node = builder.add_named_node(format!("bone{i}")).unwrap();
            BoneIdentity {
                node,
                name_crc: *crc,
                key_bone: *key,
                pivot: Vec3::from_array(*pivot),
                name: format!("bone{i}"),
            }
        })
        .collect();
    CharacterCatalog {
        bones,
        ..SlotConstruct::construct(&mut slot_values)
    }
}
#[test]
fn matches_native_identities_without_relying_on_joint_order() {
    let parent = catalog(&[(20, -1, [0.; 3]), (10, -1, [1.; 3]), (30, 4, [2.; 3])]);
    let child = catalog(&[(10, -1, [1.; 3]), (99, 4, [2.; 3]), (20, -1, [0.; 3])]);
    let required: Vec<_> = child.bones.iter().map(|bone| bone.node).collect();
    let matches = parent.match_joints(&child, &required).unwrap();
    assert_eq!(
        matches,
        vec![
            (required[0], parent.bones[1].node),
            (required[1], parent.bones[2].node),
            (required[2], parent.bones[0].node)
        ]
    );
}
#[test]
fn repeated_crc_requires_a_unique_pivot_match() {
    let parent = catalog(&[
        (0, -1, [1., 0., 0.]),
        (0, -1, [2., 0., 0.]),
        (40, -1, [3., 0., 0.]),
        (40, -1, [4., 0., 0.]),
    ]);
    let child = catalog(&[(0, -1, [2., 0., 0.]), (40, -1, [4., 0., 0.])]);
    let required: Vec<_> = child.bones.iter().map(|bone| bone.node).collect();
    assert_eq!(
        parent.match_joints(&child, &required).unwrap()[0].1,
        parent.bones[1].node
    );
    assert_eq!(
        parent.match_joints(&child, &required).unwrap()[1].1,
        parent.bones[3].node
    );
    let ambiguous = catalog(&[(40, -1, [4., 0., 0.]), (40, -1, [4., 0., 0.])]);
    assert_eq!(
        ambiguous.match_joints(&child, &required[1..]),
        Err(JointMatchError::Ambiguous(required[1]))
    );
}
#[test]
fn missing_identities_and_duplicate_requests_are_explicit_errors() {
    let parent = catalog(&[(10, -1, [0.; 3])]);
    let child = catalog(&[(20, -1, [0.; 3])]);
    let node = child.bones[0].node;
    assert_eq!(
        parent.match_joints(&child, &[node]),
        Err(JointMatchError::Unmatched(node))
    );
    let matching = catalog(&[(10, -1, [0.; 3])]);
    let node = matching.bones[0].node;
    assert_eq!(
        parent.match_joints(&matching, &[node, node]),
        Err(JointMatchError::DuplicateChild(node))
    );
}
