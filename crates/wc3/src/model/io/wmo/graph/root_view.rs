//! Borrowed access to fields shared by native root envelopes.
use crate::model::formats::wmo::legacy::{Root14, Root16, RootChunk14};
use crate::model::formats::wmo::root::RootChunk;
use crate::model::formats::wmo::{records::Light, ReadError};
use crate::model::formats::wmo::{Classic, Dialect, Root};
/// This view only borrows existing source records; it constructs no upgraded document.
pub(super) trait CommonRoot<'data> {
    type Dialect: Dialect;
    fn lights<'view>(&'view self) -> Result<Vec<Light>, ReadError>
    where
        'data: 'view,
    {
        self.common_chunks()
            .filter_map(|chunk| match chunk {
                RootChunk::Lights(rows) => Some(rows),
                _ => None,
            })
            .flat_map(|rows| rows.iter())
            .collect()
    }
    fn common_chunks<'view>(
        &'view self,
    ) -> impl Iterator<Item = &'view RootChunk<'data, Self::Dialect>>
    where
        'data: 'view;
}
impl<'data, D: Dialect> CommonRoot<'data> for Root<'data, D> {
    type Dialect = D;
    fn common_chunks<'view>(&'view self) -> impl Iterator<Item = &'view RootChunk<'data, D>>
    where
        'data: 'view,
    {
        self.chunks()
    }
}
impl<'data> CommonRoot<'data> for Root16<'data> {
    type Dialect = Classic;
    fn common_chunks<'view>(&'view self) -> impl Iterator<Item = &'view RootChunk<'data, Classic>>
    where
        'data: 'view,
    {
        self.chunks()
    }
}
impl<'data> CommonRoot<'data> for Root14<'data> {
    type Dialect = Classic;
    fn lights<'view>(&'view self) -> Result<Vec<Light>, ReadError>
    where
        'data: 'view,
    {
        self.chunks()
            .filter_map(|chunk| match chunk {
                RootChunk14::Lights(rows) => Some(rows),
                _ => None,
            })
            .flat_map(|rows| rows.iter())
            .map(|row| {
                row.map(|row| Light {
                    kind: row.kind,
                    use_attenuation: row.use_attenuation,
                    padding: row.reserved,
                    color: row.color,
                    position: row.position,
                    intensity: row.intensity,
                    rotation: [0., 0., 0., 1.],
                    attenuation_start: row.attenuation_start,
                    attenuation_end: row.attenuation_end,
                })
            })
            .collect()
    }
    fn common_chunks<'view>(&'view self) -> impl Iterator<Item = &'view RootChunk<'data, Classic>>
    where
        'data: 'view,
    {
        self.chunks().filter_map(|chunk| match chunk {
            RootChunk14::Common(chunk) => Some(chunk),
            _ => None,
        })
    }
}
