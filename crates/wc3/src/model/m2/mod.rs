//! World of Warcraft M2 documents, typed layouts, and supporting file codecs.
//!
//! [`Model`](crate::model::m2::Model) selects the core layout and file envelope through
//! its dialect. Records borrow encoded leaf arrays; edits allocate only changed
//! buffers. Companion codecs retain their native documents and external references.
//! Whole-document encoding assigns fresh offsets to locally stored data.
//!
//! Layout types select native record fields. A classic bone has float rotations
//! and no name CRC; these choices cannot be replaced with newer representations.
//!
//! ```compile_fail
//! use wc3::model::m2::{records::{Bone, Track}, V256};
//! let mut bone = Bone::<V256>::default();
//! bone.rotation = Track::<[i16; 4], V256>::default();
//! ```
//!
//! ```compile_fail
//! use wc3::model::m2::{records::Bone, V256};
//! let mut bone = Bone::<V256>::default();
//! bone.name_crc = 123u32;
//! ```
//!
//! The two version-272 profiles also select different blend-time types.
//!
//! ```compile_fail
//! use wc3::model::m2::{records::{Sequence, SplitBlendTime}, V272PreSplit};
//! let mut sequence = Sequence::<V272PreSplit>::default();
//! sequence.blend_time = SplitBlendTime { incoming: 10, outgoing: 20 };
//! ```
mod error;
pub use error::{ReadError, WriteError};
pub mod codec;
mod dialect;
pub use dialect::*;
pub mod chunks;
pub mod companions;
mod model;
pub mod records;
pub use model::{
    Core, FlaggedCombiner, HeaderField, HeaderLayout, HeaderTail256, HeaderTail257, OpaqueArray,
    SkinStorage,
};
mod container;
pub use crate::model::common::{AsULE, Borrowed, IntoOwned, IntoZeroVec, ZeroVec};
pub use container::{ChunkStream, DocumentStorage, Model, ModelChunk};

mod dynamic;
pub use dynamic::{DynamicLayout, DynamicModel};
mod io;
pub use io::{from_reader, from_reader_dynamic, to_writer, Read, RecordLayout, Write};

mod bundle;
pub mod resources;
pub use bundle::{
    BoneTable, Bundle, EncodedBundle, File, ProfileRef, ResolvedTrack, ResolvedValues,
};

mod validation;
pub use resources::{Dependency, DependencyKind, FileKey};

/// Standalone skin profiles and their shared embedded representation.
pub mod skin {
    pub use super::companions::{Skin as File, SkinProfile as Profile};
    pub use super::records::{Batch, ShadowBatch, SkinSection as Section};
}
/// Raw or chunked animation streams interpreted through model track descriptors.
pub mod animation {
    pub use super::companions::{AnimationFile as File, AnimationStream as Stream};
    pub use super::records::{ParticleTrack, Track, TrackBase, TrackValues};
}
/// Shared skeleton records and explicit parent-skeleton references.
pub mod skeleton {
    pub use super::companions::{Skeleton as File, SkeletonChunk as Chunk};
}
/// Sparse character customization bone overrides.
pub mod bone {
    pub use super::companions::{BoneFile as File, BoneMatrix as Matrix};
}
/// Versioned physics documents, including inline PFDC payloads.
pub mod physics {
    pub use super::companions::{
        PhysicsBody as Body, PhysicsBody2 as Body2, PhysicsBody3 as Body3, PhysicsBody4 as Body4,
    };
    pub use super::companions::{PhysicsChunk as Chunk, PhysicsFile as File, Polytope};
}

/// Modern chunked geometry streams referenced alongside or instead of skin profiles.
pub mod geometry {
    pub use super::companions::{
        GeometryChunk as Chunk, GeometryFile as File, StreamValues, VertexStream,
    };
}

/// Derives field-order M2 record codecs and inline metadata layouts.
pub use wc3_derive::{M2Read as Read, M2RecordLayout as RecordLayout, M2Write as Write};

/// Dispatches an operation to a dynamically detected model’s concrete dialect.
pub use crate::visit_m2_model;

/// Dispatches an operation to a supporting document’s concrete native layout.
pub use crate::{visit_m2_skeleton, visit_m2_skin};
/// Supporting-document dispatch and external-skin capabilities.
pub use companions::{
    CompanionLayout, DynamicSkeleton, DynamicSkin, LayoutKind, SupportsExternalSkins,
};

/// Field-wise validation and context-preserving animation traversal.
pub use records::{ArrayVisitor, Validate, VisitArrays};
/// Derives recursive validation and traversal for ordinary records and enums.
pub use wc3_derive::{M2Validate as Validate, M2VisitArrays as VisitArrays};

/// Derives exact scalar enum wire-code conversions.
pub use wc3_derive::M2Value as Value;
