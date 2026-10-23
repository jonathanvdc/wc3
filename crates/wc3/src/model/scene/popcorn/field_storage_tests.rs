use super::*;
use std::array::from_fn;

#[test]
fn flat_storage_has_independent_contiguous_component_planes() {
    let layout = FieldLayout::from_widths(&[1, 3, 0, 4]).unwrap();
    assert_eq!(
        layout.component_ordinals(),
        &[
            [0, usize::MAX, usize::MAX, usize::MAX],
            [1, 2, 3, usize::MAX],
            [usize::MAX; 4],
            [4, 5, 6, 7],
        ]
    );
    let mut fields = FieldPlanes::new(layout, 3).unwrap();
    for slot in 0..3 {
        fields.write(
            1,
            slot,
            [10 + slot as u32, 20 + slot as u32, 30 + slot as u32, 99],
        );
    }
    assert_eq!(&fields.data[3..12], &[10, 11, 12, 20, 21, 22, 30, 31, 32]);
    assert_eq!(fields.read(1, 1), [11, 21, 31, 0]);
    assert_eq!(fields.read(0, 1), [0; 4]);
    assert_eq!(fields.read(3, 1), [0; 4]);
    assert_eq!(fields.plane_stride(), 3);
    assert_eq!(fields.data_mut_ptr(), fields.data.as_mut_ptr());
}

#[test]
fn shared_wider_physical_fields_clear_inactive_words_on_narrow_store() {
    let mut fields = FieldPlanes::new(FieldLayout::from_widths(&[4]).unwrap(), 2).unwrap();
    fields.write(0, 1, [1, 2, 3, 4]);
    fields.write(0, 0, [5, 6, 7, 8]);
    // A Float store supplies normalized four words, even when another layer
    // caused this shared local index to have four physical component planes.
    fields.write(0, 1, [9, 0, 0, 0]);
    assert_eq!(fields.read(0, 1), [9, 0, 0, 0]);
    assert_eq!(fields.read(0, 0), [5, 6, 7, 8]);
}

#[test]
fn empty_zero_capacity_and_out_of_range_slots_keep_original_noop_contract() {
    let empty = FieldPlanes::new(FieldLayout::from_widths(&[]).unwrap(), 12).unwrap();
    assert!(empty.data.is_empty());
    assert!(empty.layout().component_ordinals().is_empty());
    let mut fields = FieldPlanes::new(FieldLayout::from_widths(&[0, 2]).unwrap(), 0).unwrap();
    fields.write(0, 0, [8; 4]);
    fields.write(1, usize::MAX, [9; 4]);
    assert_eq!(fields.read(0, 0), [0; 4]);
    assert_eq!(fields.read(1, usize::MAX), [0; 4]);
    let mut fields = FieldPlanes::new(FieldLayout::from_widths(&[0, 2]).unwrap(), 1).unwrap();
    fields.write(1, 1, [9; 4]);
    fields.write(1, usize::MAX, [10; 4]);
    assert_eq!(fields.read(1, 0), [0; 4]);
    assert_eq!(fields.read(1, 1), [0; 4]);
    fields.write(0, 0, [11; 4]);
    assert_eq!(fields.read(0, 0), [0; 4]);
}

#[test]
#[should_panic]
fn absent_field_index_still_panics_even_for_an_out_of_range_slot() {
    let fields = FieldPlanes::new(FieldLayout::from_widths(&[1]).unwrap(), 0).unwrap();
    let _ = fields.read(1, usize::MAX);
}

#[test]
fn storage_prediction_includes_payload_and_ordinal_metadata_with_checked_arithmetic() {
    let layout = FieldLayout::from_widths(&[1, 3, 0, 4]).unwrap();
    assert_eq!(
        layout.storage_bytes(3),
        Some(8 * 3 * size_of::<u32>() + 4 * size_of::<[usize; 4]>() + size_of::<FieldPlanes>())
    );
    assert!(layout.storage_bytes(usize::MAX).is_none());
    assert!(FieldPlanes::new(layout, usize::MAX).is_none());
    assert!(FieldLayout::from_widths(&[5]).is_none());
}

#[test]
fn graph_layout_unions_layer_local_widths_and_retains_opaque_word_storage() {
    use crate::model::formats::pkb::{Document, EffectCompileOptions};
    let mut program = Document::read(include_bytes!("../../../../tests/fixtures/pkb/sprite.pkb"))
        .unwrap()
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let mut first = graph.layers[0].clone();
    let mut second = first.clone();
    first.fields = first.fields[..2].to_vec().into_boxed_slice();
    first.fields[0].value_type = Some(ValueType::Float);
    first.fields[1].value_type = None;
    second.fields = second.fields[..3].to_vec().into_boxed_slice();
    second.fields[0].value_type = Some(ValueType::Float4);
    second.fields[1].value_type = Some(ValueType::Float2);
    second.fields[2].value_type = None;
    graph.layers = vec![first, second].into_boxed_slice();
    let layout = FieldLayout::for_graph(graph).unwrap();
    assert_eq!(
        layout.component_ordinals(),
        &[
            [0, 1, 2, 3],
            [4, 5, usize::MAX, usize::MAX],
            [6, usize::MAX, usize::MAX, usize::MAX],
        ]
    );
}

#[test]
fn complete_read_write_trajectory_matches_original_separate_plane_contract() {
    for capacity in [0, 1, 7, 32] {
        let widths = [0, 1, 2, 3, 4];
        let mut old: Vec<[Vec<u32>; 4]> = widths
            .iter()
            .map(|&width| {
                from_fn(|component| {
                    if component < width {
                        vec![0; capacity]
                    } else {
                        Vec::new()
                    }
                })
            })
            .collect();
        let mut flat =
            FieldPlanes::new(FieldLayout::from_widths(&widths).unwrap(), capacity).unwrap();
        for tick in 0..97u32 {
            for (field, planes) in old.iter_mut().enumerate() {
                for slot in 0..=capacity {
                    let words = [
                        tick.wrapping_mul(12345),
                        (field as u32) ^ tick,
                        slot as u32,
                        u32::MAX - tick,
                    ];
                    for (plane, word) in planes.iter_mut().zip(words) {
                        if let Some(value) = plane.get_mut(slot) {
                            *value = word;
                        }
                    }
                    flat.write(field, slot, words);
                }
                for slot in 0..=capacity {
                    let expected =
                        from_fn(|component| planes[component].get(slot).copied().unwrap_or(0));
                    assert_eq!(flat.read(field, slot), expected);
                }
            }
        }
    }
}

#[test]
fn checked_component_borrows_expose_only_the_selected_full_plane() {
    let mut fields = FieldPlanes::new(FieldLayout::from_widths(&[1, 3, 0]).unwrap(), 3).unwrap();
    for slot in 0..3 {
        fields.write(
            1,
            slot,
            [10 + slot as u32, 20 + slot as u32, 30 + slot as u32, 0],
        );
    }
    assert_eq!(fields.component(1, 0), Some([10, 11, 12].as_slice()));
    assert_eq!(fields.component(1, 2), Some([30, 31, 32].as_slice()));
    assert_eq!(fields.component(1, 3), None);
    assert_eq!(fields.component(2, 0), None);
    assert_eq!(fields.component(3, 0), None);
    assert_eq!(fields.component(0, usize::MAX), None);
    let empty = FieldPlanes::new(FieldLayout::from_widths(&[1]).unwrap(), 0).unwrap();
    assert_eq!(empty.component(0, 0), Some([].as_slice()));
}
