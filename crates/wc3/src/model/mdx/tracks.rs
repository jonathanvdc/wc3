//! Track dispatch for fields in a sized record's animation tail.
use super::{Cursor, Encoder, ReadError, WriteError};
use crate::model::{Animatable, Tag, Track, TrackValue};

/// Reads one recognized animation payload, after the enclosing record reads its tag.
pub trait ReadTracks {
    fn read_mdx_track(&mut self, tag: Tag, cursor: &mut Cursor<'_>) -> Result<bool, ReadError>;
}
/// Writes animation tails in declaration order, including nested field groups.
pub trait WriteTracks {
    fn write_mdx_tracks(&self, encoder: &mut Encoder<'_>) -> Result<(), WriteError>;
}
/// Field storage that accepts a replacement animation.
pub trait ReadTrackProperty {
    fn read_mdx_track_property(&mut self, cursor: &mut Cursor<'_>) -> Result<(), ReadError>;
}
/// Field storage that can emit an optional animation.
pub trait WriteTrackProperty {
    fn write_mdx_track_property(
        &self,
        tag: Tag,
        encoder: &mut Encoder<'_>,
    ) -> Result<(), WriteError>;
}
impl<T: TrackValue> ReadTrackProperty for Animatable<T> {
    fn read_mdx_track_property(&mut self, cursor: &mut Cursor<'_>) -> Result<(), ReadError> {
        self.set_track(cursor.read()?);
        Ok(())
    }
}
impl<T: TrackValue> WriteTrackProperty for Animatable<T> {
    fn write_mdx_track_property(
        &self,
        tag: Tag,
        encoder: &mut Encoder<'_>,
    ) -> Result<(), WriteError> {
        if let Some(track) = self.track() {
            encoder.write_bytes(&tag);
            encoder.write(track)?;
        }
        Ok(())
    }
}
impl<T: TrackValue> ReadTrackProperty for Option<Track<T>> {
    fn read_mdx_track_property(&mut self, cursor: &mut Cursor<'_>) -> Result<(), ReadError> {
        *self = Some(cursor.read()?);
        Ok(())
    }
}
impl<T: TrackValue> WriteTrackProperty for Option<Track<T>> {
    fn write_mdx_track_property(
        &self,
        tag: Tag,
        encoder: &mut Encoder<'_>,
    ) -> Result<(), WriteError> {
        if let Some(track) = self {
            encoder.write_bytes(&tag);
            encoder.write(track)?;
        }
        Ok(())
    }
}
