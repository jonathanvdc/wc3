//! Direct historical group fields, with no upgraded native document.
use super::{bounds, geometry, relationships, root_view::CommonRoot, scene::Identities};
use crate::model::conversion::{Context, Error};
use crate::model::semantic::profiles::wmo::{
    DrawControls, Group as WorldGroup, GroupOptions, Lighting, RegionControls, WorldSystem,
};
use crate::model::semantic::profiles::Wmo;
use crate::model::semantic::{
    ColorSpace, Draw, ImageEncoding, InstanceId, MaterialId, Mesh, MeshInstance, ModelBuilder,
    Node, Resource, Sampler, SystemId, Texture, Values, WmoGeometry,
};
use crate::model::wmo::group::GroupChunk;
use crate::model::wmo::legacy::{Batch14, Batch16, LegacyGroupChunk};
use crate::model::wmo::records::{BatchFlags, CullingBox, GroupFlags, TriangleFlags2};
use crate::model::wmo::{Classic, Read, RecordLayout};
use std::borrow::Cow;
use zerovec::ZeroVec;

pub(super) struct Header<'a> {
    pub name: u32,
    pub description: u32,
    pub bounds: [[f32; 3]; 2],
    pub area: i32,
    pub flags: GroupFlags,
    pub supplemental: u32,
    pub liquid_type: u32,
    pub references: relationships::References,
    pub transition: usize,
    pub interior: usize,
    pub batch_control: u16,
    pub interior_windows: Option<[[u16; 4]; 4]>,
    pub exterior_windows: Option<[[u16; 4]; 4]>,
    pub extra: &'a [u8],
}
pub(super) trait Batch: for<'data> Read<'data> + RecordLayout {
    fn first(&self) -> u32;
    fn count(&self) -> u16;
    fn material(&self) -> u8;
    fn flags(&self) -> BatchFlags;
    fn bounds(&self) -> CullingBox;
    fn vertices(&self) -> [u32; 2];
    fn lightmap(&self) -> Option<u8> {
        None
    }
    fn alignment(&self) -> u8 {
        0
    }
    fn supplemental(&self) -> [u16; 5] {
        [0; 5]
    }
}
impl Batch for Batch14 {
    fn first(&self) -> u32 {
        u32::from(self.start_index)
    }
    fn count(&self) -> u16 {
        self.index_count
    }
    fn material(&self) -> u8 {
        self.material
    }
    fn flags(&self) -> BatchFlags {
        self.flags
    }
    fn bounds(&self) -> CullingBox {
        self.bounds
    }
    fn vertices(&self) -> [u32; 2] {
        [
            u32::from(self.minimum_vertex),
            u32::from(self.maximum_vertex),
        ]
    }
    fn lightmap(&self) -> Option<u8> {
        Some(self.lightmap)
    }
    fn alignment(&self) -> u8 {
        self.reserved
    }
}
impl Batch for Batch16 {
    fn first(&self) -> u32 {
        self.start_index
    }
    fn count(&self) -> u16 {
        self.index_count
    }
    fn material(&self) -> u8 {
        self.material
    }
    fn flags(&self) -> BatchFlags {
        self.flags
    }
    fn bounds(&self) -> CullingBox {
        self.bounds
    }
    fn vertices(&self) -> [u32; 2] {
        [
            u32::from(self.minimum_vertex),
            u32::from(self.maximum_vertex),
        ]
    }
    fn supplemental(&self) -> [u16; 5] {
        let mut result = [0; 5];
        for (destination, source) in result
            .iter_mut()
            .zip(self.extension.as_chunks::<2>().0.iter())
        {
            *destination = u16::from_le_bytes([source[0], source[1]]);
        }
        result
    }
}

pub(super) struct ReadContext<'view, 'data> {
    pub world: &'view mut WorldSystem<'data>,
    pub owner: SystemId,
    pub identities: &'view Identities,
    pub context: &'view mut Context,
}
pub(super) fn read<'a, 'data: 'a, B: Batch>(
    root: &'a impl CommonRoot<'data, Dialect = Classic>,
    chunks: &'a [LegacyGroupChunk<'data, B>],
    header: Header<'_>,
    index: usize,
    builder: &mut ModelBuilder<'a, Wmo>,
    materials: &[MaterialId],
    target: ReadContext<'_, 'a>,
) -> Result<InstanceId, Error> {
    let ReadContext {
        world,
        owner,
        identities,
        context,
    } = target;
    let common = || {
        chunks.iter().filter_map(|chunk| match chunk {
            LegacyGroupChunk::Common(chunk) => Some(chunk),
            _ => None,
        })
    };
    let mut positions = Values::default();
    let mut normals = Values::default();
    let mut coordinates = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Values::default();
    for chunk in common() {
        match chunk {
            GroupChunk::Positions(rows) => {
                positions = Values::from_zero_vec(
                    ZeroVec::parse_bytes(rows.as_bytes()).map_err(read_error)?,
                )
            }
            GroupChunk::Normals(rows) => {
                normals = Values::from_zero_vec(
                    ZeroVec::parse_bytes(rows.as_bytes()).map_err(read_error)?,
                )
            }
            GroupChunk::TextureCoordinates(rows) => coordinates.push(Values::from_zero_vec(
                ZeroVec::parse_bytes(rows.as_bytes()).map_err(read_error)?,
            )),
            GroupChunk::Indices(rows) => {
                indices = Values::view(rows.len(), move |index| {
                    u32::from(rows.get(index).unwrap().unwrap())
                })
            }
            GroupChunk::VertexColors(rows) | GroupChunk::VertexColors2(rows) => {
                colors.push(Values::view(rows.len(), move |index| {
                    super::rgba(rows.get(index).unwrap().unwrap())
                        .map(|value| f32::from(value) / 255.)
                }))
            }
            _ => {}
        }
    }
    for chunk in chunks {
        if let LegacyGroupChunk::LegacyIndices(rows) = chunk {
            indices = Values::view(rows.len(), move |index| {
                u32::from(rows.get(index).unwrap().unwrap())
            });
        }
    }
    if indices.len() % 3 != 0 {
        return Err(Error::new(
            "world.group.indices",
            "historical indices must form complete triangles",
        ));
    }
    let triangles = Values::view(indices.len() / 3, move |index| {
        [
            indices.get(index * 3).unwrap(),
            indices.get(index * 3 + 1).unwrap(),
            indices.get(index * 3 + 2).unwrap(),
        ]
    });
    let mut faces = Vec::new();
    for chunk in chunks {
        if let LegacyGroupChunk::Triangles(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(read_error)?;
                let mut face = geometry::face(
                    TriangleFlags2(u16::from(row.flags.bits())),
                    u16::from(row.material),
                    row.material == u8::MAX,
                    materials,
                )?;
                face.lightmap_selector = Some(row.lightmap);
                face.alignment = row.reserved;
                faces.push(face);
            }
        }
    }
    for index in super::spatial_queries::faces(common(), context)? {
        faces
            .get_mut(index)
            .ok_or_else(|| Error::new("world.group.query", "query references absent face"))?
            .spatial_queries = true;
    }
    let mut geometry = WmoGeometry::triangles(positions, triangles)?
        .with_normals(normals)?
        .with_texture_coordinates(coordinates)?
        .with_color_sets(colors)?
        .with_triangle_controls(Values::owned(faces))?;
    let mut region = RegionControls {
        description: relationships::name(root, header.description)?,
        area_id: u32::from_ne_bytes(header.area.to_ne_bytes()),
        options: group_options(header.flags, header.supplemental),
        batch_control: header.batch_control,
        interior_windows: header.interior_windows,
        exterior_windows: header.exterior_windows,
        ..Default::default()
    };
    for chunk in chunks {
        match chunk {
            LegacyGroupChunk::LightMapCoordinates(rows) => {
                geometry = geometry.with_lightmap_coordinates(Values::from_zero_vec(
                    ZeroVec::parse_bytes(rows.as_bytes()).map_err(read_error)?,
                ))?
            }
            LegacyGroupChunk::LightMaps(rows) => {
                region.lightmap_rectangles = Values::view(rows.len(), move |index| {
                    let row = rows.get(index).unwrap().unwrap();
                    [row.x, row.y, row.width, row.height]
                })
            }
            LegacyGroupChunk::LightMapData(rows) => {
                for index in 0..rows.len() {
                    let image = &rows.as_bytes()[index * 32772..index * 32772 + 32768];
                    region.lightmap_images.push(builder.add_texture(Texture {
                        resource: Resource::EmbeddedImage {
                            encoding: ImageEncoding::Unspecified,
                            bytes: Cow::Borrowed(image),
                        },
                        controls: Default::default(),
                        color_space: ColorSpace::Unspecified,
                        sampler: Sampler::default(),
                    })?);
                }
            }
            LegacyGroupChunk::Liquid(liquid) => {
                region
                    .liquids
                    .push(crate::model::semantic::profiles::wmo::Liquid {
                        position: liquid.header.corner,
                        dimensions: liquid
                            .header
                            .vertex_dimensions
                            .map(|value| u32::try_from(value).expect("validated dimensions")),
                        vertices: Values::view(liquid.vertices.len(), move |index| {
                            let row = liquid.vertices.get(index).unwrap().unwrap();
                            crate::model::semantic::profiles::wmo::LiquidVertex::Unresolved {
                                height: row.height,
                                controls: [
                                    u16::from_le_bytes([row.data[0], row.data[1]]),
                                    u16::from_le_bytes([row.data[2], row.data[3]]),
                                ],
                            }
                        }),
                        tiles: Values::view(liquid.tiles.len(), move |index| {
                            liquid.tiles.get(index).unwrap().unwrap()
                        }),
                        material: Some(
                            *materials
                                .get(usize::from(liquid.header.material))
                                .ok_or_else(|| {
                                    Error::new(
                                        "world.liquid.material",
                                        "liquid material does not exist",
                                    )
                                })?,
                        ),
                        liquid_type: header.liquid_type,
                    });
            }
            LegacyGroupChunk::BatchIndices(rows) => {
                region
                    .auxiliary_geometry
                    .get_or_insert_with(Default::default)
                    .indices =
                    Values::view(rows.len(), move |index| rows.get(index).unwrap().unwrap())
            }
            LegacyGroupChunk::BatchGeometry(rows) => {
                region
                    .auxiliary_geometry
                    .get_or_insert_with(Default::default)
                    .values = Values::from_zero_vec(
                    ZeroVec::parse_bytes(rows.as_bytes()).map_err(read_error)?,
                )
            }
            LegacyGroupChunk::BatchVertices(rows) => {
                region
                    .auxiliary_geometry
                    .get_or_insert_with(Default::default)
                    .vertex_windows = Values::view(rows.len(), move |index| {
                    let row = rows.get(index).unwrap().unwrap();
                    [row.start, row.count]
                })
            }
            LegacyGroupChunk::BatchPlanes(rows) => {
                region
                    .auxiliary_geometry
                    .get_or_insert_with(Default::default)
                    .plane_windows = Values::view(rows.len(), move |index| {
                    let row = rows.get(index).unwrap().unwrap();
                    [row.start, row.count]
                })
            }
            _ => {}
        }
    }
    region.supplemental_controls = header
        .extra
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect();
    relationships::read_fields(
        root,
        common(),
        header.references,
        &mut region,
        identities,
        owner,
    )?;
    if world.fogs.is_empty() {
        region.fogs.clear();
    }
    let mut draws = Vec::new();
    for chunk in chunks {
        if let LegacyGroupChunk::Batches(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(read_error)?;
                if row.first() % 3 != 0 || row.count() % 3 != 0 {
                    return Err(Error::new(
                        "world.group.batch",
                        "historical draw does not cover whole triangles",
                    ));
                }
                let material = *materials
                    .get(usize::from(row.material()))
                    .ok_or_else(|| Error::new("world.group.batch", "material does not exist"))?;
                let lighting = if draws.len() < header.transition {
                    Lighting::Transition
                } else if draws.len() < header.transition + header.interior {
                    Lighting::Interior
                } else {
                    Lighting::Exterior
                };
                draws.push(Draw {
                    first: row.first() / 3,
                    count: u32::from(row.count()) / 3,
                    material,
                    controls: Some(DrawControls {
                        lighting,
                        vertex_window: Some(row.vertices()),
                        bounds: Some(bounds([
                            row.bounds().minimum.map(f32::from),
                            row.bounds().maximum.map(f32::from),
                        ])),
                        lightmap_selector: row.lightmap(),
                        alignment: row.alignment(),
                        unclassified: row.flags().bits().into(),
                        supplemental: row.supplemental(),
                        ..Default::default()
                    }),
                });
            }
        }
    }
    let mut mesh = Mesh::<Wmo>::new(geometry);
    mesh.name = relationships::name(root, header.name)?;
    mesh.bounds = bounds(header.bounds);
    let mesh = builder.add_mesh(mesh)?;
    let node = builder.add_node(Node::named(format!("Group {index}")))?;
    if header.flags.has_liquid() && region.liquids.is_empty() {
        region.implicit_liquid = Some(header.liquid_type);
    }
    let instance = builder.add_instance(MeshInstance {
        controls: region,
        node,
        mesh,
        skin: None,
        draws,
        lod: Some(0),
        selection_group: 0,
    })?;
    while world.groups.len() <= index {
        world.groups.push(WorldGroup::default());
    }
    world.groups[index].instances.push(instance);
    Ok(instance)
}
pub(super) fn group_options(flags: GroupFlags, supplemental: u32) -> GroupOptions {
    super::group_flags::read(flags, supplemental)
}
fn read_error(error: impl std::fmt::Display) -> Error {
    Error::new("world.group", error.to_string())
}
