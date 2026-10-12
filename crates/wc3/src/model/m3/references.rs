//! Typed section links independent of section revisions and physical offsets.
use super::codec::{Decoder, Encoder, Read, RecordLayout, Validate, Write};
use super::dialect::Envelope;
use super::{ReadError, WriteError};
use crate::model::common::Ownership;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
/// Identity of a fixed native record revision.
pub trait Record {
    /// Section tag containing this record type.
    const TAG: [u8; 4];
    /// Independent section revision containing this record type.
    const VERSION: u32;
}
/// Expected section family of a reference, independent of its revision.
pub trait Target {
    /// Expected section tag, or `None` for an explicitly untyped reference.
    const TAG: Option<[u8; 4]>;
}
impl<T: Record> Target for T {
    const TAG: Option<[u8; 4]> = Some(<T as Record>::TAG);
}
/// A reference whose section family has not been established.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Any;
impl Target for Any {
    const TAG: Option<[u8; 4]> = None;
}
/// Native reference metadata suitable for graph validation without target loading.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership)]
pub struct RawReference {
    /// Number of target records requested by this reference.
    pub entries: u32,
    /// Stable section ID, initially equal to the source index-table position.
    pub index: u32,
    /// Native MD34 reference flags; absent and zero in MD33.
    pub flags: u32,
    /// Expected section family when known from the record schema.
    pub target: Option<[u8; 4]>,
}
/// Visits encoded section references without resolving or allocating their targets.
pub trait ReferenceVisitor {
    /// Visitor failure.
    type Error;
    /// Receives one reference in native field order.
    fn visit(&mut self, reference: RawReference) -> Result<(), Self::Error>;
}
/// Compositional immutable traversal of encoded section references.
pub trait VisitReferences {
    /// Visits native references in field and array order.
    fn visit_references<V: ReferenceVisitor>(&self, visitor: &mut V) -> Result<(), V::Error>;
}
/// Typed count/index link into a section, with envelope-selected wire width.
///
/// The target marker names a section family, rather than choosing its revision.
/// Empty links retain their native index and flag words without resolving a
/// target. Writing MD33 rejects flags that its eight-byte layout cannot store.
#[derive(Ownership)]
pub struct Ref<T: Target = Any> {
    /// Number of records requested from the target section.
    pub entries: u32,
    /// Stable section ID; section movement does not change this identity.
    pub index: u32,
    /// Native MD34 reference flag word.
    pub flags: u32,
    target: PhantomData<T>,
}
impl<T: Target> Clone for Ref<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Target> Copy for Ref<T> {}
impl<T: Target> Default for Ref<T> {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}
impl<T: Target> Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ref")
            .field("entries", &self.entries)
            .field("index", &self.index)
            .field("flags", &self.flags)
            .field("target", &T::TAG)
            .finish()
    }
}
impl<T: Target> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries && self.index == other.index && self.flags == other.flags
    }
}
impl<T: Target> Eq for Ref<T> {}
impl<T: Target> Ref<T> {
    /// Creates a reference with no flag bits set.
    pub const fn from_section(entries: u32, index: u32) -> Self {
        Self::new(entries, index, 0)
    }
    /// Creates a reference retaining its native count, index, and flag words.
    pub const fn new(entries: u32, index: u32, flags: u32) -> Self {
        Self {
            entries,
            index,
            flags,
            target: PhantomData,
        }
    }
    /// Creates a reference retaining an explicit flag word.
    pub const fn with_flags(entries: u32, index: u32, flags: u32) -> Self {
        Self {
            entries,
            index,
            flags,
            target: PhantomData,
        }
    }
    /// Whether this reference requests no records.
    pub const fn is_empty(&self) -> bool {
        self.entries == 0
    }
    /// Metadata used by structural graph visitors.
    pub fn raw(&self) -> RawReference {
        RawReference {
            entries: self.entries,
            index: self.index,
            flags: self.flags,
            target: T::TAG,
        }
    }
    /// Changes the type-level target expectation without changing encoded words.
    pub const fn cast<U: Target>(self) -> Ref<U> {
        Ref::with_flags(self.entries, self.index, self.flags)
    }
}
impl<'data, T: Target, E: Envelope> Read<'data, E> for Ref<T> {
    fn read(input: &mut Decoder<'data, E>) -> Result<Self, ReadError> {
        let entries = u32::read(input)?;
        let index = u32::read(input)?;
        let flags = if E::REFERENCE_SIZE == 12 {
            u32::read(input)?
        } else {
            0
        };
        Ok(Self::with_flags(entries, index, flags))
    }
}
impl<T: Target, E: Envelope> Write<E> for Ref<T> {
    fn write(&self, output: &mut Encoder<E>) -> Result<(), WriteError> {
        if E::REFERENCE_SIZE == 8 && self.flags != 0 {
            return Err(WriteError::invalid("MD33 reference flags"));
        }
        self.entries.write(output)?;
        self.index.write(output)?;
        if E::REFERENCE_SIZE == 12 {
            self.flags.write(output)?;
        }
        Ok(())
    }
}
impl<T: Target, E: Envelope> RecordLayout<E> for Ref<T> {
    fn size() -> usize {
        E::REFERENCE_SIZE
    }
}
impl<T: Target> Validate for Ref<T> {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<T: Target> VisitReferences for Ref<T> {
    fn visit_references<V: ReferenceVisitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
        visitor.visit(self.raw())
    }
}
macro_rules! leaves {
    ($($ty:ty),*) => {
        $(impl VisitReferences for $ty {
            fn visit_references<V: ReferenceVisitor>(&self, _: &mut V) -> Result<(), V::Error> {
                Ok(())
            }
        })*
    };
}
leaves!((), u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
impl<T: VisitReferences, const N: usize> VisitReferences for [T; N] {
    fn visit_references<V: ReferenceVisitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
        for value in self {
            value.visit_references(visitor)?;
        }
        Ok(())
    }
}
impl<T: VisitReferences> VisitReferences for Vec<T> {
    fn visit_references<V: ReferenceVisitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
        for value in self {
            value.visit_references(visitor)?;
        }
        Ok(())
    }
}

impl VisitReferences for RawReference {
    fn visit_references<V: ReferenceVisitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
        visitor.visit(*self)
    }
}
impl Validate for RawReference {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}

impl<T> VisitReferences for PhantomData<T> {
    fn visit_references<V: ReferenceVisitor>(&self, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
