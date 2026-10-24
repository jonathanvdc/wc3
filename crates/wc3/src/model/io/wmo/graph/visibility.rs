//! Portal polygons, authored separating planes, and historical visibility regions.
use crate::model::common::PackedArray;
use crate::model::conversion::Error;
use crate::model::error::Location;
use crate::model::io::materialize;
use crate::model::profiles::wmo as profile;
use glam::Vec4;

use crate::model::formats::wmo::root::RootChunk;
use crate::model::formats::wmo::Dialect;
use zerovec::ZeroVec;
pub(super) fn read<'a, 'data: 'a>(
    root: &'a impl super::root_view::CommonRoot<'data>,
    world: &mut profile::WorldSystem<'a>,
    _owner: crate::model::SystemId,
) -> Result<(), Error> {
    let vertices = root.common_chunks().find_map(|chunk| {
        if let RootChunk::PortalVertices(values) = chunk {
            Some(values)
        } else {
            None
        }
    });
    for chunk in root.common_chunks() {
        if let RootChunk::Portals(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(|error| {
                    Error::from_source(Location::new().field("world").field("portal"), error)
                })?;
                let source = vertices.ok_or_else(|| {
                    Error::requirement(
                        Location::new().field("world").field("portal"),
                        "portal vertices are missing",
                    )
                })?;
                let start = usize::from(row.start_vertex);
                let count = usize::from(row.vertex_count);
                if start + count > source.len() {
                    return Err(Error::requirement(
                        Location::new().field("world").field("portal"),
                        "vertex range exceeds portal table",
                    ));
                }
                world.portals.push(profile::Portal {
                    vertices: PackedArray::from_storage(
                        (materialize(count, move |index| {
                            source
                                .get(start + index)
                                .expect("checked portal index")
                                .expect("validated portal vertex")
                        }))
                        .into_storage(),
                    ),
                    plane: Vec4::from_array(row.plane),
                    adjustment: None,
                });
            }
        }
    }
    for chunk in root.common_chunks() {
        if let RootChunk::PortalExtras(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(|error| {
                    Error::from_source(Location::new().field("world").field("portal"), error)
                })?;
                let portal = world
                    .portals
                    .get_mut(row.portal_index as usize)
                    .ok_or_else(|| {
                        Error::requirement(
                            Location::new().field("world").field("portal"),
                            "adjustment references absent portal",
                        )
                    })?;
                portal.adjustment = Some(row.unknown);
            }
        }
    }
    let visible = root.common_chunks().find_map(|chunk| {
        if let RootChunk::VisibleVertices(values) = chunk {
            Some(values)
        } else {
            None
        }
    });
    for chunk in root.common_chunks() {
        match chunk {
            RootChunk::VisibleBlocks(rows) => {
                for row in rows.iter() {
                    let row = row.map_err(|error| {
                        Error::from_source(
                            Location::new().field("world").field("visibility"),
                            error,
                        )
                    })?;
                    let source = visible.ok_or_else(|| {
                        Error::requirement(
                            Location::new().field("world").field("visibility"),
                            "visibility vertices are missing",
                        )
                    })?;
                    let start = usize::from(row.first_vertex);
                    let count = usize::from(row.count);
                    if start + count > source.len() {
                        return Err(Error::requirement(
                            Location::new().field("world").field("visibility"),
                            "visibility range exceeds table",
                        ));
                    }
                    world.visibility.push(PackedArray::from_storage(
                        (materialize(count, move |index| {
                            source
                                .get(start + index)
                                .expect("checked visibility vertex")
                                .expect("validated visibility vertex")
                        }))
                        .into_storage(),
                    ));
                }
            }
            RootChunk::ConvexPlanes(rows) => {
                world.occluders = PackedArray::from_storage(
                    ZeroVec::parse_bytes(rows.as_bytes()).map_err(|error| {
                        Error::from_source(Location::new().field("world").field("occluders"), error)
                    })?,
                )
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn write<'a, D: Dialect>(
    source: &profile::WorldSystem<'_>,
    root: &mut impl super::root_output::RootOutput<'a, D>,
) -> Result<(), Error> {
    use crate::model::formats::wmo::records::{Portal, PortalExtra, VisibleBlock};
    let capacity = |value: usize| {
        u16::try_from(value).map_err(|_| {
            Error::requirement(
                Location::new().field("world").field("visibility"),
                "vertex range exceeds native capacity",
            )
        })
    };
    let mut vertices = Vec::new();
    let mut portals = Vec::new();
    let mut extras = Vec::new();
    for (index, portal) in source.portals.iter().enumerate() {
        portals.push(Portal {
            start_vertex: capacity(vertices.len())?,
            vertex_count: capacity(portal.vertices.len())?,
            plane: (portal.plane).to_array(),
        });
        vertices.extend(portal.vertices.iter());
        if let Some(unknown) = portal.adjustment {
            extras.push(PortalExtra {
                portal_index: u32::try_from(index).map_err(|_| {
                    Error::requirement(
                        Location::new().field("world").field("portal"),
                        "portal count exceeds capacity",
                    )
                })?,
                unknown,
            });
        }
    }
    if !portals.is_empty() {
        super::encode::append(
            root,
            RootChunk::PortalVertices(super::encode::array(
                &vertices.iter().map(|v| v.to_array()).collect::<Vec<_>>(),
            )?),
        )?;
        super::encode::append(root, RootChunk::Portals(super::encode::array(&portals)?))?;
    }
    if !extras.is_empty() {
        super::encode::append(
            root,
            RootChunk::PortalExtras(super::encode::array(&extras)?),
        )?;
    }
    root.portal_count(u32::try_from(portals.len()).map_err(|_| {
        Error::requirement(
            Location::new().field("world").field("portal"),
            "portal count exceeds capacity",
        )
    })?);
    let mut visible = Vec::new();
    let mut blocks = Vec::new();
    for polygon in &source.visibility {
        blocks.push(VisibleBlock {
            first_vertex: capacity(visible.len())?,
            count: capacity(polygon.len())?,
        });
        visible.extend(polygon.iter());
    }
    if !blocks.is_empty() {
        super::encode::append(
            root,
            RootChunk::VisibleVertices(super::encode::array(
                &visible.iter().map(|v| v.to_array()).collect::<Vec<_>>(),
            )?),
        )?;
        super::encode::append(
            root,
            RootChunk::VisibleBlocks(super::encode::array(&blocks)?),
        )?;
    }
    if !source.occluders.is_empty() {
        super::encode::append(
            root,
            RootChunk::ConvexPlanes(super::encode::array(
                &source.occluders.storage().iter().collect::<Vec<_>>(),
            )?),
        )?;
    }
    Ok(())
}
