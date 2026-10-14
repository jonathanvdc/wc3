//! Destination-selected sinks for common group records and actor selections.
use crate::model::conversion::Error;
use crate::model::wmo::group::GroupChunk;
use crate::model::wmo::legacy::{Group14, Group16, LegacyGroupChunk};
use crate::model::wmo::{Classic, Dialect, Group};
pub(super) trait GroupOutput<'a, D: Dialect> {
    fn push(&mut self, chunk: GroupChunk<'a, D>);
    fn actor_flags(&mut self, lights: bool, doodads: bool);
    fn portal_window(&mut self, start: usize, count: usize) -> Result<(), Error>;
}
impl<'a, D: Dialect> GroupOutput<'a, D> for Group<'a, D> {
    fn push(&mut self, chunk: GroupChunk<'a, D>) {
        self.chunks_mut().push(chunk);
    }
    fn actor_flags(&mut self, lights: bool, doodads: bool) {
        self.header_mut().flags.set_has_lights(lights);
        self.header_mut().flags.set_has_doodads(doodads);
    }
    fn portal_window(&mut self, start: usize, count: usize) -> Result<(), Error> {
        self.header_mut().portal_start = u16::try_from(start).map_err(error)?;
        self.header_mut().portal_count = u16::try_from(count).map_err(error)?;
        Ok(())
    }
}
impl<'a> GroupOutput<'a, Classic> for Group14<'a> {
    fn push(&mut self, chunk: GroupChunk<'a, Classic>) {
        self.chunks.push(LegacyGroupChunk::Common(chunk));
    }
    fn actor_flags(&mut self, lights: bool, doodads: bool) {
        self.header.flags.set_has_lights(lights);
        self.header.flags.set_has_doodads(doodads);
    }
    fn portal_window(&mut self, start: usize, count: usize) -> Result<(), Error> {
        self.header.portal_start = i32::try_from(start).map_err(error)?;
        self.header.portal_count = u32::try_from(count).map_err(error)?;
        Ok(())
    }
}
impl<'a> GroupOutput<'a, Classic> for Group16<'a> {
    fn push(&mut self, chunk: GroupChunk<'a, Classic>) {
        self.chunks.push(LegacyGroupChunk::Common(chunk));
    }
    fn actor_flags(&mut self, lights: bool, doodads: bool) {
        self.header.flags.set_has_lights(lights);
        self.header.flags.set_has_doodads(doodads);
    }
    fn portal_window(&mut self, start: usize, count: usize) -> Result<(), Error> {
        self.header.portal_start = i16::try_from(start).map_err(error)?;
        self.header.portal_count = u16::try_from(count).map_err(error)?;
        Ok(())
    }
}
fn error(error: impl std::fmt::Display) -> Error {
    Error::new("world.group.portal", error.to_string())
}
