//! Dialect-selected scalar fields without document dependencies.
use super::*;
use crate::model::error::Location;
/// Reads and constructs dialect-selected bone-name checksum fields.
pub trait NameCrcField: Sized {
    /// Returns the stored checksum, zero when the field is absent.
    fn get(&self) -> u32;
    /// Constructs the native field from its stored checksum.
    fn make(value: u32, context: &mut Context) -> Result<Self, Error>;
}
impl NameCrcField for () {
    fn get(&self) -> u32 {
        0
    }
    fn make(value: u32, context: &mut Context) -> Result<Self, Error> {
        if value != 0 {
            context.unsupported(
                Location::new().field("bones").field("name_crc"),
                "target classic layout omits bone-name CRCs",
            )?;
        }
        Ok(())
    }
}
impl NameCrcField for u32 {
    fn get(&self) -> u32 {
        *self
    }
    fn make(value: u32, _: &mut Context) -> Result<Self, Error> {
        Ok(value)
    }
}
/// Reads and constructs shared or independently timed playback transitions.
pub trait BlendField: Sized {
    /// Returns incoming and outgoing durations in native milliseconds.
    fn parts(&self) -> [u32; 2];
    /// Constructs the selected native transition fields.
    fn make(parts: [u32; 2], context: &mut Context) -> Result<Self, Error>;
}
impl BlendField for u32 {
    fn parts(&self) -> [u32; 2] {
        [*self; 2]
    }
    fn make(parts: [u32; 2], context: &mut Context) -> Result<Self, Error> {
        if parts[0] != parts[1] {
            context.unsupported(
                Location::new().field("clips").field("blend"),
                "target layout has one shared transition duration",
            )?;
        }
        Ok(parts[0])
    }
}
impl BlendField for native::records::SplitBlendTime {
    fn parts(&self) -> [u32; 2] {
        [u32::from(self.incoming), u32::from(self.outgoing)]
    }
    fn make(parts: [u32; 2], _: &mut Context) -> Result<Self, Error> {
        Ok(Self {
            incoming: u16::try_from(parts[0]).map_err(|_| {
                Error::requirement(
                    Location::new().field("clips").field("blend_in"),
                    "transition exceeds u16",
                )
            })?,
            outgoing: u16::try_from(parts[1]).map_err(|_| {
                Error::requirement(
                    Location::new().field("clips").field("blend_out"),
                    "transition exceeds u16",
                )
            })?,
        })
    }
}
/// Reads and constructs dialect-selected particle texture-atlas controls.
pub trait FlipbookField<'data, V: ModelLayout>: Sized {
    /// Borrows the legacy frame-selection records, empty when the field is absent.
    fn records(&self) -> &[native::records::TextureFlipbook<'data, V>];
    /// Installs records in a target supporting this field.
    fn make(
        records: Vec<native::records::TextureFlipbook<'data, V>>,
        context: &mut Context,
    ) -> Result<Self, Error>;
}
impl<'data, V: ModelLayout> FlipbookField<'data, V> for () {
    fn records(&self) -> &[native::records::TextureFlipbook<'data, V>] {
        &[]
    }
    fn make(
        records: Vec<native::records::TextureFlipbook<'data, V>>,
        context: &mut Context,
    ) -> Result<Self, Error> {
        if !records.is_empty() {
            context.unsupported(
                Location::new().field("textures").field("flipbooks"),
                "target core layout has no legacy texture-frame array",
            )?;
        }
        Ok(())
    }
}
impl<'data, V: ModelLayout> FlipbookField<'data, V>
    for Vec<native::records::TextureFlipbook<'data, V>>
{
    fn records(&self) -> &[native::records::TextureFlipbook<'data, V>] {
        self
    }
    fn make(
        records: Vec<native::records::TextureFlipbook<'data, V>>,
        _: &mut Context,
    ) -> Result<Self, Error> {
        Ok(records)
    }
}

/// Dialect-selected section sorting controls.
pub trait SectionSorting: Sized {
    /// Logical sorting sphere, when the dialect includes it.
    fn logical(&self) -> Option<([f32; 3], f32)>;
    /// Constructs the dialect's field representation.
    fn make(value: Option<([f32; 3], f32)>, context: &mut Context) -> Result<Self, Error>;
}
impl SectionSorting for () {
    fn logical(&self) -> Option<([f32; 3], f32)> {
        None
    }
    fn make(value: Option<([f32; 3], f32)>, context: &mut Context) -> Result<Self, Error> {
        if value.is_some() {
            context.unsupported(
                Location::new().field("m2").field("geoset").field("sorting"),
                "target dialect has no authored sorting sphere",
            )?
        }
        Ok(())
    }
}
impl SectionSorting for native::records::SkinSort {
    fn logical(&self) -> Option<([f32; 3], f32)> {
        Some((self.center, self.radius))
    }
    fn make(value: Option<([f32; 3], f32)>, _: &mut Context) -> Result<Self, Error> {
        let (center, radius) = value.unwrap_or(([0.; 3], 0.));
        Ok(Self { center, radius })
    }
}
