//! Conversion between native M2 bundles and editable WEM documents.
use super::{
    Asset, Context, Conversion, Error, ExportOptions, Format, ImportOptions, IssueKind,
    OpaquePayload, Plan, UnknownPolicy,
};
use crate::model::m2::{
    self as native,
    companions::{Skin, SkinProfile},
    records::{SkinSection, Vertex},
};
use crate::model::wem::{self, records as wr, semantic as sm};
use native::chunks::Chunk;
use native::codec::{Array, Decoder, Value};
use native::records::{
    BatchFlags, BlendMode, Bone, BoneFlags, CameraKind, Interpolation as NativeInterpolation,
    LegacyTiming, LightKind, MaterialFlags, ModelFlags, PackedParticleBlend,
    PackedParticleGenerator, ParticleBlend, ParticleFlags, ParticleGenerator, ParticleHeadTail,
    PhysicsAttachment, PhysicsBodyKind, PhysicsGameFlags, PhysicsJointKind, PhysicsKind,
    PhysicsMotorMode, PhysicsShapeKind, SequenceFlags, TextureFlags, TextureSource,
    TimelineStorage, Track, ValuesStorage,
};
use native::{
    Bundle, Chunked, Core, Dialect, File, ModelLayout, V272PreSplit, V256, V257, V260, V263, V264,
    V265, V272, V273, V274,
};
use native::{ProfileRef, ResolvedValues};
use std::array::from_fn;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::slice::from_ref;
use zerovec::ZeroVec;

mod animation;
mod cameras;
mod effects;
mod geometry;
mod hierarchy;
mod layout;
mod materials;
mod metadata;
pub use materials::FlipbookField;
use materials::*;
mod particles;
mod physics;
use physics::*;

use animation::*;
pub use animation::{
    RotationComponents, SequenceClock, SplineFactory, TimelineFactory, ValuesFactory,
};
pub use cameras::CameraConversion;
use effects::*;
use geometry::*;
use hierarchy::*;
pub use layout::{ExportDialect, ExportLayout};
use metadata::*;
pub use metadata::{BlendField, NameCrcField};
pub use particles::ParticleConversion;
use particles::{gravity_vector, gravity_word};

mod export;
mod import;
pub use export::plan;
pub use import::import;
