use super::*;
use bevy::math::Vec3;
use std::cell::Cell;

#[test]
fn paused_views_skip_all_sorting_and_upload_work() {
    let mut cache = ViewOrders::<u32, usize>::default();
    let calls = Cell::new(0);
    let prepare = |cache: &mut ViewOrders<u32, usize>, view, affine| {
        cache.prepare(view, affine, |_| {
            calls.set(calls.get() + 1);
            calls.get()
        })
    };
    assert!(prepare(&mut cache, 1, Affine3A::IDENTITY));
    assert!(prepare(&mut cache, 2, Affine3A::IDENTITY));
    for _ in 0..120 {
        assert!(!prepare(&mut cache, 1, Affine3A::IDENTITY));
        assert!(!prepare(&mut cache, 2, Affine3A::IDENTITY));
    }
    assert_eq!(calls.get(), 2);
    let moved = Affine3A::from_translation(Vec3::X);
    assert!(prepare(&mut cache, 2, moved));
    assert!(!prepare(&mut cache, 1, Affine3A::IDENTITY));
    assert_eq!(calls.get(), 3);
    cache.invalidate();
    assert!(prepare(&mut cache, 1, Affine3A::IDENTITY));
    assert!(prepare(&mut cache, 2, moved));
    assert_eq!(calls.get(), 5);
}

#[test]
fn culled_views_keep_buffers_but_dead_views_release_them() {
    let mut cache = ViewOrders::<u32, usize>::default();
    for view in [1, 2] {
        cache.prepare(view, Affine3A::IDENTITY, |_| view as usize);
    }
    // Both cameras still exist; view2 is temporarily culled and not prepared.
    cache.retain(|view| [1, 2].contains(&view));
    cache.prepare(1, Affine3A::IDENTITY, |_| panic!("visible cache hit"));
    assert_eq!(cache.get(&2), Some(&2));
    assert!(!cache.prepare(2, Affine3A::IDENTITY, |_| panic!(
        "returning culled view hit"
    )));
    cache.retain(|view| view == 2);
    assert!(cache.get(&1).is_none());
    cache.generation = u64::MAX;
    cache.invalidate();
    assert!(cache.get(&2).is_none());
}

#[test]
fn scratch_retains_input_ties_and_reuses_both_allocations() {
    let mut scratch = SortScratch::default();
    for (index, distance) in [(7, 3.), (1, 1.), (9, 3.), (2, 2.)] {
        scratch.push(index, distance);
    }
    scratch.sort(false, |index, order| order.extend([index, index + 100]));
    assert_eq!(scratch.order, [7, 107, 9, 109, 2, 102, 1, 101]);
    let depth_pointer = scratch.depths.as_ptr();
    let order_pointer = scratch.order.as_ptr();
    scratch.clear();
    for (index, distance) in [(9, 3.), (7, 3.), (1, 1.)] {
        scratch.push(index, distance);
    }
    scratch.sort(true, |index, order| order.push(index));
    assert_eq!(scratch.order, [1, 9, 7]);
    assert_eq!(scratch.depths.as_ptr(), depth_pointer);
    assert_eq!(scratch.order.as_ptr(), order_pointer);
}

#[test]
fn changing_sort_policy_invalidates_orders_even_when_geometry_is_unchanged() {
    let mut cache = ViewOrders::<u32, usize>::default();
    cache.set_policy(SortPolicy::Far);
    cache.prepare(1, Affine3A::IDENTITY, |_| 1);
    cache.set_policy(SortPolicy::Far);
    assert!(!cache.prepare(1, Affine3A::IDENTITY, |_| panic!("same policy hit")));
    cache.set_policy(SortPolicy::Near);
    assert!(cache.prepare(1, Affine3A::IDENTITY, |previous| {
        assert_eq!(previous, Some(&1));
        2
    }));
}

#[test]
#[ignore = "opt-in paused-view CPU comparison; excludes actual GPU submission"]
fn benchmark_paused_view_order_cache() {
    use bevy::math::{Quat, Vec3A};
    use std::hint::black_box;
    use std::time::Instant;

    let positions: Vec<_> = (0..8192u32)
        .map(|index| {
            let value = index.wrapping_mul(0x9e37_79b9);
            Vec3A::new(
                (value & 1023) as f32,
                ((value >> 10) & 1023) as f32,
                ((value >> 20) & 1023) as f32,
            )
        })
        .collect();
    let views = [
        (1, Affine3A::IDENTITY),
        (2, Affine3A::from_quat(Quat::from_rotation_y(0.7))),
    ];
    let mut cache = ViewOrders::<u32, usize>::default();
    cache.set_policy(SortPolicy::Far);
    let mut scratch = SortScratch::default();
    let mut prepares = 0;
    let payload_bytes = Cell::new(0);
    let mut prepare_view = |cache: &mut ViewOrders<u32, usize>, view, affine: Affine3A| {
        cache.prepare(view, affine, |_| {
            prepares += 1;
            scratch.clear();
            let forward = -affine.matrix3.z_axis;
            for (index, position) in positions.iter().enumerate() {
                scratch.push(index as u32, (*position - affine.translation).dot(forward));
            }
            scratch.sort(false, |index, order| {
                order.extend([index, index | 0x8000_0000])
            });
            black_box(&scratch.order);
            payload_bytes.set(payload_bytes.get() + scratch.order.len() * size_of::<u32>());
            scratch.order.len()
        });
    };
    for &(view, affine) in &views {
        prepare_view(&mut cache, view, affine);
    }
    let initial_payload = payload_bytes.get();
    let frames = 120;
    let start = Instant::now();
    for _ in 0..frames {
        cache.invalidate(); // Previously every frame rebuilt each view's order.
        for &(view, affine) in &views {
            prepare_view(&mut cache, view, affine);
        }
    }
    let recompute = start.elapsed();
    let recomputed_payload = payload_bytes.get() - initial_payload;
    let start = Instant::now();
    for _ in 0..frames {
        for &(view, affine) in &views {
            prepare_view(&mut cache, view, affine);
        }
    }
    let reuse = start.elapsed();
    assert_eq!(prepares, 2 + frames * 2);
    let cached_payload = payload_bytes.get() - initial_payload - recomputed_payload;
    assert_eq!(recomputed_payload / frames, 131072);
    assert_eq!(cached_payload, 0);
    println!("8192 head+tail particles, 2 paused views, {frames} frames: recompute {:?}; cached {:?} (CPU sorting only, no GPU timing)", recompute, reuse);
    println!("sorted-index payload per paused frame: {} bytes recomputed vs {} cached, after initial upload; natural-order/source uploads excluded", recomputed_payload / frames, cached_payload / frames);
}
