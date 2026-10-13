//! Embedded and standalone mesh views with statically selected native fields.
use super::super as m2;
use super::super::records::ModelFlags;
use super::DynamicSkin;
use crate::model::common::Ownership;
use m2::codec::{Decoder, Encoder};
use m2::records::{
    ArrayVisitor, Batch, ScalarField, ShadowBatch, SkinSection, Validate, VisitArrays,
};
use m2::{
    ModelLayout, ReadError, RecordLayout as _, V272PreSplit, WriteError, ZeroVec, V256, V257, V260,
    V263, V264, V265, V272, V273, V274,
};

/// Native shadow-profile metadata present in version 265 and later layouts.
#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
pub struct ShadowTables {
    /// Shadow rendering batches in native order.
    pub batches: Vec<ShadowBatch>,
    /// Two reserved trailing words with no established semantic meaning; preserved verbatim.
    pub reserved: [u32; 2],
}
impl Validate for ShadowTables {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data> VisitArrays<'data> for ShadowTables {
    fn visit_arrays<A: ArrayVisitor<'data>>(&mut self, _: &mut A) -> Result<(), A::Error> {
        Ok(())
    }
}
/// Version-selected shadow tables; absent layouts use the zero-width unit type.
pub trait SkinLayout {
    /// Native shadow-profile fields, including the reserved trailing words.
    type Shadows: ScalarField;
}
macro_rules! skin_layouts {
    ($shadow:ty; $($version:ty),*$(,)?)=>{$(impl SkinLayout for $version {type Shadows=$shadow;})*};
}
skin_layouts!(() ; V256,V257,V260,V263,V264);
skin_layouts!(ShadowTables ; V265,V272,V272PreSplit,V273,V274);

/// Layouts whose profiles have standalone SKIN file framing.
pub trait SupportsExternalSkins: ModelLayout {
    /// Wraps this layout's standalone document without erasing its native fields.
    fn into_dynamic_skin<'data>(file: Skin<'data, Self>) -> DynamicSkin<'data>;
}
macro_rules! external_skin_layouts {
    ($($version:ident),*$(,)?)=>{$(impl SupportsExternalSkins for $version{
        fn into_dynamic_skin<'data>(file:Skin<'data,Self>)->DynamicSkin<'data>{DynamicSkin::$version(file)}
    })*};
}
external_skin_layouts!(V264, V265, V272, V272PreSplit, V273, V274);

/// A profile header and its arrays, without standalone SKIN framing.
#[derive(Clone, Debug, Default, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout)]
pub struct SkinProfile<'data, V: ModelLayout> {
    /// Vertex remap into the owning model's vertex array; split LOD geometry also applies `lod_vertex_base` when selected by the model flags.
    pub vertices: ZeroVec<'data, u16>,
    /// Triangle corner indices into `vertices`, with three consecutive entries per triangle.
    pub indices: ZeroVec<'data, u16>,
    /// Four bone-remapping bytes per profile vertex, corresponding to its four bone slots.
    pub bones: ZeroVec<'data, [u8; 4]>,
    /// Submesh vertex/triangle ranges, bone-combination ranges, and layout-selected sorting metadata.
    pub sections: Vec<SkinSection<V>>,
    /// Material passes referencing submesh sections and the owning core's material/texture lookup tables.
    pub batches: Vec<Batch>,
    /// Base vertex offset for a split LOD vertex window; the model flags determine whether this offset is applied.
    pub lod_vertex_base: u32,
    /// Shadow tables in supporting layouts, or an absent unit field.
    pub shadows: V::Shadows,
}
impl<V: ModelLayout> Validate for SkinProfile<'_, V> {
    fn validate(&self) -> Result<(), ReadError> {
        if !self.indices.len().is_multiple_of(3) {
            return Err(ReadError::invalid(0, "skin triangle cardinality"));
        }
        if self
            .indices
            .iter()
            .any(|index| index as usize >= self.vertices.len())
        {
            return Err(ReadError::invalid(0, "skin vertex index"));
        }
        Ok(())
    }
}
impl<'data, V: ModelLayout> VisitArrays<'data> for SkinProfile<'data, V> {
    fn visit_arrays<A: ArrayVisitor<'data>>(&mut self, _: &mut A) -> Result<(), A::Error> {
        Ok(())
    }
}
/// A standalone SKIN file for an explicitly selected model layout.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct Skin<'data, V: ModelLayout> {
    /// Native profile tables; array offsets use the complete file as their base.
    pub profile: SkinProfile<'data, V>,
}
impl<'data, V: SupportsExternalSkins> Skin<'data, V> {
    /// Reads a standalone profile using the selected model layout.
    pub fn read(data: &'data [u8]) -> Result<Self, ReadError> {
        <Self as m2::Read>::read(&mut Decoder::new(data, V::VERSION, ModelFlags::default()))
    }
    /// Rebuilds the profile and assigns fresh local offsets.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        let mut encoder = Encoder::new(V::VERSION, ModelFlags::default());
        m2::Write::write(self, &mut encoder)?;
        Ok(encoder.finish())
    }
}
impl<'data, V: SupportsExternalSkins> m2::Read<'data> for Skin<'data, V> {
    fn read(decoder: &mut Decoder<'data>) -> Result<Self, ReadError> {
        if decoder.bytes(4)? != b"SKIN" {
            return Err(ReadError::invalid(0, "skin magic"));
        }
        Ok(Self {
            profile: SkinProfile::read(decoder)?,
        })
    }
}
impl<V: SupportsExternalSkins> m2::Write for Skin<'_, V> {
    fn write(&self, encoder: &mut Encoder) -> Result<(), WriteError> {
        encoder.reserve_metadata(Self::size(V::VERSION, encoder.flags.bits()))?;
        encoder.bytes(b"SKIN")?;
        self.profile.write(encoder)
    }
}
impl<V: ModelLayout> m2::RecordLayout for Skin<'_, V> {
    fn size(_: u32, flags: u32) -> usize {
        4 + SkinProfile::<V>::size(V::VERSION, flags)
    }
}
