//! Creation defaults independent of native section revisions in existing files.
use super::records::{ModelRecord, V23, V29, V30};
use super::{Document, Envelope, Md33, Md34, Model, Record, RecordLayout, Ref, Write, WriteError};

mod sealed {
    pub trait Profile {}
}
/// Default root revision for creating a new document. Reading and writing
/// existing documents never converts their records to a creation profile.
pub trait CreationProfile: sealed::Profile {
    /// Native reference envelope for the new document.
    type Envelope: Envelope;
    /// Initial native MODL record revision.
    type Root: Record + Default + Write<Self::Envelope> + RecordLayout<Self::Envelope>;
}
/// MD33 creation with MODL revision 23.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Beta;
impl sealed::Profile for Beta {}
impl CreationProfile for Beta {
    type Envelope = Md33;
    type Root = ModelRecord<V23>;
}
/// MD34 creation with the StarCraft II MODL revision 29.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StarCraft2;
impl sealed::Profile for StarCraft2 {}
impl CreationProfile for StarCraft2 {
    type Envelope = Md34;
    type Root = ModelRecord<V29>;
}
/// MD34 creation with the Heroes of the Storm MODL revision 30.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HeroesOfTheStorm;
impl sealed::Profile for HeroesOfTheStorm {}
impl CreationProfile for HeroesOfTheStorm {
    type Envelope = Md34;
    type Root = ModelRecord<V30>;
}
impl<E: Envelope> Model<'static, E> {
    /// Creates an empty graph using a profile's native root and envelope.
    /// Other sections are added through their independently selected record types.
    pub fn new<P: CreationProfile<Envelope = E>>() -> Result<Self, WriteError> {
        let mut document = Document::<E>::new();
        let id = document.append(&[P::Root::default()])?;
        document.set_root(Ref::<P::Root>::from_section(1, id.index()));
        Self::from_document(document).map_err(|_| WriteError::invalid("creation profile root"))
    }
}
