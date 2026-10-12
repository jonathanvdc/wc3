//! Owned decoding belongs to the codec, independently of buffer ownership.
use mdx::{Read as _, ReadOwned as _};
use std::borrow::Cow;
use wc3::model::common::Ownership;
use wc3::model::mdlx::mdx;

#[derive(Clone, Debug, Ownership, mdx::ReadOwned)]
struct Handwritten<'data> {
    bytes: Cow<'data, [u8]>,
}
impl<'data> mdx::Read<'data> for Handwritten<'data> {
    fn read_mdx(cursor: &mut mdx::Cursor<'data>) -> Result<Self, mdx::ReadError> {
        let count = cursor.read::<u32>()? as usize;
        Ok(Self {
            bytes: Cow::Borrowed(cursor.read_bytes(count)?),
        })
    }
}
#[derive(Debug, PartialEq, mdx::Read, mdx::ReadOwned)]
struct Scalar(u32);
#[derive(Debug)]
struct BorrowedLeaf<'data>(&'data [u8]);
impl<'data> mdx::Read<'data> for BorrowedLeaf<'data> {
    fn read_mdx(cursor: &mut mdx::Cursor<'data>) -> Result<Self, mdx::ReadError> {
        Ok(Self(cursor.read_bytes(4)?))
    }
}
#[derive(Debug, mdx::Read)]
struct BorrowOnly<'data> {
    payload: BorrowedLeaf<'data>,
}

#[test]
fn owned_codec_uses_a_handwritten_reader_and_shared_detachment() {
    let source = [3, 0, 0, 0, 11, 12, 13];
    let borrowed = Handwritten::decode_mdx(&source).unwrap();
    assert!(matches!(borrowed.bytes, Cow::Borrowed(_)));
    assert_eq!(borrowed.bytes.as_ptr(), source[4..].as_ptr());
    let owned = Handwritten::decode_owned(&source).unwrap();
    assert!(matches!(owned.bytes, Cow::Owned(_)));
    assert_ne!(owned.bytes.as_ptr(), source[4..].as_ptr());
    assert_eq!(owned.bytes.as_ref(), [11, 12, 13]);
    let owned: Handwritten<'static> = mdx::from_reader(source.as_slice()).unwrap();
    assert_eq!(owned.bytes.as_ref(), [11, 12, 13]);
    assert!(Handwritten::decode_owned(&source[..6]).is_err());
    assert!(Handwritten::decode_owned(&[0, 0, 0, 0, 1]).is_err());
}
#[test]
fn reading_and_owned_reading_have_independent_ownership_requirements() {
    assert_eq!(
        Scalar::decode_owned(&7u32.to_le_bytes()).unwrap(),
        Scalar(7)
    );
    let bytes = [7, 8, 9, 10];
    let borrowed = BorrowOnly::decode_mdx(&bytes).unwrap();
    assert_eq!(borrowed.payload.0, bytes);
    assert_eq!(borrowed.payload.0.as_ptr(), bytes.as_ptr());
}

struct NonClone(u32);
impl<'data> mdx::Read<'data> for NonClone {
    fn read_mdx(cursor: &mut mdx::Cursor<'data>) -> Result<Self, mdx::ReadError> {
        Ok(Self(cursor.read()?))
    }
}
#[derive(Ownership, mdx::ReadOwned)]
struct Generic<'data, T> {
    bytes: Cow<'data, [u8]>,
    metadata: T,
}
impl<'data, T: mdx::Read<'data>> mdx::Read<'data> for Generic<'data, T> {
    fn read_mdx(cursor: &mut mdx::Cursor<'data>) -> Result<Self, mdx::ReadError> {
        Ok(Self {
            bytes: Cow::Borrowed(cursor.read_bytes(2)?),
            metadata: T::read_mdx(cursor)?,
        })
    }
}
#[test]
fn owned_decoding_moves_metadata_without_requiring_clone() {
    let owned = Generic::<NonClone>::decode_owned(&[10, 11, 7, 0, 0, 0]).unwrap();
    assert!(matches!(owned.bytes, Cow::Owned(_)));
    assert_eq!(owned.bytes.as_ref(), [10, 11]);
    assert_eq!(owned.metadata.0, 7);
}
