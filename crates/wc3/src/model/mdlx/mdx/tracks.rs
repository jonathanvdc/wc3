//! Track dispatch for fields in a sized record's animation tail.
use super::{Cursor, Encoder};
use crate::model::mdlx::mdx;
use crate::model::mdlx::{Animatable, Tag, Track, TrackValue};

/// Reads one recognized animation payload, after the enclosing record reads its tag.
pub trait ReadTracks<'data> {
    /// Reads the payload for `tag`, returning whether this record recognizes it.
    /// An unrecognized tag returns `false` without consuming its payload.
    fn read_mdx_track(
        &mut self,
        tag: Tag,
        cursor: &mut Cursor<'data>,
    ) -> Result<bool, mdx::ReadError>;
}
/// Writes animation tails in declaration order, including nested field groups.
pub trait WriteTracks {
    /// Appends all present tracks, including their tags, in field order.
    fn write_mdx_tracks(&self, encoder: &mut Encoder<'_>) -> Result<(), mdx::WriteError>;
}
/// Field storage that accepts a replacement animation.
pub trait ReadTrackProperty<'data> {
    /// Reads a track payload and replaces the stored animation.
    /// The enclosing record has already consumed its tag.
    fn read_mdx_track_property(&mut self, cursor: &mut Cursor<'data>)
        -> Result<(), mdx::ReadError>;
}
/// Field storage that can emit an optional animation.
pub trait WriteTrackProperty {
    /// Appends `tag` and its track payload when animation is present.
    fn write_mdx_track_property(
        &self,
        tag: Tag,
        encoder: &mut Encoder<'_>,
    ) -> Result<(), mdx::WriteError>;
}
impl<'data, T: TrackValue> ReadTrackProperty<'data> for Animatable<'data, T> {
    fn read_mdx_track_property(
        &mut self,
        cursor: &mut Cursor<'data>,
    ) -> Result<(), mdx::ReadError> {
        self.set_track(cursor.read()?);
        Ok(())
    }
}
impl<'data, T: TrackValue> WriteTrackProperty for Animatable<'data, T> {
    fn write_mdx_track_property(
        &self,
        tag: Tag,
        encoder: &mut Encoder<'_>,
    ) -> Result<(), mdx::WriteError> {
        if let Some(track) = self.track() {
            encoder.write_bytes(&tag);
            encoder.write(track)?;
        }
        Ok(())
    }
}
impl<'data, T: TrackValue> ReadTrackProperty<'data> for Option<Track<'data, T>> {
    fn read_mdx_track_property(
        &mut self,
        cursor: &mut Cursor<'data>,
    ) -> Result<(), mdx::ReadError> {
        *self = Some(cursor.read()?);
        Ok(())
    }
}
impl<'data, T: TrackValue> WriteTrackProperty for Option<Track<'data, T>> {
    fn write_mdx_track_property(
        &self,
        tag: Tag,
        encoder: &mut Encoder<'_>,
    ) -> Result<(), mdx::WriteError> {
        if let Some(track) = self {
            encoder.write_bytes(&tag);
            encoder.write(track)?;
        }
        Ok(())
    }
}
