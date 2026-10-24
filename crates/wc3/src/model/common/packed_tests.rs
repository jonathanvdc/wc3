use super::*;
use crate::model::common::IntoOwned;

#[test]
fn constructors_distinguish_static_borrows_from_owned_native_storage() {
    static BYTES: [u8; 4] = [3, 0, 5, 0];
    let borrowed = PackedArray::<u16>::parse_bytes(&BYTES).unwrap();
    assert!(!borrowed.is_owned());
    assert!(matches!(&borrowed.storage, Storage::Borrowed(_)));
    assert_eq!(borrowed.as_bytes().as_ptr(), BYTES.as_ptr());

    let native = ZeroVec::<u16>::alloc_from_slice(&[3, 5]);
    let pointer = native.as_bytes().as_ptr();
    let owned = PackedArray::<u16>::from_storage(native);
    assert!(matches!(&owned.storage, Storage::Owned(_)));
    assert_eq!(owned.as_bytes().as_ptr(), pointer);

    let detached = borrowed.into_owned();
    assert!(matches!(&detached.storage, Storage::Owned(_)));
    assert_ne!(detached.as_bytes().as_ptr(), BYTES.as_ptr());
}

#[test]
fn unaligned_vectors_borrow_and_detach_without_layout_assumptions() {
    let raw: Vec<_> = [1f32, -2., 3., 4., 5., -6.]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let mut unaligned = vec![0u8];
    unaligned.extend_from_slice(&raw);
    let values = PackedArray::<Vec3>::parse_bytes(&unaligned[1..]).unwrap();
    assert_eq!(values.as_bytes().as_ptr(), unaligned[1..].as_ptr());
    assert_eq!(values.get(1), Some(Vec3::new(4., 5., -6.)));
    let tail = values.get_subslice(1..2).unwrap();
    assert_eq!(tail.as_bytes().as_ptr(), unaligned[13..].as_ptr());
    assert!(values.get_subslice(1..3).is_none());
    assert!(values.get(2).is_none());
    let owned = values.clone().into_owned();
    assert_eq!(owned, values);
    assert_ne!(owned.as_bytes().as_ptr(), values.as_bytes().as_ptr());
    let pointer = owned.as_bytes().as_ptr();
    let moved = IntoOwned::into_owned(owned);
    assert_eq!(moved.as_bytes().as_ptr(), pointer);
    assert_eq!(moved.as_borrowed().as_bytes().as_ptr(), pointer);
    assert!(PackedArray::<Vec3>::parse_bytes(&raw[..23]).is_err());
}

#[test]
fn quaternion_storage_preserves_authored_components_and_float_bits() {
    let bits = [0x80000000, 0x7fc01234, 0x3f800000, 0x40000000];
    let bytes: Vec<_> = bits.into_iter().flat_map(u32::to_le_bytes).collect();
    let values = PackedArray::<Quat>::parse_bytes(&bytes).unwrap();
    assert_eq!(values.get(0).unwrap().to_array().map(f32::to_bits), bits);
    let encoded: PackedArray<Quat> = values.iter().collect();
    assert_eq!(encoded.as_bytes(), bytes);
    let unnormalized = Quat::from_xyzw(0., 0., 0., 2.);
    assert_eq!(
        [unnormalized].into_packed_array().get(0),
        Some(unnormalized)
    );
}

#[test]
fn edits_copy_borrowed_bytes_once_and_reuse_owned_storage() {
    let source = [Vec3::ONE, Vec3::ZERO].into_packed_array();
    let mut edited = source.as_borrowed();
    assert!(!edited.is_owned());
    assert_eq!(edited.set(2, Vec3::X), None);
    assert!(!edited.is_owned());
    assert_eq!(edited.set(0, Vec3::Y), Some(Vec3::ONE));
    assert_eq!(source.first(), Some(Vec3::ONE));
    let pointer = edited.as_bytes().as_ptr();
    edited.for_each_mut(|v| *v *= 2.);
    assert_eq!(edited.as_bytes().as_ptr(), pointer);
    assert_eq!(edited.first(), Some(Vec3::Y * 2.));
}

#[test]
fn owned_adoption_cloning_and_detaching_retain_encoded_storage() {
    let native: ZeroVec<u32> = [7, 11, 13].into_iter().collect();
    let pointer = native.as_bytes().as_ptr();
    let values = PackedArray::<u32>::from_storage(native);
    assert!(values.is_owned());
    assert_eq!(values.as_bytes().as_ptr(), pointer);
    let cloned = values.clone();
    assert_eq!(cloned.as_bytes().as_ptr(), pointer);
    let detached = IntoOwned::into_owned(cloned);
    assert_eq!(detached.as_bytes().as_ptr(), pointer);
    drop(values);
    let native = detached.into_storage();
    assert_eq!(native.as_bytes().as_ptr(), pointer);
    assert_eq!(native.to_vec(), [7, 11, 13]);
}

#[test]
fn exclusive_native_extraction_copies_only_shared_owned_storage() {
    let values = [3u16, 5, 8].into_packed_array();
    let pointer = values.as_bytes().as_ptr();
    let extracted = values.clone().into_storage();
    assert_ne!(extracted.as_bytes().as_ptr(), pointer);
    assert_eq!(extracted.as_bytes(), values.as_bytes());
    assert_eq!(values.into_storage().as_bytes().as_ptr(), pointer);

    let bytes = [3u16, 5, 8]
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    let borrowed = PackedArray::<u16>::parse_bytes(&bytes).unwrap();
    assert_eq!(borrowed.clone().as_bytes().as_ptr(), bytes.as_ptr());
    let native = borrowed.into_storage();
    assert!(!native.is_owned());
    assert_eq!(native.as_bytes().as_ptr(), bytes.as_ptr());
}

#[test]
fn shared_edits_detach_once_and_unique_edits_reuse_storage() {
    let original = [2u32, 4, 6].into_packed_array();
    let pointer = original.as_bytes().as_ptr();
    let mut single_edit = original.clone();
    assert_eq!(single_edit.set(3, 9), None);
    assert_eq!(single_edit.as_bytes().as_ptr(), pointer);
    assert_eq!(single_edit.set(1, 8), Some(4));
    let edited_pointer = single_edit.as_bytes().as_ptr();
    assert_ne!(edited_pointer, pointer);
    assert_eq!(single_edit.set(2, 10), Some(6));
    assert_eq!(single_edit.as_bytes().as_ptr(), edited_pointer);
    assert_eq!(single_edit.to_vec(), [2, 8, 10]);
    assert_eq!(original.to_vec(), [2, 4, 6]);

    let mut bulk_edit = original.clone();
    bulk_edit.for_each_mut(|value| *value += 1);
    let edited_pointer = bulk_edit.as_bytes().as_ptr();
    assert_ne!(edited_pointer, pointer);
    assert_eq!(bulk_edit.to_vec(), [3, 5, 7]);
    bulk_edit.for_each_mut(|value| *value *= 2);
    assert_eq!(bulk_edit.as_bytes().as_ptr(), edited_pointer);
    assert_eq!(bulk_edit.to_vec(), [6, 10, 14]);
    drop(single_edit);
    drop(bulk_edit);
    let mut unique = original;
    unique.for_each_mut(|value| *value *= 3);
    assert_eq!(unique.as_bytes().as_ptr(), pointer);
    assert_eq!(unique.to_vec(), [6, 12, 18]);
}

#[test]
fn shared_float_storage_preserves_bits_through_detachment_and_extraction() {
    let bits = [0x80000000, 0x7fc01234, 0x3f800000, 0x40000000];
    let bytes: Vec<_> = bits.into_iter().flat_map(u32::to_le_bytes).collect();
    let values = PackedArray::<Quat>::parse_bytes(&bytes)
        .unwrap()
        .into_owned();
    let pointer = values.as_bytes().as_ptr();
    let alias = values.clone().into_owned();
    assert_eq!(alias.as_bytes().as_ptr(), pointer);
    assert_eq!(alias.as_bytes(), bytes);
    let extracted = alias.into_storage();
    assert_eq!(extracted.as_bytes(), bytes);
    assert_ne!(extracted.as_bytes().as_ptr(), pointer);
    assert_eq!(values.as_bytes(), bytes);
}

#[test]
fn panicking_bulk_edit_leaves_shared_aliases_unchanged() {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let original = [f32::from_bits(0x7fc01234), -0., 1.].into_packed_array();
    let alias = original.clone();
    let bytes = original.as_bytes().to_vec();
    let mut edited = original.clone();
    let mut visits = 0;
    let result = catch_unwind(AssertUnwindSafe(|| {
        edited.for_each_mut(|value| {
            visits += 1;
            assert_ne!(visits, 2, "stop after the first committed edit");
            *value = 2.;
        });
    }));
    assert!(result.is_err());
    assert_eq!(original.as_bytes(), bytes);
    assert_eq!(alias.as_bytes(), bytes);
    assert_ne!(edited.as_bytes().as_ptr(), original.as_bytes().as_ptr());
    assert_eq!(edited.first(), Some(2.));
    assert_eq!(edited.get(1).unwrap().to_bits(), (-0f32).to_bits());
    assert_eq!(edited.last(), Some(1.));
}

#[test]
fn empty_storage_stays_borrowed_through_clone_ownership_and_edits() {
    const EMPTY: PackedArray<'static, u32> = PackedArray::new();
    let mut empty = EMPTY.clone().into_owned();
    assert!(matches!(empty.storage, Storage::Borrowed(_)));
    assert_eq!(empty.set(0, 1), None);
    empty.for_each_mut(|_| panic!("empty arrays have no values"));
    assert!(matches!(empty.storage, Storage::Borrowed(_)));
    assert!(!empty.into_storage().is_owned());
    let native = ZeroVec::<u32>::new_owned(Vec::with_capacity(4));
    let empty = PackedArray::<u32>::from_storage(native);
    assert!(matches!(empty.storage, Storage::Borrowed(_)));
    assert!(empty.is_empty());
}

#[test]
fn static_source_bytes_remain_borrowed_until_detached() {
    static BYTES: [u8; 4] = 7u32.to_le_bytes();
    let borrowed = PackedArray::<u32>::parse_bytes(&BYTES).unwrap();
    assert!(!borrowed.is_owned());
    assert_eq!(borrowed.storage().as_bytes().as_ptr(), BYTES.as_ptr());
    let native = borrowed.clone().into_storage();
    assert!(!native.is_owned());
    assert_eq!(native.as_bytes().as_ptr(), BYTES.as_ptr());
    let owned = borrowed.into_owned();
    assert!(owned.is_owned());
    assert_ne!(owned.as_bytes().as_ptr(), BYTES.as_ptr());
    assert_eq!(owned.first(), Some(7));
}

#[test]
fn owned_subslices_borrow_and_detach_independently() {
    let detached = {
        let original = [2u32, 4, 6].into_packed_array();
        let mut tail = original.get_subslice(1..3).unwrap();
        assert!(!tail.is_owned());
        assert_eq!(tail.as_bytes().as_ptr(), original.as_bytes()[4..].as_ptr());
        assert_eq!(tail.set(usize::MAX, 9), None);
        assert!(!tail.is_owned());
        assert_eq!(tail.set(0, 8), Some(4));
        assert_eq!(original.to_vec(), [2, 4, 6]);
        assert_eq!(tail.to_vec(), [8, 6]);
        assert!(original.get_subslice(Range { start: 2, end: 1 }).is_none());
        assert!(original.get_subslice(0..usize::MAX).is_none());
        assert!(original.get_subslice(3..3).unwrap().is_empty());
        original.get_subslice(1..3).unwrap().into_owned()
    };
    assert_eq!(detached.to_vec(), [4, 6]);
}

#[test]
fn builtin_packed_storage_remains_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PackedArray<u32>>();
    assert_send_sync::<PackedArray<Vec3>>();
    assert_send_sync::<PackedArray<Quat>>();
    assert_send_sync::<PackedArray<Mat4>>();
}

#[test]
fn owning_storage_does_not_require_static_semantic_values() {
    use std::marker::PhantomData;

    #[derive(Clone, Copy)]
    struct Tagged<'a>(u8, PhantomData<&'a ()>);
    impl AsULE for Tagged<'_> {
        type ULE = u8;
        fn to_unaligned(self) -> u8 {
            self.0
        }
        fn from_unaligned(value: u8) -> Self {
            Self(value, PhantomData)
        }
    }
    impl PackedValue for Tagged<'_> {
        type Storage = Self;
        fn pack(self) -> Self {
            self
        }
        fn unpack(value: Self) -> Self {
            value
        }
    }
    fn detach<'a>(value: Tagged<'a>) -> PackedArray<'static, Tagged<'a>> {
        [value].into_packed_array().into_owned()
    }
    let values = detach(Tagged(7, PhantomData));
    assert_eq!(values.clone().first().unwrap().0, 7);
}

#[test]
fn matrices_and_scalar_collections_round_trip() {
    let matrix = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 3., 4.),
        Quat::IDENTITY,
        Vec3::new(5., 6., 7.),
    );
    let values = [matrix].into_packed_array();
    assert_eq!(values.as_bytes().len(), 64);
    assert_eq!(
        PackedArray::<Mat4>::parse_bytes(values.as_bytes())
            .unwrap()
            .first(),
        Some(matrix)
    );
    let scalar = [1u32, 2, 3].into_packed_array();
    assert_eq!(scalar.iter().rev().collect::<Vec<_>>(), [3, 2, 1]);
}
