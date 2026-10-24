//! Historical group records constructed directly from logical geometry.
use super::encode::array;
use crate::model::common::PackedArray;
use crate::model::conversion::{Context, Error};
use crate::model::error::Location;
use crate::model::formats::wmo::group::{GroupChunk, LiquidHeader, LiquidVertex};
use crate::model::formats::wmo::legacy::{
    Batch14, Batch16, BatchRange, LegacyGroupChunk, LegacyLiquid, LegacyTriangle, LightMap,
    LightMapData,
};
use crate::model::formats::wmo::records::{
    BatchFlags, BspFlags, BspNode, CullingBox, GroupFlags, TriangleFlags,
};
use crate::model::formats::wmo::{RecordLayout, Write};
use crate::model::profiles::wmo::{Lighting, Pass, Region};
use crate::model::profiles::Wmo;
use crate::model::WorldImageResource;
use crate::model::{InstanceId, LookId, MaterialId, Model, Topology};
use std::collections::BTreeMap;
use std::error::Error as StdError;

pub(super) struct DrawFields {
    pub bounds: CullingBox,
    pub first: u32,
    pub count: u16,
    pub minimum: u16,
    pub maximum: u16,
    pub material: u8,
    pub flags: BatchFlags,
    pub lightmap: Option<u8>,
    pub alignment: u8,
    pub supplemental: [u16; 5],
}
pub(super) trait Batch: Write + RecordLayout {
    fn create(fields: DrawFields, context: &mut Context) -> Result<Self, Error>
    where
        Self: Sized;
}
impl Batch for Batch14 {
    fn create(fields: DrawFields, context: &mut Context) -> Result<Self, Error> {
        if fields.supplemental.iter().any(|value| *value != 0) {
            context.unsupported(
                Location::new().field("world").field("draw"),
                "historical lightmapped draw has no supplemental control words",
            )?;
        }
        let lightmap = if let Some(value) = fields.lightmap {
            value
        } else {
            context.unsupported(
                Location::new()
                    .field("world")
                    .field("draw")
                    .field("lightmap"),
                "historical lightmapped draw requires an explicit baked-light selector",
            )?;
            0
        };
        Ok(Self {
            lightmap,
            material: fields.material,
            bounds: fields.bounds,
            start_index: u16::try_from(fields.first).map_err(error)?,
            index_count: fields.count,
            minimum_vertex: fields.minimum,
            maximum_vertex: fields.maximum,
            flags: fields.flags,
            reserved: fields.alignment,
        })
    }
}
impl Batch for Batch16 {
    fn create(fields: DrawFields, context: &mut Context) -> Result<Self, Error> {
        if fields.lightmap.is_some() || fields.alignment != 0 || fields.supplemental[4] != 0 {
            context.unsupported(Location::new().field("world").field("draw"), "historical split draw cannot encode baked-light selector, alignment, or fifth supplemental control")?;
        }
        let mut extension = [0; 8];
        for (bytes, value) in extension
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .zip(fields.supplemental)
        {
            bytes.copy_from_slice(&value.to_le_bytes());
        }
        Ok(Self {
            bounds: fields.bounds,
            start_index: fields.first,
            index_count: fields.count,
            minimum_vertex: fields.minimum,
            maximum_vertex: fields.maximum,
            flags: fields.flags,
            material: fields.material,
            extension,
        })
    }
}
pub(super) struct Body<'a, B> {
    pub chunks: Vec<LegacyGroupChunk<'a, B>>,
    pub flags: GroupFlags,
    pub counts: [u16; 3],
}
pub(super) fn write<'a, B: Batch>(
    model: &Model<'_, Wmo>,
    instance_id: InstanceId,
    materials: &BTreeMap<MaterialId, u16>,
    look: Option<LookId>,
    context: &mut Context,
) -> Result<Body<'a, B>, Error> {
    let instance = model.instance(instance_id).expect("validated placement");
    let mesh = model.mesh(instance.mesh).expect("validated geometry");
    let region = &instance.properties;
    if !mesh.geometry.strip_indices().is_empty() || !mesh.geometry.tangent_vectors().is_empty() {
        context.unsupported(
            Location::new()
                .field("world")
                .field("geometry")
                .field("streams"),
            "historical geometry cannot encode strip or tangent pools",
        )?;
    }
    if instance.lod.is_some_and(|lod| lod != 0)
        || region.file_id.is_some()
        || region.parent_or_first_child.is_some()
        || region.next_child.is_some()
        || !region.point_lights.is_empty()
        || !region.point_programs.is_empty()
        || !region.terrain_cut_planes.is_empty()
        || region.ambient_override.is_some()
        || region.detail_doodad_key.is_some()
    {
        context.unsupported(Location::new().field("world").field("group"), "historical group has no database-addressed detail levels, split links, modern light programs, or terrain/ambient extensions")?;
    }
    let mut flags = super::group_flags::write(region.options);
    flags.set_has_vertex_colors(!mesh.geometry.color_sets().is_empty());
    flags.set_has_liquid(!region.liquids.is_empty() || region.implicit_liquid.is_some());
    let mut result = Body {
        chunks: Vec::new(),
        flags,
        counts: [0; 3],
    };
    result
        .chunks
        .push(LegacyGroupChunk::Common(GroupChunk::Positions(array(
            &mesh
                .positions()
                .iter()
                .map(|v| v.to_array())
                .collect::<Vec<_>>(),
        )?)));
    if let Some(normals) = mesh.normals() {
        result
            .chunks
            .push(LegacyGroupChunk::Common(GroupChunk::Normals(array(
                &normals.iter().map(|v| v.to_array()).collect::<Vec<_>>(),
            )?)));
    }
    for coordinates in mesh.texture_coordinates() {
        result
            .chunks
            .push(LegacyGroupChunk::Common(GroupChunk::TextureCoordinates(
                array(&coordinates.storage().iter().collect::<Vec<_>>())?,
            )));
    }
    for colors in mesh.geometry.color_sets() {
        let colors = colors
            .iter()
            .map(|rgba| {
                let mut encoded = [0; 4];
                for (byte, value) in encoded.iter_mut().zip(rgba.to_array()) {
                    let rounded = (value.clamp(0., 1.) * 255.).round() as u8;
                    context.approximate(
                        Location::new().field("world").field("colors"),
                        (value - f32::from(rounded) / 255.).abs(),
                        "vertex colors quantized to eight-bit components",
                    )?;
                    *byte = rounded;
                }
                Ok(super::rgba(encoded))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        result
            .chunks
            .push(LegacyGroupChunk::Common(GroupChunk::VertexColors(array(
                &colors,
            )?)));
    }
    let Topology::Triangles(triangles) = mesh.topology() else {
        return Err(Error::requirement(
            Location::new().field("world").field("topology"),
            "historical group requires triangle connectivity",
        ));
    };
    let indices = triangles
        .iter()
        .flatten()
        .map(|index| u16::try_from(index).map_err(error))
        .collect::<Result<Vec<_>, _>>()?;
    result
        .chunks
        .push(LegacyGroupChunk::Common(GroupChunk::Indices(array(
            &indices,
        )?)));
    let faces = mesh
        .geometry
        .triangle_properties()
        .iter()
        .map(|face| {
            if face.ground_type_selector.is_some() || face.ground_type.is_some() {
                context.unsupported(
                    Location::new().field("world").field("face"),
                    "historical face has no extended query selectors",
                )?;
            }
            let mut flags = TriangleFlags(0);
            flags.set_transition(face.transition);
            flags.set_no_camera_collision(face.no_camera_collision);
            flags.set_detail(face.detail);
            flags.set_collision(face.collision);
            flags.set_hint(face.hint);
            flags.set_render(face.render);
            flags.set_cull_objects(face.cull_objects);
            flags.set_collide_hit(face.collide_hit);
            Ok(LegacyTriangle {
                flags,
                lightmap: face.lightmap_selector.unwrap_or(0),
                material: face.material.map_or(Ok(u8::MAX), |id| {
                    u8::try_from(materials[&id]).map_err(error)
                })?,
                reserved: face.alignment,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    result
        .chunks
        .push(LegacyGroupChunk::Triangles(array(&faces)?));
    let query = mesh
        .geometry
        .triangle_properties()
        .iter()
        .enumerate()
        .filter_map(|(index, face)| face.spatial_queries.then_some(index))
        .map(|index| u16::try_from(index).map_err(error))
        .collect::<Result<Vec<_>, _>>()?;
    result.flags.set_has_bsp(!query.is_empty());
    if !query.is_empty() {
        let mut flags = BspFlags::default();
        flags.set_leaf(true);
        result
            .chunks
            .push(LegacyGroupChunk::Common(GroupChunk::BspNodes(array(&[
                BspNode {
                    flags,
                    negative_child: -1,
                    positive_child: -1,
                    face_count: u16::try_from(query.len()).map_err(error)?,
                    first_face: 0,
                    plane_distance: 0.,
                },
            ])?)));
        result
            .chunks
            .push(LegacyGroupChunk::Common(GroupChunk::BspFaces(array(
                &query,
            )?)));
    }
    let mut batches = Vec::new();
    for draw in &instance.draws {
        let draw = mesh.resolve_draw(draw)?;
        let control = draw.properties.clone().unwrap_or_default();
        if draw.subset.strip.is_some() || draw.subset.tangents.is_some() {
            context.unsupported(
                Location::new()
                    .field("world")
                    .field("draw")
                    .field("streams"),
                "historical draws cannot encode strip overrides or tangent associations",
            )?;
        }
        if control.pass != Pass::Surface {
            context.unsupported(
                Location::new().field("world").field("draw").field("pass"),
                "historical draw has no independent shadow-pass record",
            )?;
            continue;
        }
        let material = u8::try_from(
            materials[&model
                .material_for_draw(instance_id, draw.id, look)
                .expect("validated material selection")],
        )
        .map_err(error)?;
        let bounds = control.bounds.unwrap_or(mesh.bounds);
        let bounds =
            [bounds.minimum, bounds.maximum].map(|corner| corner.map(|value| value.round()));
        if bounds
            .iter()
            .flat_map(|v| v.to_array())
            .any(|value| value < f32::from(i16::MIN) || value > f32::from(i16::MAX))
        {
            return Err(Error::requirement(
                Location::new().field("world").field("draw").field("bounds"),
                "culling bounds exceed signed sixteen-bit coordinates",
            ));
        }
        let exact = control.bounds.unwrap_or(mesh.bounds);
        let quantization = [exact.minimum, exact.maximum]
            .iter()
            .flat_map(|v| v.to_array())
            .zip(bounds.iter().flat_map(|v| v.to_array()))
            .map(|(a, b)| (a - b).abs())
            .fold(0., f32::max);
        context.approximate(
            Location::new().field("world").field("draw").field("bounds"),
            quantization,
            "historical draw bounds quantized to integer coordinates",
        )?;
        let bounds = CullingBox {
            minimum: bounds[0].to_array().map(|value| value as i16),
            maximum: bounds[1].to_array().map(|value| value as i16),
        };
        let selected = triangles
            .iter()
            .skip(draw.first as usize)
            .take(draw.count as usize)
            .flatten()
            .collect::<Vec<_>>();
        let [minimum, maximum] = draw.subset.vertex_window.unwrap_or([
            selected.iter().copied().min().unwrap_or(0),
            selected.iter().copied().max().unwrap_or(0),
        ]);
        let minimum = u16::try_from(minimum).map_err(error)?;
        let maximum = u16::try_from(maximum).map_err(error)?;
        let batch = B::create(
            DrawFields {
                bounds,
                first: draw.first.checked_mul(3).ok_or_else(|| {
                    Error::requirement(
                        Location::new().field("world").field("draw"),
                        "draw index overflows",
                    )
                })?,
                count: u16::try_from(draw.count.checked_mul(3).ok_or_else(|| {
                    Error::requirement(
                        Location::new().field("world").field("draw"),
                        "draw count overflows",
                    )
                })?)
                .map_err(error)?,
                minimum,
                maximum,
                material,
                flags: BatchFlags(0),
                lightmap: control.lightmap_selector,
                alignment: control.alignment,
                supplemental: control.supplemental,
            },
            context,
        )?;
        let index = match control.lighting {
            Lighting::Transition => 0,
            Lighting::Interior => 1,
            Lighting::Exterior => 2,
            Lighting::Unknown(_) => {
                context.unsupported(
                    Location::new()
                        .field("world")
                        .field("draw")
                        .field("lighting"),
                    "historical draw requires an established illumination class",
                )?;
                2
            }
        };
        result.counts[index] = result.counts[index].checked_add(1).ok_or_else(|| {
            Error::requirement(
                Location::new().field("world").field("draw"),
                "draw count exceeds header capacity",
            )
        })?;
        batches.push(batch);
    }
    result
        .chunks
        .push(LegacyGroupChunk::Batches(array(&batches)?));
    write_lighting(region, model, &mut result.chunks)?;
    if !mesh.geometry.lightmap_coordinates().is_empty() {
        result
            .chunks
            .push(LegacyGroupChunk::LightMapCoordinates(array(
                &mesh
                    .geometry
                    .lightmap_coordinates()
                    .storage()
                    .iter()
                    .collect::<Vec<_>>(),
            )?));
    }
    for liquid in &region.liquids {
        let material = liquid.material.map_or(0, |id| materials[&id]);
        let dimensions = liquid
            .dimensions
            .map(|value| i32::try_from(value).map_err(error));
        let [x, y] = dimensions;
        let dimensions = [x?, y?];
        let vertices = liquid
            .vertices
            .iter()
            .map(|vertex| match vertex {
                crate::model::profiles::wmo::LiquidVertex::Unresolved { height, controls } => {
                    let a = controls[0].to_le_bytes();
                    let b = controls[1].to_le_bytes();
                    Ok(LiquidVertex {
                        height,
                        data: [a[0], a[1], b[0], b[1]],
                    })
                }
                crate::model::profiles::wmo::LiquidVertex::Water {
                    height,
                    velocities,
                    first_percentage,
                    control,
                } => Ok(LiquidVertex {
                    height,
                    data: [velocities[0], velocities[1], first_percentage, control],
                }),
                crate::model::profiles::wmo::LiquidVertex::Magma {
                    height,
                    coordinates,
                } => {
                    let coords = coordinates.map(|value| u16::try_from(value).map_err(error));
                    let [a, b] = coords;
                    let a = a?.to_le_bytes();
                    let b = b?.to_le_bytes();
                    Ok(LiquidVertex {
                        height,
                        data: [a[0], a[1], b[0], b[1]],
                    })
                }
            })
            .collect::<Result<Vec<_>, Error>>()?;
        result.chunks.push(LegacyGroupChunk::Liquid(LegacyLiquid {
            header: LiquidHeader {
                vertex_dimensions: dimensions,
                tile_dimensions: dimensions.map(|value| value.saturating_sub(1)),
                corner: (liquid.position).to_array(),
                material,
            },
            vertices: array(&vertices)?,
            tiles: array(&liquid.tiles.to_vec())?,
        }));
    }
    Ok(result)
}
fn write_lighting<'a, B: RecordLayout>(
    region: &Region<'_>,
    model: &Model<'_, Wmo>,
    chunks: &mut Vec<LegacyGroupChunk<'a, B>>,
) -> Result<(), Error> {
    if !region.lightmap_rectangles.is_empty() {
        let rows = region
            .lightmap_rectangles
            .iter()
            .map(|[x, y, width, height]| LightMap {
                x,
                y,
                width,
                height,
            })
            .collect::<Vec<_>>();
        chunks.push(LegacyGroupChunk::LightMaps(array(&rows)?));
    }
    if !region.lightmap_images.is_empty() {
        let images = region
            .lightmap_images
            .iter()
            .map(|id| {
                let texture = id;
                let WorldImageResource::EmbeddedImage { bytes, .. } = model
                    .image_resource(texture.resource)
                    .expect("validated image")
                else {
                    return Err(Error::requirement(
                        Location::new().field("world").field("lightmap"),
                        "historical lightmap requires embedded encoded image bytes",
                    ));
                };
                let texels = bytes.as_ref().try_into().map_err(|_| {
                    Error::requirement(
                        Location::new().field("world").field("lightmap"),
                        "historical lightmap requires exactly 32768 encoded texel bytes",
                    )
                })?;
                Ok(LightMapData {
                    texels,
                    padding: [0; 4],
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        chunks.push(LegacyGroupChunk::LightMapData(array(&images)?));
    }
    if let Some(auxiliary) = &region.auxiliary_geometry {
        if !auxiliary.indices.is_empty() {
            chunks.push(LegacyGroupChunk::BatchIndices(array(
                &auxiliary.indices.to_vec(),
            )?));
        }
        if !auxiliary.values.is_empty() {
            chunks.push(LegacyGroupChunk::BatchGeometry(array(
                &auxiliary
                    .values
                    .iter()
                    .map(|v| v.to_array())
                    .collect::<Vec<_>>(),
            )?));
        }
        let windows = |values: &PackedArray<'_, [u16; 2]>| {
            values
                .iter()
                .map(|[start, count]| BatchRange { start, count })
                .collect::<Vec<_>>()
        };
        if !auxiliary.vertex_windows.is_empty() {
            chunks.push(LegacyGroupChunk::BatchVertices(array(&windows(
                &auxiliary.vertex_windows,
            ))?));
        }
        if !auxiliary.plane_windows.is_empty() {
            chunks.push(LegacyGroupChunk::BatchPlanes(array(&windows(
                &auxiliary.plane_windows,
            ))?));
        }
    }
    Ok(())
}
fn error(error: impl StdError + Send + Sync + 'static) -> Error {
    Error::from_source(Location::new().field("world").field("group"), error)
}
