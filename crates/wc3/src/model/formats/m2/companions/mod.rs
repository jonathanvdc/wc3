//! Standalone M2 companion codecs. Reading a companion never accesses other files.
use super as m2;
use super::records::ModelFlags;
pub(crate) mod fixed;
pub use fixed::*;
mod skin;
pub use skin::{ShadowTables, Skin, SkinLayout, SkinProfile, SupportsExternalSkins};
mod skeleton;
pub use skeleton::{Skeleton, SkeletonChunk};
mod bone;
pub use bone::BoneFile;
mod animation;
pub use animation::{AnimationFile, AnimationStream};
mod physics;
pub use physics::{PhysicsChunk, PhysicsFile, Polytope};

use super::codec::{ArrayRef, Decoder, Encoder};
use super::{ReadError, WriteError};

pub(crate) fn flat_records<'data, T: m2::Read<'data> + m2::RecordLayout>(
    data: &'data [u8],
    version: u32,
    flags: ModelFlags,
) -> Result<Vec<T>, ReadError> {
    let stride = T::size(version, flags.bits());
    if stride == 0 || !data.len().is_multiple_of(stride) {
        return Err(ReadError::invalid(0, "record array byte length"));
    }
    let count = u32::try_from(data.len() / stride)
        .map_err(|_| ReadError::invalid(0, "record array count"))?;
    Decoder::new(data, version, flags).records(ArrayRef { count, offset: 0 })
}
pub(crate) fn encode_records<T: m2::Write + m2::RecordLayout>(
    records: &[T],
    version: u32,
    flags: ModelFlags,
) -> Result<Vec<u8>, WriteError> {
    let mut e = Encoder::new(version, flags);
    e.write_records(records)?;
    Ok(e.finish())
}
pub(crate) fn write_chunk(e: &mut Encoder, tag: [u8; 4], data: &[u8]) -> Result<(), WriteError> {
    let size =
        u32::try_from(data.len()).map_err(|_| WriteError::invalid("chunk size exceeds u32"))?;
    e.bytes(&tag)?;
    e.u32(size)?;
    e.bytes(data)
}

mod geometry;
pub use geometry::{GeometryChunk, GeometryFile, StreamValues, VertexStream};

mod dynamic;
pub use dynamic::{CompanionLayout, DynamicSkeleton, DynamicSkin, LayoutKind};
