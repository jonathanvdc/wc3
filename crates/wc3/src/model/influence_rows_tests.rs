use super::*;
use crate::model::common::IntoPackedArray;

fn contribution(joint: u32, weight: f32) -> Influence {
    Influence { joint, weight }
}

#[test]
fn packed_rows_borrow_unaligned_bytes_and_preserve_float_bits() {
    let source = [
        contribution(8, -0.),
        contribution(8, f32::from_bits(0x7fc01234)),
    ]
    .into_packed_array();
    let mut unaligned = vec![0];
    unaligned.extend_from_slice(source.as_bytes());
    let row = JointWeights::packed(PackedArray::parse_bytes(&unaligned[1..]).unwrap());
    let JointWeightStorage::Packed(values) = &row.storage else {
        unreachable!()
    };
    assert_eq!(values.as_bytes().as_ptr(), unaligned[1..].as_ptr());
    assert!(!values.is_owned());
    assert_eq!(row.len(), 2);
    assert_eq!(row.get(0).unwrap().weight.to_bits(), (-0f32).to_bits());
    assert_eq!(row.get(1).unwrap().weight.to_bits(), 0x7fc01234);
    assert_eq!(row.get(1).unwrap().joint, 8);
    assert!(row.get(2).is_none());
    assert!(row.get(usize::MAX).is_none());
    assert_eq!(row.to_vec()[1].weight.to_bits(), 0x7fc01234);
}

#[test]
fn fixed_rows_preserve_zero_slots_and_compare_by_logical_values() {
    let values = [
        contribution(3, 0.5),
        contribution(8, 0.),
        contribution(3, 0.5),
    ];
    let three = JointWeights::three(values);
    assert_eq!(three, JointWeights::packed(values.into_packed_array()));
    assert_eq!(three.to_vec(), values);
    assert_eq!(three.len(), 3);
    assert!(three.get(3).is_none());
    let values = [values[0], values[1], values[2], contribution(9, 0.)];
    let four = JointWeights::four(values);
    assert_eq!(four, JointWeights::packed(values.into_packed_array()));
    assert_eq!(four.to_vec(), values);
    assert_eq!(four.len(), 4);
    assert!(four.get(4).is_none());
    assert_eq!(
        VertexInfluences::Joints(four).weights().collect::<Vec<_>>(),
        [0.5, 0., 0.5, 0.]
    );
}

#[test]
fn matrix_groups_weight_each_authored_member_without_merging_duplicates() {
    let source = [4u32, 9, 4].into_packed_array();
    let row = JointWeights::matrix_groups(source.get_subslice(0..3).unwrap());
    let JointWeightStorage::MatrixGroups {
        joints: values,
        weight,
    } = &row.storage
    else {
        unreachable!()
    };
    assert_eq!(values.as_bytes().as_ptr(), source.as_bytes().as_ptr());
    assert_eq!(*weight, 1. / 3.);
    assert_eq!(
        row.to_vec(),
        [
            contribution(4, 1. / 3.),
            contribution(9, 1. / 3.),
            contribution(4, 1. / 3.)
        ]
    );
    let empty = JointWeights::matrix_groups(PackedArray::new());
    assert!(empty.is_empty());
    assert!(empty.get(0).is_none());
    assert_eq!(empty, JointWeights::default());
}

#[test]
fn iterators_track_exact_remaining_length_under_mixed_direction_and_skips() {
    let row = JointWeights::four([
        contribution(0, 0.),
        contribution(1, 0.2),
        contribution(2, 0.3),
        contribution(3, 0.5),
    ]);
    let mut borrowed = row.iter();
    assert_eq!(borrowed.len(), 4);
    assert_eq!(borrowed.next_back().unwrap().joint, 3);
    assert_eq!(borrowed.next().unwrap().joint, 0);
    assert_eq!(borrowed.nth(1).unwrap().joint, 2);
    assert_eq!(borrowed.len(), 0);
    assert_eq!(borrowed.next(), None);
    assert_eq!(borrowed.next_back(), None);
    drop(borrowed);

    let mut owned = row.into_iter();
    assert_eq!(owned.size_hint(), (4, Some(4)));
    assert_eq!(owned.next().unwrap().joint, 0);
    assert_eq!(owned.nth_back(1).unwrap().joint, 2);
    assert_eq!(owned.len(), 1);
    assert_eq!(owned.next_back().unwrap().joint, 1);
    assert_eq!(owned.len(), 0);
    assert_eq!(owned.next(), None);
    assert_eq!(owned.next_back(), None);
    assert_eq!(owned.nth(usize::MAX), None);
}
