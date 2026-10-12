//! Typed core layouts and independently selected file containers.
//!
//! M2's header version identifies record layouts, not an expansion. For example,
//! version 272 appears in several client generations and in both flat and chunked
//! containers. Choose the layout and envelope separately; changing the envelope
//! does not convert the records.
//!
//! ```
//! use wc3::model::m2::{Chunked, Dialect, Flat, ModelLayout, V272};
//! assert_eq!(<Flat<V272> as Dialect>::Layout::VERSION, 272);
//! assert_eq!(<Chunked<V272> as Dialect>::Layout::VERSION, 272);
//! ```
use super::codec::Value;
use super::companions::{CompanionLayout, SkinLayout, SkinProfile};
use super::container::{ChunkStream, DocumentStorage};
use super::model::Core;
use super::model::{FlaggedCombiner, HeaderLayout, HeaderTail256, HeaderTail257, OpaqueArray};
use super::records::*;
use crate::model::common::{Borrowed, Detached};
use std::fmt::Debug;
use std::marker::PhantomData;
use zerovec::ZeroVec;

mod sealed {
    pub trait Layout {}
    pub trait Container {}
}

/// A supported on-disk core record layout.
///
/// Native fields have the selected layout's exact type. A classic bone cannot
/// contain a compressed quaternion, and an external-skin core cannot contain
/// embedded profiles.
///
/// ```compile_fail
/// use wc3::model::m2::{records::Bone,V256};
/// let mut bone = Bone::<V256>::new();
/// bone.name_crc = 7u32;
/// ```
///
/// ```compile_fail
/// use wc3::model::m2::{Core,V264};
/// let mut core = Core::<V264>::new();
/// core.skins = Vec::new();
/// ```
///
/// Layouts are sealed so their numeric identities and field selections remain
/// consistent with the codec. Numeric versions with no established layout are
/// reported as unsupported rather than assigned an expansion by approximation.
pub trait ModelLayout:
    sealed::Layout
    + Clone
    + Copy
    + Debug
    + Default
    + Eq
    + 'static
    + TrackLayout
    + BoneLayout
    + SequenceLayout
    + CameraLayout
    + RibbonLayout
    + ParticleLayout
    + SkinSectionLayout
    + HeaderLayout
    + SkinLayout
    + CompanionLayout
{
    /// Exact version stored in the MD20 core header.
    const VERSION: u32;
}

macro_rules! layouts {
    ($($name:ident = $number:literal; $doc:literal),* $(,)?) => {
        $(
            #[doc = $doc]
            #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
            pub struct $name;
            impl sealed::Layout for $name {}
            impl ModelLayout for $name { const VERSION: u32 = $number; }
        )*
    };
}
layouts! {
    V256 = 256; "Version 256: original release layout with float bone rotations and inline skins.",
    V257 = 257; "Version 257: classic layout with an additional header array descriptor.",
    V260 = 260; "Version 260: compressed bone rotations with legacy timeline tracks and inline skins.",
    V263 = 263; "Version 263: late TBC layout with legacy timelines and inline skins.",
    V264 = 264; "Version 264: per-sequence tracks and external skin profiles.",
    V265 = 265; "Version 265: animated camera field of view and revised particle fields.",
    V272 = 272; "Version 272: split sequence blend times and extended particle records.",
    V273 = 273; "Version 273 core layout.",
    V274 = 274; "Version 274 core layout."
}

/// Version 272 interpreted using the pre-WoD single blend-time word.
///
/// The header number is shared across Cata, MoP, and WoD. Choose this profile
/// when client provenance establishes the older sequence interpretation; the
/// ordinary [`V272`] profile selects separate incoming and outgoing halfwords.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct V272PreSplit;
impl sealed::Layout for V272PreSplit {}
impl ModelLayout for V272PreSplit {
    const VERSION: u32 = 272;
}

impl TrackLayout for V256 {
    type Timeline<'data> = LegacyTimeline<'data>;
    type TrackValues<'data, T>
        = LegacyValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = LegacySplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V256 {
    type NameCrc = ();
    type RotationValue = [f32; 4];
}
impl SequenceLayout for V256 {
    type Timing = LegacyTiming;
    type BlendTime = u32;
}
impl CameraLayout for V256 {
    type FovPrefix = f32;
    type FovTrack<'data> = ();
}
impl RibbonLayout for V256 {
    type Tail = ();
}
impl ParticleLayout for V256 {
    type Header = WideParticleHeader;
    type TextureScale = LegacyTextureScale;
    type Variation = ();
    type Lifetime<'data> = LegacyParticleLifetime;
    type Spin = f32;
    type Scroll = ();
}
impl SkinSectionLayout for V256 {
    type Tail = ();
}
impl HeaderLayout for V256 {
    const HEADER_SIZE: usize = 340;
    type Skins<'data> = Vec<SkinProfile<'data, Self>>;
    type PlayableLookup<'data> = ZeroVec<'data, u32>;
    type Flipbooks<'data> = ();
    type ClassicLookup<'data> = OpaqueArray<'data>;
    type ClassicTail<'data> = HeaderTail256;
    type Combiner<'data> = ();
}
impl TrackLayout for V257 {
    type Timeline<'data> = LegacyTimeline<'data>;
    type TrackValues<'data, T>
        = LegacyValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = LegacySplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V257 {
    type NameCrc = ();
    type RotationValue = [f32; 4];
}
impl SequenceLayout for V257 {
    type Timing = LegacyTiming;
    type BlendTime = u32;
}
impl CameraLayout for V257 {
    type FovPrefix = f32;
    type FovTrack<'data> = ();
}
impl RibbonLayout for V257 {
    type Tail = ();
}
impl ParticleLayout for V257 {
    type Header = WideParticleHeader;
    type TextureScale = LegacyTextureScale;
    type Variation = ();
    type Lifetime<'data> = LegacyParticleLifetime;
    type Spin = f32;
    type Scroll = ();
}
impl SkinSectionLayout for V257 {
    type Tail = ();
}
impl HeaderLayout for V257 {
    const HEADER_SIZE: usize = 340;
    type Skins<'data> = Vec<SkinProfile<'data, Self>>;
    type PlayableLookup<'data> = ZeroVec<'data, u32>;
    type Flipbooks<'data> = ();
    type ClassicLookup<'data> = OpaqueArray<'data>;
    type ClassicTail<'data> = HeaderTail257<'data>;
    type Combiner<'data> = ();
}
impl TrackLayout for V260 {
    type Timeline<'data> = LegacyTimeline<'data>;
    type TrackValues<'data, T>
        = LegacyValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = LegacySplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V260 {
    type NameCrc = u32;
    type RotationValue = [i16; 4];
}
impl SequenceLayout for V260 {
    type Timing = LegacyTiming;
    type BlendTime = u32;
}
impl CameraLayout for V260 {
    type FovPrefix = f32;
    type FovTrack<'data> = ();
}
impl RibbonLayout for V260 {
    type Tail = ();
}
impl ParticleLayout for V260 {
    type Header = WideParticleHeader;
    type TextureScale = LegacyTextureScale;
    type Variation = ();
    type Lifetime<'data> = LegacyParticleLifetime;
    type Spin = f32;
    type Scroll = ();
}
impl SkinSectionLayout for V260 {
    type Tail = SkinSort;
}
impl HeaderLayout for V260 {
    const HEADER_SIZE: usize = 324;
    type Skins<'data> = Vec<SkinProfile<'data, Self>>;
    type PlayableLookup<'data> = ZeroVec<'data, u32>;
    type Flipbooks<'data> = Vec<TextureFlipbook<'data, Self>>;
    type ClassicLookup<'data> = ();
    type ClassicTail<'data> = ();
    type Combiner<'data> = FlaggedCombiner<'data>;
}
impl TrackLayout for V263 {
    type Timeline<'data> = LegacyTimeline<'data>;
    type TrackValues<'data, T>
        = LegacyValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = LegacySplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V263 {
    type NameCrc = u32;
    type RotationValue = [i16; 4];
}
impl SequenceLayout for V263 {
    type Timing = LegacyTiming;
    type BlendTime = u32;
}
impl CameraLayout for V263 {
    type FovPrefix = f32;
    type FovTrack<'data> = ();
}
impl RibbonLayout for V263 {
    type Tail = ();
}
impl ParticleLayout for V263 {
    type Header = PackedParticleHeader;
    type TextureScale = LegacyTextureScale;
    type Variation = ();
    type Lifetime<'data> = LegacyParticleLifetime;
    type Spin = f32;
    type Scroll = ();
}
impl SkinSectionLayout for V263 {
    type Tail = SkinSort;
}
impl HeaderLayout for V263 {
    const HEADER_SIZE: usize = 324;
    type Skins<'data> = Vec<SkinProfile<'data, Self>>;
    type PlayableLookup<'data> = ZeroVec<'data, u32>;
    type Flipbooks<'data> = Vec<TextureFlipbook<'data, Self>>;
    type ClassicLookup<'data> = ();
    type ClassicTail<'data> = ();
    type Combiner<'data> = FlaggedCombiner<'data>;
}
impl TrackLayout for V264 {
    type Timeline<'data> = ModernTimeline<'data>;
    type TrackValues<'data, T>
        = TrackValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = ModernSplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V264 {
    type NameCrc = u32;
    type RotationValue = [i16; 4];
}
impl SequenceLayout for V264 {
    type Timing = u32;
    type BlendTime = u32;
}
impl CameraLayout for V264 {
    type FovPrefix = f32;
    type FovTrack<'data> = ();
}
impl RibbonLayout for V264 {
    type Tail = RibbonModern;
}
impl ParticleLayout for V264 {
    type Header = PackedParticleHeader;
    type TextureScale = LegacyTextureScale;
    type Variation = f32;
    type Lifetime<'data> = ModernParticleLifetime<'data>;
    type Spin = ModernSpin;
    type Scroll = FlaggedScroll;
}
impl SkinSectionLayout for V264 {
    type Tail = SkinSort;
}
impl HeaderLayout for V264 {
    const HEADER_SIZE: usize = 304;
    type Skins<'data> = u32;
    type PlayableLookup<'data> = ();
    type Flipbooks<'data> = ();
    type ClassicLookup<'data> = ();
    type ClassicTail<'data> = ();
    type Combiner<'data> = FlaggedCombiner<'data>;
}
impl TrackLayout for V265 {
    type Timeline<'data> = ModernTimeline<'data>;
    type TrackValues<'data, T>
        = TrackValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = ModernSplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V265 {
    type NameCrc = u32;
    type RotationValue = [i16; 4];
}
impl SequenceLayout for V265 {
    type Timing = u32;
    type BlendTime = u32;
}
impl CameraLayout for V265 {
    type FovPrefix = ();
    type FovTrack<'data> = SplineTrack<'data, f32, Self>;
}
impl RibbonLayout for V265 {
    type Tail = RibbonModern;
}
impl ParticleLayout for V265 {
    type Header = PackedParticleHeader;
    type TextureScale = ModernTextureScale;
    type Variation = f32;
    type Lifetime<'data> = ModernParticleLifetime<'data>;
    type Spin = ModernSpin;
    type Scroll = FlaggedScroll;
}
impl SkinSectionLayout for V265 {
    type Tail = SkinSort;
}
impl HeaderLayout for V265 {
    const HEADER_SIZE: usize = 304;
    type Skins<'data> = u32;
    type PlayableLookup<'data> = ();
    type Flipbooks<'data> = ();
    type ClassicLookup<'data> = ();
    type ClassicTail<'data> = ();
    type Combiner<'data> = FlaggedCombiner<'data>;
}
impl TrackLayout for V272 {
    type Timeline<'data> = ModernTimeline<'data>;
    type TrackValues<'data, T>
        = TrackValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = ModernSplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V272 {
    type NameCrc = u32;
    type RotationValue = [i16; 4];
}
impl SequenceLayout for V272 {
    type Timing = u32;
    type BlendTime = SplitBlendTime;
}
impl CameraLayout for V272 {
    type FovPrefix = ();
    type FovTrack<'data> = SplineTrack<'data, f32, Self>;
}
impl RibbonLayout for V272 {
    type Tail = RibbonModern;
}
impl ParticleLayout for V272 {
    type Header = PackedParticleHeader;
    type TextureScale = ModernTextureScale;
    type Variation = f32;
    type Lifetime<'data> = ModernParticleLifetime<'data>;
    type Spin = ModernSpin;
    type Scroll = ExtendedScroll;
}
impl SkinSectionLayout for V272 {
    type Tail = SkinSort;
}
impl HeaderLayout for V272 {
    const HEADER_SIZE: usize = 304;
    type Skins<'data> = u32;
    type PlayableLookup<'data> = ();
    type Flipbooks<'data> = ();
    type ClassicLookup<'data> = ();
    type ClassicTail<'data> = ();
    type Combiner<'data> = ();
}
impl TrackLayout for V272PreSplit {
    type Timeline<'data> = ModernTimeline<'data>;
    type TrackValues<'data, T>
        = TrackValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = ModernSplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V272PreSplit {
    type NameCrc = u32;
    type RotationValue = [i16; 4];
}
impl SequenceLayout for V272PreSplit {
    type Timing = u32;
    type BlendTime = u32;
}
impl CameraLayout for V272PreSplit {
    type FovPrefix = ();
    type FovTrack<'data> = SplineTrack<'data, f32, Self>;
}
impl RibbonLayout for V272PreSplit {
    type Tail = RibbonModern;
}
impl ParticleLayout for V272PreSplit {
    type Header = PackedParticleHeader;
    type TextureScale = ModernTextureScale;
    type Variation = f32;
    type Lifetime<'data> = ModernParticleLifetime<'data>;
    type Spin = ModernSpin;
    type Scroll = ExtendedScroll;
}
impl SkinSectionLayout for V272PreSplit {
    type Tail = SkinSort;
}
impl HeaderLayout for V272PreSplit {
    const HEADER_SIZE: usize = 304;
    type Skins<'data> = u32;
    type PlayableLookup<'data> = ();
    type Flipbooks<'data> = ();
    type ClassicLookup<'data> = ();
    type ClassicTail<'data> = ();
    type Combiner<'data> = ();
}
impl TrackLayout for V273 {
    type Timeline<'data> = ModernTimeline<'data>;
    type TrackValues<'data, T>
        = TrackValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = ModernSplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V273 {
    type NameCrc = u32;
    type RotationValue = [i16; 4];
}
impl SequenceLayout for V273 {
    type Timing = u32;
    type BlendTime = SplitBlendTime;
}
impl CameraLayout for V273 {
    type FovPrefix = ();
    type FovTrack<'data> = SplineTrack<'data, f32, Self>;
}
impl RibbonLayout for V273 {
    type Tail = RibbonModern;
}
impl ParticleLayout for V273 {
    type Header = PackedParticleHeader;
    type TextureScale = ModernTextureScale;
    type Variation = f32;
    type Lifetime<'data> = ModernParticleLifetime<'data>;
    type Spin = ModernSpin;
    type Scroll = ExtendedScroll;
}
impl SkinSectionLayout for V273 {
    type Tail = SkinSort;
}
impl HeaderLayout for V273 {
    const HEADER_SIZE: usize = 304;
    type Skins<'data> = u32;
    type PlayableLookup<'data> = ();
    type Flipbooks<'data> = ();
    type ClassicLookup<'data> = ();
    type ClassicTail<'data> = ();
    type Combiner<'data> = ();
}
impl TrackLayout for V274 {
    type Timeline<'data> = ModernTimeline<'data>;
    type TrackValues<'data, T>
        = TrackValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
    type SplineValues<'data, T>
        = ModernSplineValues<'data, T>
    where
        T: Value,
        [T; 3]: Value;
}
impl BoneLayout for V274 {
    type NameCrc = u32;
    type RotationValue = [i16; 4];
}
impl SequenceLayout for V274 {
    type Timing = u32;
    type BlendTime = SplitBlendTime;
}
impl CameraLayout for V274 {
    type FovPrefix = ();
    type FovTrack<'data> = SplineTrack<'data, f32, Self>;
}
impl RibbonLayout for V274 {
    type Tail = RibbonModern;
}
impl ParticleLayout for V274 {
    type Header = PackedParticleHeader;
    type TextureScale = ModernTextureScale;
    type Variation = f32;
    type Lifetime<'data> = ModernParticleLifetime<'data>;
    type Spin = ModernSpin;
    type Scroll = ExtendedScroll;
}
impl SkinSectionLayout for V274 {
    type Tail = SkinSort;
}
impl HeaderLayout for V274 {
    const HEADER_SIZE: usize = 304;
    type Skins<'data> = u32;
    type PlayableLookup<'data> = ();
    type Flipbooks<'data> = ();
    type ClassicLookup<'data> = ();
    type ClassicTail<'data> = ();
    type Combiner<'data> = ();
}

/// A file envelope independent of the numeric core layout.
pub trait Container: sealed::Container + Clone + Copy + Debug + Default + Eq + 'static {
    /// Native storage selected by the file envelope.
    type Storage<'data, V: ModelLayout>: DocumentStorage<'data, V>
        + for<'view> Borrowed<'view, View = Self::Storage<'view, V>>
        + for<'owned> Detached<'owned, Detached = Self::Storage<'owned, V>>;
}

/// A flat MD20 envelope whose array offsets are relative to the file start.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FlatContainer;
impl sealed::Container for FlatContainer {}
impl Container for FlatContainer {
    type Storage<'data, V: ModelLayout> = Core<'data, V>;
}

/// A chunk stream containing an MD21 core payload with its own offset base.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChunkedContainer;
impl sealed::Container for ChunkedContainer {}
impl Container for ChunkedContainer {
    type Storage<'data, V: ModelLayout> = ChunkStream<'data, V>;
}

/// Selects a typed core layout and its independently chosen envelope.
pub trait Dialect: Clone + Copy + Debug + Default + Eq + 'static {
    /// Layout of the core records.
    type Layout: ModelLayout;
    /// Envelope containing the core and any auxiliary chunks.
    type Container: Container;
    /// Exact numeric core version selected by this dialect.
    const VERSION: u32 = Self::Layout::VERSION;
}

/// A flat MD20 file using layout `V`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Flat<V: ModelLayout>(PhantomData<V>);
impl<V: ModelLayout> Dialect for Flat<V> {
    type Layout = V;
    type Container = FlatContainer;
}

/// A chunked file containing an MD21 payload using layout `V`.
///
/// Wrapping a legacy layout is allowed. Historical exporters may not have used
/// that combination, but the envelope itself does not alter its record layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Chunked<V: ModelLayout>(PhantomData<V>);
impl<V: ModelLayout> Dialect for Chunked<V> {
    type Layout = V;
    type Container = ChunkedContainer;
}

#[cfg(test)]
mod tests {
    use super::*;
    fn header<V: ModelLayout>(width: usize) {
        let bytes = Core::<V>::new().encode().unwrap();
        assert_eq!(bytes.len(), width);
        assert_eq!(&bytes[..4], b"MD20");
        assert_eq!(
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            V::VERSION
        );
        assert_eq!(Core::<V>::decode(&bytes, 0).unwrap(), Core::<V>::new());
    }
    #[test]
    fn typed_empty_headers_have_native_widths() {
        header::<V256>(340);
        header::<V257>(340);
        header::<V260>(324);
        header::<V263>(324);
        header::<V264>(304);
        header::<V265>(304);
        header::<V272>(304);
        header::<V272PreSplit>(304);
        header::<V273>(304);
        header::<V274>(304);
    }
}
