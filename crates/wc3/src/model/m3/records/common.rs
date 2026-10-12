//! Inline property metadata and bounding shapes shared by section records.
//!
//! Animated values carry identifiers and fallback values within their containing
//! record. Reserved reference-width words occupy the same wire width as links
//! while remaining outside the reference graph.
use crate::model::m3;
/// Metadata and default/reset values for a property addressed by animation ID.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct AnimationValue<T> {
    /// Curve mode code: 0 step, 1 linear, 2 Hermite, 3 Bezier.
    pub interpolation: u16,
    /// Property animation control bits.
    pub flags: u16,
    /// Identifier matched against animation-controller lookup entries.
    pub animation_id: u32,
    /// Value supplied when this property has no selected animated key.
    pub initial_value: T,
    /// Value stored for resetting this property.
    pub null_value: T,
    /// Final signed metadata word; its complete meaning is unresolved.
    pub reserved: i32,
}
/// An axis-aligned box accompanied by a bounding radius.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Extent {
    /// Lower corner of the model-space axis-aligned box.
    pub minimum: [f32; 3],
    /// Upper corner of the model-space axis-aligned box.
    pub maximum: [f32; 3],
    /// Radius stored alongside the axis-aligned box.
    pub radius: f32,
}
/// A center and radius defining a model-space sphere.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Sphere {
    /// Position of the sphere center in model coordinates.
    pub center: [f32; 3],
    /// Distance from the center to the sphere surface in model units.
    pub radius: f32,
}

/// Opaque metadata occupying reference-width slots without participating in the section graph.
#[derive(Clone, Copy, Debug, Default, PartialEq, m3::Validate, m3::VisitReferences)]
pub struct ReservedReferenceWords {
    /// First opaque word, occupying the position normally used for an entry count.
    pub entries: u32,
    /// Second opaque word, occupying the position normally used for a section index.
    pub index: u32,
    /// Third opaque word in MD34; MD33 omits it and requires zero on writing.
    pub flags: u32,
}
impl<'data, E: m3::Envelope> m3::Read<'data, E> for ReservedReferenceWords {
    fn read(input: &mut m3::codec::Decoder<'data, E>) -> Result<Self, m3::ReadError> {
        let words = <m3::Ref<m3::Any> as m3::Read<'data, E>>::read(input)?;
        Ok(Self {
            entries: words.entries,
            index: words.index,
            flags: words.flags,
        })
    }
}
impl<E: m3::Envelope> m3::Write<E> for ReservedReferenceWords {
    fn write(&self, output: &mut m3::codec::Encoder<E>) -> Result<(), m3::WriteError> {
        m3::Write::write(
            &m3::Ref::<m3::Any>::new(self.entries, self.index, self.flags),
            output,
        )
    }
}
impl<E: m3::Envelope> m3::RecordLayout<E> for ReservedReferenceWords {
    fn size() -> usize {
        <m3::Ref<m3::Any> as m3::RecordLayout<E>>::size()
    }
}
