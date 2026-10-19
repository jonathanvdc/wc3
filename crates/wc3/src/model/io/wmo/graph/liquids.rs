//! Authored liquid grids retain the explicitly selected flow vocabulary.
use crate::model::conversion::Error;
use crate::model::formats::wmo::group::{GroupChunk, LiquidVertices};
use crate::model::formats::wmo::{Dialect, Group};
use crate::model::io::materialize;
use crate::model::profiles::wmo as profile;
use crate::model::MaterialId;
use glam::Vec3;
pub(super) fn read<'a, D: Dialect>(
    source: &'a Group<'_, D>,
    materials: &[MaterialId],
    database_ids: bool,
) -> Result<Vec<profile::Liquid<'a>>, Error> {
    let mut result = Vec::new();
    for chunk in source.chunks() {
        if let GroupChunk::Liquid(liquid) = chunk {
            let dimensions = liquid
                .header
                .vertex_dimensions
                .map(|value| u32::try_from(value).expect("validated nonnegative liquid dimension"));
            let vertices = match &liquid.vertices {
                LiquidVertices::Water(rows) => materialize(rows.len(), move |index| {
                    let row = rows
                        .get(index)
                        .expect("checked liquid vertex")
                        .expect("validated liquid vertex");
                    profile::LiquidVertex::Water {
                        height: row.height,
                        velocities: [row.flow_1, row.flow_2],
                        first_percentage: row.flow_1_percentage,
                        control: row.filler,
                    }
                }),
                LiquidVertices::Magma(rows) => materialize(rows.len(), move |index| {
                    let row = rows
                        .get(index)
                        .expect("checked liquid vertex")
                        .expect("validated liquid vertex");
                    profile::LiquidVertex::Magma {
                        height: row.height,
                        coordinates: row.texture_coordinates,
                    }
                }),
                LiquidVertices::Unresolved(rows) => materialize(rows.len(), move |index| {
                    let row = rows
                        .get(index)
                        .expect("checked liquid vertex")
                        .expect("validated liquid vertex");
                    profile::LiquidVertex::Unresolved {
                        height: row.height,
                        controls: [
                            u16::from_le_bytes([row.data[0], row.data[1]]),
                            u16::from_le_bytes([row.data[2], row.data[3]]),
                        ],
                    }
                }),
            };
            let material = if database_ids {
                None
            } else {
                Some(
                    *materials
                        .get(usize::from(liquid.header.material))
                        .ok_or_else(|| {
                            Error::new("world.liquid.material", "liquid material does not exist")
                        })?,
                )
            };
            let tiles = materialize(liquid.tiles.len(), move |index| {
                liquid
                    .tiles
                    .get(index)
                    .expect("checked liquid tile")
                    .expect("validated liquid tile")
            });
            result.push(profile::Liquid {
                position: Vec3::from_array(liquid.header.corner),
                dimensions,
                vertices,
                tiles,
                material,
                liquid_type: if database_ids {
                    u32::from(liquid.header.material)
                } else {
                    source.header().liquid_type
                },
            });
        }
    }
    Ok(result)
}

pub(super) fn write<'a>(
    source: &profile::Liquid<'_>,
    materials: &std::collections::BTreeMap<MaterialId, u16>,
    database_ids: bool,
) -> Result<crate::model::formats::wmo::group::Liquid<'a>, Error> {
    use crate::model::formats::wmo::group::{
        Liquid, LiquidHeader, LiquidVertex, MagmaVertex, WaterVertex,
    };
    let dimensions = [
        i32::try_from(source.dimensions[0]),
        i32::try_from(source.dimensions[1]),
    ];
    let [x, y] = dimensions;
    let dimensions = [
        x.map_err(|_| {
            Error::new(
                "world.liquid.dimensions",
                "grid exceeds native dimension capacity",
            )
        })?,
        y.map_err(|_| {
            Error::new(
                "world.liquid.dimensions",
                "grid exceeds native dimension capacity",
            )
        })?,
    ];
    let material = if database_ids {
        u16::try_from(source.liquid_type)
            .map_err(|_| Error::new("world.liquid.type", "liquid type exceeds native capacity"))?
    } else {
        let id = source.material.ok_or_else(|| {
            Error::new(
                "world.liquid.material",
                "material-addressed liquid requires a material",
            )
        })?;
        *materials
            .get(&id)
            .ok_or_else(|| Error::new("world.liquid.material", "liquid material is unavailable"))?
    };
    let rows = source.vertices.iter().collect::<Vec<_>>();
    let vertices = match rows.first() {
        Some(profile::LiquidVertex::Water { .. }) => {
            let rows = rows
                .iter()
                .map(|row| match row {
                    profile::LiquidVertex::Water {
                        height,
                        velocities,
                        first_percentage,
                        control,
                    } => Ok(WaterVertex {
                        height: *height,
                        flow_1: velocities[0],
                        flow_2: velocities[1],
                        flow_1_percentage: *first_percentage,
                        filler: *control,
                    }),
                    _ => Err(Error::new(
                        "world.liquid.vertices",
                        "one liquid grid cannot mix union interpretations",
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            LiquidVertices::Water(super::encode::array(&rows)?)
        }
        Some(profile::LiquidVertex::Magma { .. }) => {
            let rows = rows
                .iter()
                .map(|row| match row {
                    profile::LiquidVertex::Magma {
                        height,
                        coordinates,
                    } => Ok(MagmaVertex {
                        height: *height,
                        texture_coordinates: *coordinates,
                    }),
                    _ => Err(Error::new(
                        "world.liquid.vertices",
                        "one liquid grid cannot mix union interpretations",
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            LiquidVertices::Magma(super::encode::array(&rows)?)
        }
        _ => {
            let rows = rows
                .iter()
                .map(|row| match row {
                    profile::LiquidVertex::Unresolved { height, controls } => {
                        let a = controls[0].to_le_bytes();
                        let b = controls[1].to_le_bytes();
                        Ok(LiquidVertex {
                            height: *height,
                            data: [a[0], a[1], b[0], b[1]],
                        })
                    }
                    _ => Err(Error::new(
                        "world.liquid.vertices",
                        "one liquid grid cannot mix union interpretations",
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            LiquidVertices::Unresolved(super::encode::array(&rows)?)
        }
    };
    Ok(Liquid {
        header: LiquidHeader {
            vertex_dimensions: dimensions,
            tile_dimensions: dimensions.map(|value| value.saturating_sub(1)),
            corner: (source.position).to_array(),
            material,
        },
        vertices,
        tiles: super::encode::array(&source.tiles.iter().collect::<Vec<_>>())?,
    })
}

/// Resolves the group-level category independently of database-addressed surface grids.
pub(super) fn category(region: &profile::Region<'_>, database_ids: bool) -> Result<u32, Error> {
    if let Some(value) = region.implicit_liquid {
        if !region.liquids.is_empty() {
            return Err(Error::new(
                "world.liquid",
                "implicit full-volume and explicit surface-grid declarations conflict",
            ));
        }
        return Ok(value);
    }
    if database_ids {
        return Ok(0);
    }
    let value = region
        .liquids
        .first()
        .map_or(0, |liquid| liquid.liquid_type);
    if region
        .liquids
        .iter()
        .any(|liquid| liquid.liquid_type != value)
    {
        return Err(Error::new(
            "world.liquid.type",
            "one material-addressed region cannot contain differing liquid categories",
        ));
    }
    Ok(value)
}
