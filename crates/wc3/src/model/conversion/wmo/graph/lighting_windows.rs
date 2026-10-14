//! Historical lighting selections are relocated with joined vertex and draw arrays.
use crate::model::conversion::Error;

type LightingWindows = Option<[[u16; 4]; 4]>;

#[derive(Default)]
pub(super) struct Windows {
    interior: Option<Vec<[u16; 4]>>,
    exterior: Option<Vec<[u16; 4]>>,
}
impl Windows {
    pub(super) fn append(
        &mut self,
        interior: Option<[[u16; 4]; 4]>,
        exterior: Option<[[u16; 4]; 4]>,
        vertices: u32,
        draws: u32,
    ) -> Result<(), Error> {
        append(&mut self.interior, interior, vertices, draws)?;
        append(&mut self.exterior, exterior, vertices, draws)
    }
    pub(super) fn finish(self) -> Result<(LightingWindows, LightingWindows), Error> {
        Ok((finish(self.interior)?, finish(self.exterior)?))
    }
}
fn append(
    target: &mut Option<Vec<[u16; 4]>>,
    source: Option<[[u16; 4]; 4]>,
    vertices: u32,
    draws: u32,
) -> Result<(), Error> {
    let Some(source) = source else { return Ok(()) };
    let target = target.get_or_insert_with(Vec::new);
    for [vertex, count, batch, batches] in source {
        if count == 0 && batches == 0 {
            continue;
        }
        let vertex =
            if count == 0 {
                0
            } else {
                u16::try_from(vertices.checked_add(u32::from(vertex)).ok_or_else(|| {
                    Error::new("world.group.lighting", "vertex selection overflows")
                })?)
                .map_err(error)?
            };
        let batch =
            if batches == 0 {
                0
            } else {
                u16::try_from(draws.checked_add(u32::from(batch)).ok_or_else(|| {
                    Error::new("world.group.lighting", "batch selection overflows")
                })?)
                .map_err(error)?
            };
        if let Some(previous) = target.last_mut() {
            let adjacent = (count == 0 && previous[1] == 0
                || u32::from(previous[0]) + u32::from(previous[1]) == u32::from(vertex))
                && (batches == 0 && previous[3] == 0
                    || u32::from(previous[2]) + u32::from(previous[3]) == u32::from(batch));
            if adjacent {
                if let (Some(vertices), Some(draws)) = (
                    previous[1].checked_add(count),
                    previous[3].checked_add(batches),
                ) {
                    previous[1] = vertices;
                    previous[3] = draws;
                    continue;
                }
            }
        }
        target.push([vertex, count, batch, batches]);
    }
    Ok(())
}
fn finish(source: Option<Vec<[u16; 4]>>) -> Result<Option<[[u16; 4]; 4]>, Error> {
    source
        .map(|source| {
            if source.len() > 4 {
                return Err(Error::new(
                    "world.group.lighting",
                    "joined geometry needs more than four disjoint historical lighting windows",
                ));
            }
            let mut target = [[0; 4]; 4];
            for (slot, value) in target.iter_mut().zip(source) {
                *slot = value;
            }
            Ok(target)
        })
        .transpose()
}
fn error(value: impl std::fmt::Display) -> Error {
    Error::new("world.group.lighting", value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adjacent_historical_selections_coalesce_after_relocation() {
        let mut windows = Windows::default();
        let selection = Some([[0, 3, 0, 1], [0; 4], [0; 4], [0; 4]]);
        windows.append(selection, None, 0, 0).unwrap();
        windows.append(selection, None, 3, 1).unwrap();
        let (interior, exterior) = windows.finish().unwrap();
        assert_eq!(interior.unwrap(), [[0, 6, 0, 2], [0; 4], [0; 4], [0; 4]]);
        assert_eq!(exterior, None);
    }
}
