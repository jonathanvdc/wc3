//! Portal polygons, authored separating planes, and historical visibility regions.
use crate::model::conversion::materialize;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wmo as profile;

use crate::model::wmo::root::RootChunk;
use crate::model::wmo::Dialect;
use zerovec::ZeroVec;
pub(super) fn read<'a, 'data: 'a>(
    root: &'a impl super::root_view::CommonRoot<'data>,
    world: &mut profile::WorldSystem<'a>,
    _owner: crate::model::semantic::SystemId,
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
                let row = row.map_err(|error| Error::new("world.portal", error.to_string()))?;
                let source = vertices
                    .ok_or_else(|| Error::new("world.portal", "portal vertices are missing"))?;
                let start = usize::from(row.start_vertex);
                let count = usize::from(row.vertex_count);
                if start + count > source.len() {
                    return Err(Error::new(
                        "world.portal",
                        "vertex range exceeds portal table",
                    ));
                }
                world.portals.push(profile::Portal {
                    vertices: materialize(count, move |index| {
                        source
                            .get(start + index)
                            .expect("checked portal index")
                            .expect("validated portal vertex")
                    }),
                    plane: row.plane,
                    adjustment: None,
                });
            }
        }
    }
    for chunk in root.common_chunks() {
        if let RootChunk::PortalExtras(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(|error| Error::new("world.portal", error.to_string()))?;
                let portal = world
                    .portals
                    .get_mut(row.portal_index as usize)
                    .ok_or_else(|| {
                        Error::new("world.portal", "adjustment references absent portal")
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
                    let row =
                        row.map_err(|error| Error::new("world.visibility", error.to_string()))?;
                    let source = visible.ok_or_else(|| {
                        Error::new("world.visibility", "visibility vertices are missing")
                    })?;
                    let start = usize::from(row.first_vertex);
                    let count = usize::from(row.count);
                    if start + count > source.len() {
                        return Err(Error::new(
                            "world.visibility",
                            "visibility range exceeds table",
                        ));
                    }
                    world.visibility.push(materialize(count, move |index| {
                        source
                            .get(start + index)
                            .expect("checked visibility vertex")
                            .expect("validated visibility vertex")
                    }));
                }
            }
            RootChunk::ConvexPlanes(rows) => {
                world.occluders = ZeroVec::parse_bytes(rows.as_bytes())
                    .map_err(|error| Error::new("world.occluders", error.to_string()))?
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
    use crate::model::wmo::records::{Portal, PortalExtra, VisibleBlock};
    let capacity = |value: usize| {
        u16::try_from(value)
            .map_err(|_| Error::new("world.visibility", "vertex range exceeds native capacity"))
    };
    let mut vertices = Vec::new();
    let mut portals = Vec::new();
    let mut extras = Vec::new();
    for (index, portal) in source.portals.iter().enumerate() {
        portals.push(Portal {
            start_vertex: capacity(vertices.len())?,
            vertex_count: capacity(portal.vertices.len())?,
            plane: portal.plane,
        });
        vertices.extend(portal.vertices.iter());
        if let Some(unknown) = portal.adjustment {
            extras.push(PortalExtra {
                portal_index: u32::try_from(index)
                    .map_err(|_| Error::new("world.portal", "portal count exceeds capacity"))?,
                unknown,
            });
        }
    }
    if !portals.is_empty() {
        super::encode::append(
            root,
            RootChunk::PortalVertices(super::encode::array(&vertices)?),
        )?;
        super::encode::append(root, RootChunk::Portals(super::encode::array(&portals)?))?;
    }
    if !extras.is_empty() {
        super::encode::append(
            root,
            RootChunk::PortalExtras(super::encode::array(&extras)?),
        )?;
    }
    root.portal_count(
        u32::try_from(portals.len())
            .map_err(|_| Error::new("world.portal", "portal count exceeds capacity"))?,
    );
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
            RootChunk::VisibleVertices(super::encode::array(&visible)?),
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
                &source.occluders.iter().collect::<Vec<_>>(),
            )?),
        )?;
    }
    Ok(())
}
