//! Direct conversion between native M2 bundles and checked logical model graphs.
use super::{Context, Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::formats::m2::{
    self as native,
    companions::{Skin, SkinProfile},
    records::{SkinSection, Vertex},
};
use crate::model::Model;
use native::chunks::Chunk;
use native::codec::{Array, Value};
use native::records::{
    BatchFlags, BlendMode, Bone, BoneFlags, CameraKind, Interpolation as NativeInterpolation,
    LegacyTiming, LightKind, MaterialFlags, ModelFlags, PackedParticleBlend,
    PackedParticleGenerator, ParticleBlend, ParticleFlags, ParticleGenerator, ParticleHeadTail,
    SequenceFlags, TextureFlags, TextureSource, TimelineStorage, Track, ValuesStorage,
};
use native::{
    Bundle, Chunked, Core, Dialect, File, ModelLayout, V272PreSplit, V256, V257, V260, V263, V264,
    V265, V272, V273, V274,
};
use native::{ProfileRef, ResolvedValues};
use std::borrow::Cow;
use std::collections::BTreeMap;

mod animation;
use animation::{ensure_constant, initial, sequence_tables};
pub use animation::{
    RotationComponents, SequenceClock, SplineFactory, TimelineFactory, ValuesFactory,
};
mod cameras;
pub use cameras::CameraConversion;
mod fields;
pub use fields::{BlendField, FlipbookField, NameCrcField, SectionSorting};
mod layout;
pub use layout::{ExportDialect, ExportLayout};
mod particles;
pub use particles::ParticleConversion;
use particles::{gravity_vector, gravity_word};
mod graph;
pub use graph::{import, import_selected, plan};
fn failure(error: impl ToString) -> Error {
    Error::new("m2", error.to_string())
}
fn section_start<V: ModelLayout>(section: &SkinSection<V>) -> usize {
    (usize::from(section.level) << 16) | usize::from(section.index_start)
}

/// Finds a native sequence index by animation ID and variation, including inherited skeleton tables.
/// The returned index can be passed to [`import_selected`].
pub fn find_sequence<D: Dialect>(
    source: &Bundle<'_, D>,
    animation_id: u16,
    variation: u16,
) -> Result<Option<usize>, Error> {
    let (owner, _) = source.bones().map_err(failure)?;
    Ok(sequence_tables(source, owner)?
        .sequences
        .iter()
        .position(|sequence| sequence.id == animation_id && sequence.variation_index == variation))
}
