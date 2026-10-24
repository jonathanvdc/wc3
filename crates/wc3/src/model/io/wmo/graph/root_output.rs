//! Destination-selected native chunk sinks for shared root records.
use super::encode::array;
use crate::model::conversion::Error;
use crate::model::error::Location;
use crate::model::formats::wmo::legacy::{Light14, Root14, Root16, RootChunk14};
use crate::model::formats::wmo::root::RootChunk;
use crate::model::formats::wmo::{Classic, Dialect, Root};
use std::error::Error as StdError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Addressing {
    Paths,
    PathsOrFileIds,
}
pub(super) trait RootOutput<'a, D: Dialect> {
    fn addressing(&self) -> Addressing {
        Addressing::PathsOrFileIds
    }
    fn append(&mut self, chunk: RootChunk<'a, D>) -> Result<(), Error>;
    fn actor_counts(&mut self, counts: [u32; 4]);
    fn portal_count(&mut self, count: u32);
    fn material_count(&mut self, count: u32);
}
impl<'a, D: Dialect> RootOutput<'a, D> for Root<'a, D> {
    fn append(&mut self, chunk: RootChunk<'a, D>) -> Result<(), Error> {
        self.insert_chunk(self.chunks().count(), chunk)
            .map_err(write_error)
    }
    fn actor_counts(&mut self, [lights, doodads, names, sets]: [u32; 4]) {
        let header = self.header_mut();
        header.light_count = lights;
        header.doodad_def_count = doodads;
        header.doodad_name_count = names;
        header.doodad_set_count = sets;
    }
    fn portal_count(&mut self, count: u32) {
        self.header_mut().portal_count = count;
    }
    fn material_count(&mut self, count: u32) {
        self.header_mut().texture_count = count;
    }
}
impl<'a> RootOutput<'a, Classic> for Root16<'a> {
    fn addressing(&self) -> Addressing {
        Addressing::Paths
    }
    fn append(&mut self, chunk: RootChunk<'a, Classic>) -> Result<(), Error> {
        self.push_chunk(chunk).map_err(write_error)
    }
    fn actor_counts(&mut self, [lights, doodads, names, sets]: [u32; 4]) {
        let header = self.header_mut();
        header.light_count = lights;
        header.doodad_def_count = doodads;
        header.doodad_name_count = names;
        header.doodad_set_count = sets;
    }
    fn portal_count(&mut self, count: u32) {
        self.header_mut().portal_count = count;
    }
    fn material_count(&mut self, count: u32) {
        self.header_mut().texture_count = count;
    }
}
impl<'a> RootOutput<'a, Classic> for Root14<'a> {
    fn addressing(&self) -> Addressing {
        Addressing::Paths
    }
    fn append(&mut self, chunk: RootChunk<'a, Classic>) -> Result<(), Error> {
        let chunk = match chunk {
            RootChunk::Lights(rows) => {
                let rows = rows
                    .iter()
                    .map(|row| {
                        row.map(|row| Light14 {
                            kind: row.kind,
                            use_attenuation: row.use_attenuation,
                            reserved: row.padding,
                            color: row.color,
                            position: row.position,
                            intensity: row.intensity,
                            attenuation_start: row.attenuation_start,
                            attenuation_end: row.attenuation_end,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(write_error)?;
                RootChunk14::Lights(array(&rows)?)
            }
            RootChunk::Materials(_) | RootChunk::Groups(_) => {
                return Err(Error::requirement(
                    Location::new().field("world").field("root"),
                    "historical tables require their own logical record writer",
                ));
            }
            chunk => RootChunk14::Common(chunk),
        };
        self.push_chunk(chunk).map_err(write_error)
    }
    fn actor_counts(&mut self, [lights, doodads, names, sets]: [u32; 4]) {
        let header = self.header_mut();
        header.light_count = lights;
        header.doodad_definition_count = doodads;
        header.doodad_name_count = names;
        header.doodad_set_count = sets;
    }
    fn portal_count(&mut self, count: u32) {
        self.header_mut().portal_count = count;
    }
    fn material_count(&mut self, count: u32) {
        self.header_mut().texture_count = count;
    }
}
fn write_error(error: impl StdError + Send + Sync + 'static) -> Error {
    Error::from_source(Location::new().field("world").field("root"), error)
}
