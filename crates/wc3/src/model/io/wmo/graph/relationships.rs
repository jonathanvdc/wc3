//! Group names and scoped scene/portal relationships.
use super::scene::Identities;
use crate::model::conversion::Error;
use crate::model::formats::wmo::group::GroupChunk;
use crate::model::formats::wmo::root::RootChunk;
use crate::model::formats::wmo::{Dialect, Group, Root};
use crate::model::profiles::wmo as profile;
use std::borrow::Cow;
pub(super) fn read<'a, D: Dialect>(
    root: &'a Root<'_, D>,
    source: &'a Group<'_, D>,
    group: &mut profile::Region<'a>,
    ids: &Identities,
    owner: crate::model::SystemId,
) -> Result<(), Error> {
    read_fields(
        root,
        source.chunks().iter(),
        References {
            fogs: source.header().fog_indices,
            portal_start: usize::from(source.header().portal_start),
            portal_count: usize::from(source.header().portal_count),
        },
        group,
        ids,
        owner,
    )
}

pub(super) struct References {
    pub fogs: [u8; 4],
    pub portal_start: usize,
    pub portal_count: usize,
}

pub(super) fn read_fields<'a, 'data: 'a, D: Dialect>(
    root: &'a impl super::root_view::CommonRoot<'data, Dialect = D>,
    chunks: impl Iterator<Item = &'a GroupChunk<'data, D>> + Clone,
    references: References,
    group: &mut profile::Region<'a>,
    ids: &Identities,
    owner: crate::model::SystemId,
) -> Result<(), Error> {
    let mut ambient_index = 0u32;
    let mut radial_ambients = Vec::new();
    let mut box_ambients = Vec::new();
    for chunk in root.common_chunks() {
        match chunk {
            RootChunk::AmbientVolumes(rows) => {
                for _ in 0..rows.len() {
                    radial_ambients.push(ambient_index);
                    ambient_index = ambient_index.checked_add(1).ok_or_else(|| {
                        Error::new("world.ambient", "ambient identity capacity exceeded")
                    })?;
                }
            }
            RootChunk::GlobalAmbients(rows) => {
                ambient_index = ambient_index
                    .checked_add(
                        u32::try_from(rows.len())
                            .map_err(|error| Error::new("world.ambient", error.to_string()))?,
                    )
                    .ok_or_else(|| {
                        Error::new("world.ambient", "ambient identity capacity exceeded")
                    })?
            }
            RootChunk::AmbientBoxes(rows) => {
                for _ in 0..rows.len() {
                    box_ambients.push(ambient_index);
                    ambient_index = ambient_index.checked_add(1).ok_or_else(|| {
                        Error::new("world.ambient", "ambient identity capacity exceeded")
                    })?;
                }
            }
            _ => {}
        }
    }
    for chunk in chunks.clone() {
        match chunk {
            GroupChunk::DoodadReferences(values) => {
                for row in values.iter() {
                    let index =
                        row.map_err(|error| Error::new("world.group.doodad", error.to_string()))?;
                    group
                        .doodads
                        .push(*ids.doodads.get(usize::from(index)).ok_or_else(|| {
                            Error::new("world.group.doodad", "reference points to absent placement")
                        })?);
                }
            }
            GroupChunk::LightReferences(values) => {
                for row in values.iter() {
                    let index =
                        row.map_err(|error| Error::new("world.group.light", error.to_string()))?;
                    group
                        .lights
                        .push(*ids.lights.get(usize::from(index)).ok_or_else(|| {
                            Error::new("world.group.light", "reference points to absent light")
                        })?);
                }
            }
            GroupChunk::NewLightReferences(values) => {
                for row in values.iter() {
                    let index =
                        row.map_err(|error| Error::new("world.group.light", error.to_string()))?;
                    group
                        .lights
                        .push(*ids.new_lights.get(usize::from(index)).ok_or_else(|| {
                            Error::new(
                                "world.group.light",
                                "reference points to absent modern light",
                            )
                        })?);
                }
            }
            GroupChunk::AmbientVolumeReferences(values)
            | GroupChunk::AmbientBoxReferences(values) => {
                let mapping = if matches!(chunk, GroupChunk::AmbientVolumeReferences(_)) {
                    &radial_ambients
                } else {
                    &box_ambients
                };
                for row in values.iter() {
                    let index =
                        usize::from(row.map_err(|error| {
                            Error::new("world.group.ambient", error.to_string())
                        })?);
                    let index = *mapping.get(index).ok_or_else(|| {
                        Error::new(
                            "world.group.ambient",
                            "ambient reference points to absent volume",
                        )
                    })?;
                    group.ambients.push(profile::AmbientId {
                        system: owner,
                        index,
                    });
                }
            }
            GroupChunk::FogReferences(values) => {
                group.fogs = values
                    .iter()
                    .map(|row| {
                        row.map(|index| profile::FogId {
                            system: owner,
                            index: u32::from(index),
                        })
                        .map_err(|error| Error::new("world.group.fog", error.to_string()))
                    })
                    .collect::<Result<_, _>>()?
            }
            _ => {}
        }
    }
    if group.fogs.is_empty() {
        group.fogs = references
            .fogs
            .iter()
            .filter(|index| **index != u8::MAX)
            .map(|index| profile::FogId {
                system: owner,
                index: u32::from(*index),
            })
            .collect();
    }
    let sphere_count = root
        .common_chunks()
        .filter_map(|chunk| match chunk {
            RootChunk::Fogs(rows) => Some(rows.len()),
            _ => None,
        })
        .sum::<usize>();
    for chunk in chunks.clone() {
        if let GroupChunk::FogBoxReferences(rows) = chunk {
            for row in rows.iter() {
                let index = usize::from(
                    row.map_err(|error| Error::new("world.group.fog", error.to_string()))?,
                );
                group.fogs.push(profile::FogId {
                    system: owner,
                    index: u32::try_from(sphere_count + index).map_err(|_| {
                        Error::new("world.group.fog", "fog identity exceeds graph capacity")
                    })?,
                });
            }
        }
    }
    let portals = root.common_chunks().find_map(|chunk| {
        if let RootChunk::PortalRefs(values) = chunk {
            Some(values)
        } else {
            None
        }
    });
    if references.portal_count != 0 {
        let source_refs =
            portals.ok_or_else(|| Error::new("world.group.portal", "references are missing"))?;
        let start = references.portal_start;
        for index in start..start + references.portal_count {
            let row = source_refs
                .get(index)
                .ok_or_else(|| Error::new("world.group.portal", "portal range exceeds table"))?
                .map_err(|error| Error::new("world.group.portal", error.to_string()))?;
            group.portals.push(profile::PortalConnection {
                portal: profile::PortalId {
                    system: owner,
                    index: u32::from(row.portal_index),
                },
                neighbor: profile::GroupId {
                    system: owner,
                    index: u32::from(row.group_index),
                },
                side: row.side,
                skip_viewer: row.flags.skip_viewer_queries(),
                adjusted_plane: row.flags.plane_adjustment(),
            });
        }
    }
    Ok(())
}

pub(super) fn name<'a, 'data: 'a>(
    root: &'a impl super::root_view::CommonRoot<'data>,
    offset: u32,
) -> Result<Cow<'a, str>, Error> {
    let names = root
        .common_chunks()
        .find_map(|chunk| {
            if let RootChunk::GroupNames(values) = chunk {
                Some(values.as_bytes())
            } else {
                None
            }
        })
        .unwrap_or_default();
    if names.is_empty() {
        return Ok(Cow::Borrowed(""));
    }
    let bytes = crate::model::formats::wmo::resources::string_at(names, offset)
        .map_err(|error| Error::new("world.region.name", error.to_string()))?;
    Ok(Cow::Borrowed(std::str::from_utf8(bytes).map_err(
        |error| Error::new("world.region.name", error.to_string()),
    )?))
}

pub(super) fn write<'a, D: Dialect>(
    group: &mut impl super::group_output::GroupOutput<'a, D>,
    source: &profile::Region<'_>,
    world: &profile::WorldSystem<'_>,
    actors: &super::encode_scene::Indices,
    portals: &mut Vec<crate::model::formats::wmo::records::PortalRef>,
) -> Result<(), Error> {
    use crate::model::formats::wmo::records::{PortalFlags, PortalRef};
    let mut lights = Vec::new();
    let mut modern = Vec::new();
    for id in &source.lights {
        if let Some(index) = actors.lights.get(id) {
            lights.push(*index);
        } else if let Some(index) = actors.modern.get(id) {
            modern.push(*index);
        } else {
            return Err(Error::new(
                "world.group.light",
                "region references an unavailable light",
            ));
        }
    }
    let doodads = source
        .doodads
        .iter()
        .map(|id| {
            actors.doodads.get(id).copied().ok_or_else(|| {
                Error::new(
                    "world.group.doodad",
                    "region references an unavailable placement",
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    group.actor_flags(
        !lights.is_empty() || !modern.is_empty(),
        !doodads.is_empty(),
    );
    if !lights.is_empty() {
        group.push(GroupChunk::LightReferences(super::encode::array(&lights)?));
    }
    if !modern.is_empty() {
        group.push(GroupChunk::NewLightReferences(super::encode::array(
            &modern,
        )?));
    }
    if !doodads.is_empty() {
        group.push(GroupChunk::DoodadReferences(super::encode::array(
            &doodads,
        )?));
    }
    let mut radial = Vec::new();
    let mut boxes = Vec::new();
    for ambient in &source.ambients {
        let selected = world
            .ambient_volumes
            .get(ambient.index as usize)
            .ok_or_else(|| {
                Error::new(
                    "world.group.ambient",
                    "ambient reference points to absent volume",
                )
            })?;
        if selected.global {
            return Err(Error::new(
                "world.group.ambient",
                "globally active ambient volumes cannot be selected by individual regions",
            ));
        }
        let index = world.ambient_volumes[..ambient.index as usize]
            .iter()
            .filter(|candidate| {
                !candidate.global
                    && matches!(
                        (&candidate.volume, &selected.volume),
                        (
                            profile::Volume::Sphere { .. },
                            profile::Volume::Sphere { .. }
                        ) | (
                            profile::Volume::Planes { .. },
                            profile::Volume::Planes { .. }
                        )
                    )
            })
            .count();
        let index = u16::try_from(index)
            .map_err(|error| Error::new("world.group.ambient", error.to_string()))?;
        match selected.volume {
            profile::Volume::Sphere { .. } => radial.push(index),
            profile::Volume::Planes { .. } => boxes.push(index),
        }
    }
    if !radial.is_empty() {
        group.push(GroupChunk::AmbientVolumeReferences(super::encode::array(
            &radial,
        )?));
    }
    if !boxes.is_empty() {
        group.push(GroupChunk::AmbientBoxReferences(super::encode::array(
            &boxes,
        )?));
    }
    group.portal_window(portals.len(), source.portals.len())?;
    for source in &source.portals {
        let mut flags = PortalFlags(0);
        flags.set_skip_viewer_queries(source.skip_viewer);
        flags.set_plane_adjustment(source.adjusted_plane);
        portals.push(PortalRef {
            portal_index: u16::try_from(source.portal.index).map_err(|_| {
                Error::new("world.portal", "portal identity exceeds native capacity")
            })?,
            group_index: u16::try_from(source.neighbor.index).map_err(|_| {
                Error::new("world.portal", "group identity exceeds native capacity")
            })?,
            side: source.side,
            flags,
        });
    }
    Ok(())
}
