use super::*;

#[test]
fn wrap_preserves_live_order_and_uploads_only_birth_ranges() {
    let mut ring = RecordRing::default();
    for i in 0..4u32 {
        ring.push(i, 4);
    }
    ring.pop_front();
    ring.pop_front();
    let previous = ring.cursor();
    ring.push(4, 4);
    assert_eq!(ring.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
    assert_eq!(ring.index(0), 2);
    assert_eq!(ring.upload_ranges(Some(previous)), [0..1, 0..0]);
    ring.pop_front();
    ring.pop_front();
    let previous = ring.cursor();
    for i in 5..8 {
        ring.push(i, 4);
    }
    assert_eq!(ring.upload_ranges(Some(previous)), [1..4, 0..0]);
    ring.pop_front();
    let previous = ring.cursor();
    ring.push(8, 4);
    assert_eq!(ring.upload_ranges(Some(previous)), [0..1, 0..0]);
}

#[test]
fn growth_rebases_wrapped_records_and_invalidates_the_gpu_cursor() {
    let mut ring = RecordRing::default();
    for i in 0..BLOCK_RECORDS as u32 {
        ring.push(i, 128);
    }
    for _ in 0..60 {
        ring.pop_front();
    }
    for i in 64..124 {
        ring.push(i, 128);
    }
    let previous = ring.cursor();
    ring.push(124, 128);
    assert_eq!(ring.capacity(), 128);
    assert_eq!(ring.index(0), 0);
    assert_eq!(
        ring.iter().copied().collect::<Vec<_>>(),
        (60..125).collect::<Vec<_>>()
    );
    assert_eq!(ring.upload_ranges(Some(previous)), [0..128, 0..0]);
}

#[test]
fn extraction_snapshots_copy_only_written_blocks_and_keep_old_values() {
    let mut ring = RecordRing::default();
    for i in 0..128u32 {
        ring.push(i, 128);
    }
    for _ in 0..65 {
        ring.pop_front();
    }
    let previous = ring.clone();
    ring.push(128, 128);
    assert!(!Arc::ptr_eq(&ring.blocks[0], &previous.blocks[0]));
    assert!(Arc::ptr_eq(&ring.blocks[1], &previous.blocks[1]));
    assert_eq!(previous[0], 0);
    assert_eq!(ring[0], 128);
    assert_eq!(ring.upload_ranges(Some(previous.cursor())), [0..1, 0..0]);
}

#[test]
fn upload_ranges_reconstruct_live_records_after_skipped_frames_and_growth() {
    use std::collections::VecDeque;
    let mut ring = RecordRing::default();
    let mut expected = VecDeque::new();
    let mut gpu = Vec::new();
    let mut cursor = None;
    let mut scratch = Vec::new();
    let mut random = 1u32;
    let mut next = 1u32;
    for frame in 0..1000 {
        random = random.wrapping_mul(1664525).wrapping_add(1013904223);
        let retire = (random as usize % 9).min(expected.len());
        for _ in 0..retire {
            ring.pop_front();
            expected.pop_front();
        }
        let births = ((random >> 8) as usize % 13).min(128 - expected.len());
        for _ in 0..births {
            ring.push(next, 128);
            expected.push_back(next);
            next += 1;
        }
        assert_eq!(
            ring.iter().copied().collect::<Vec<_>>(),
            expected.iter().copied().collect::<Vec<_>>()
        );
        if frame % 7 != 0 {
            continue;
        }
        gpu.resize(ring.capacity(), 0);
        for range in ring.upload_ranges(cursor) {
            ring.copy_range(range.clone(), &mut scratch);
            gpu[range].copy_from_slice(&scratch);
        }
        cursor = Some(ring.cursor());
        for (i, &record) in expected.iter().enumerate() {
            assert_eq!(gpu[ring.index(i) as usize], record);
        }
    }
}

#[test]
fn skipped_frames_and_full_laps_upload_current_contents() {
    let mut ring = RecordRing::default();
    for i in 0..4u32 {
        ring.push(i, 4);
    }
    let previous = ring.cursor();
    for i in 4..14 {
        ring.pop_front();
        ring.push(i, 4);
    }
    assert_eq!(ring.upload_ranges(Some(previous)), [0..4, 0..0]);
    let previous = ring.cursor();
    ring.pop_front();
    ring.push(14, 4);
    ring.pop_front();
    ring.push(15, 4);
    ring.pop_front();
    ring.push(16, 4);
    assert_eq!(ring.upload_ranges(Some(previous)), [2..4, 0..1]);
    let mut scratch = Vec::new();
    ring.copy_range(0..4, &mut scratch);
    assert_eq!(scratch, vec![16, 13, 14, 15]);
    let previous = ring.cursor();
    ring.pop_front();
    assert_eq!(ring.upload_ranges(Some(previous)), [0..0, 0..0]);
}
