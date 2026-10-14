//! Historical root envelopes decoded directly into the world vocabulary.
use super::{environment, legacy_geometry, materials, root_view::CommonRoot, scene, visibility};
use crate::model::common::FileKey;
use crate::model::conversion::{Asset, Context, Conversion, Error, ImportOptions, OpaquePayload};
use crate::model::semantic::profiles::wmo::{ModelControls, WorldOptions, WorldSystem};
use crate::model::semantic::profiles::Wmo;
use crate::model::semantic::{ModelBuilder, Resource};
use crate::model::wmo::legacy::{Bundle16, Root14, RootChunk14};
use crate::model::wmo::root::RootChunk;
use std::borrow::Cow;

pub(super) fn alpha<'a>(
    source: &'a Root14<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Asset<'a, Wmo>>, Error> {
    source.validate().map_err(read_error)?;
    let mut context = Context::import(options);
    let mut opaque = Vec::new();
    collect_root(source, options, &mut context, &mut opaque)?;
    for unknown in source.unknown_chunks() {
        super::handle_unknown(
            &mut opaque,
            &mut context,
            options,
            FileKey::Root,
            unknown.tag,
            unknown.data.as_ref(),
        )?;
    }
    let mut builder = ModelBuilder::<Wmo>::with_profile("World model");
    builder.set_controls(ModelControls {
        id: source.header().wmo_id,
        ambient: super::rgba(source.header().ambient_color),
        skybox: skybox(source),
        ..Default::default()
    });
    let materials = materials::read_alpha(source, &mut builder)?;
    let mut world = WorldSystem::default();
    let owner = builder.add_system(WorldSystem::default())?;
    let identities = scene::read(source, &mut builder, &mut world, owner, &mut context)?;
    environment::read(source, &mut world, owner)?;
    visibility::read(source, &mut world, owner)?;
    for (index, group) in source.groups().enumerate() {
        collect_group(
            &group.chunks,
            FileKey::Root.sibling(format!("_{index:03}.wmo")),
            options,
            &mut context,
            &mut opaque,
        )?;
        let header = &group.header;
        let portal_start = usize::try_from(header.portal_start).map_err(|_| {
            Error::new(
                "world.group.portal",
                "negative portal start cannot address a portal table",
            )
        })?;
        let windows = |rows: [crate::model::wmo::legacy::HeaderBatchRange; 4]| {
            rows.map(|row| {
                [
                    row.vertex_start,
                    row.vertex_count,
                    row.batch_start,
                    row.batch_count,
                ]
            })
        };
        legacy_geometry::read(
            source,
            &group.chunks,
            legacy_geometry::Header {
                name: header.name_offset,
                description: header.descriptive_name_offset,
                bounds: [header.bounds_minimum, header.bounds_maximum],
                area: header.unique_id,
                flags: header.flags,
                supplemental: 0,
                liquid_type: header.liquid_type,
                references: super::relationships::References {
                    fogs: header.fog_indices,
                    portal_start,
                    portal_count: header.portal_count as usize,
                },
                batch_control: 0,
                transition: 0,
                interior: header
                    .interior_batches
                    .iter()
                    .map(|row| usize::from(row.batch_count))
                    .sum(),
                interior_windows: Some(windows(header.interior_batches)),
                exterior_windows: Some(windows(header.exterior_batches)),
                extra: &header.reserved_tail,
            },
            index,
            &mut builder,
            &materials,
            legacy_geometry::ReadContext {
                world: &mut world,
                owner,
                identities: &identities,
                context: &mut context,
            },
        )?;
    }
    for chunk in source.chunks() {
        if let RootChunk14::Groups(rows) = chunk {
            for (index, row) in rows.iter().enumerate() {
                let row = row.map_err(read_error)?;
                let group = world.groups.get_mut(index).ok_or_else(|| {
                    Error::new("world.catalog", "catalog exceeds embedded groups")
                })?;
                group.name = if row.name_offset < 0 {
                    Cow::Borrowed("")
                } else {
                    super::relationships::name(source, row.name_offset as u32)?
                };
                group.bounds = row.bounds;
                group.options = legacy_geometry::group_options(row.flags, 0);
            }
        }
    }
    builder.set_system(owner, world)?;
    Ok(Conversion {
        value: {
            let mut asset = Asset::new(builder.finish()?);
            asset.opaque = opaque;
            asset
        },
        report: context.report,
    })
}
pub(super) fn beta<'a>(
    source: &'a Bundle16<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Asset<'a, Wmo>>, Error> {
    source.validate_complete().map_err(read_error)?;
    let mut context = Context::import(options);
    let mut opaque = Vec::new();
    collect_root(&source.root, options, &mut context, &mut opaque)?;
    let mut builder = ModelBuilder::<Wmo>::with_profile("World model");
    let header = source.root.header();
    let flags = header.flags();
    let mut unknown = flags;
    unknown.set_no_vertex_attenuation(false);
    unknown.set_unified_render_path(false);
    unknown.set_liquid_type_database_id(false);
    unknown.set_preserve_vertex_alpha(false);
    unknown.set_default_max_lod(false);
    unknown.set_has_group_lods(false);
    builder.set_bounds(Some(super::bounds(header.bounds)));
    builder.set_controls(ModelControls {
        id: header.wmo_id,
        ambient: super::rgba(header.ambient_color),
        skybox: skybox(&source.root),
        options: WorldOptions {
            default_max_lod: flags.default_max_lod(),
            no_vertex_attenuation: flags.no_vertex_attenuation(),
            unified_rendering: flags.unified_render_path(),
            database_liquids: flags.liquid_type_database_id(),
            preserve_vertex_alpha: flags.preserve_vertex_alpha(),
            unclassified: unknown.bits().into(),
        },
    });
    let materials = materials::read(&source.root, &mut builder)?;
    let mut world = WorldSystem::default();
    let owner = builder.add_system(WorldSystem::default())?;
    let identities = scene::read(&source.root, &mut builder, &mut world, owner, &mut context)?;
    environment::read(&source.root, &mut world, owner)?;
    visibility::read(&source.root, &mut world, owner)?;
    for index in 0..source.root.group_count() {
        let group = source
            .group(index)
            .expect("validated complete companion set");
        collect_group(
            &group.chunks,
            FileKey::Root.sibling(format!("_{index:03}.wmo")),
            options,
            &mut context,
            &mut opaque,
        )?;
        let header = &group.header;
        let portal_start = usize::try_from(header.portal_start).map_err(|_| {
            Error::new(
                "world.group.portal",
                "negative portal start cannot address a portal table",
            )
        })?;
        let extra = header.reserved_tail.to_le_bytes();
        legacy_geometry::read(
            &source.root,
            &group.chunks,
            legacy_geometry::Header {
                name: header.name_offset,
                description: header.descriptive_name_offset,
                bounds: [header.bounds_minimum, header.bounds_maximum],
                area: header.unique_id,
                flags: header.flags,
                supplemental: header.flags2.bits(),
                liquid_type: header.liquid_type,
                references: super::relationships::References {
                    fogs: header.fog_indices,
                    portal_start,
                    portal_count: usize::from(header.portal_count),
                },
                batch_control: header.reserved_batch_count,
                transition: usize::from(header.transition_batch_count),
                interior: usize::from(header.interior_batch_count),
                interior_windows: None,
                exterior_windows: None,
                extra: &extra,
            },
            index,
            &mut builder,
            &materials,
            legacy_geometry::ReadContext {
                world: &mut world,
                owner,
                identities: &identities,
                context: &mut context,
            },
        )?;
    }
    for chunk in source.root.chunks() {
        if let RootChunk::Groups(rows) = chunk {
            for (index, row) in rows.iter().enumerate() {
                let row = row.map_err(read_error)?;
                let group = world.groups.get_mut(index).ok_or_else(|| {
                    Error::new("world.catalog", "catalog exceeds resolved groups")
                })?;
                group.name = if row.name_offset < 0 {
                    Cow::Borrowed("")
                } else {
                    super::relationships::name(&source.root, row.name_offset as u32)?
                };
                group.bounds = row.bounds;
                group.options = legacy_geometry::group_options(row.flags, 0);
            }
        }
    }
    builder.set_system(owner, world)?;
    Ok(Conversion {
        value: {
            let mut asset = Asset::new(builder.finish()?);
            asset.opaque = opaque;
            asset
        },
        report: context.report,
    })
}
fn skybox<'a, 'data: 'a>(source: &'a impl CommonRoot<'data>) -> Option<Resource<'a>> {
    source.common_chunks().find_map(|chunk| match chunk {
        RootChunk::SkyboxName(rows) => {
            let bytes = rows.as_bytes();
            let end = bytes
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(bytes.len());
            Some(Resource::Path(Cow::Borrowed(&bytes[..end])))
        }
        _ => None,
    })
}
fn read_error(error: impl std::fmt::Display) -> Error {
    Error::new("world", error.to_string())
}

fn collect_root<'a, 'data: 'a>(
    source: &'a impl CommonRoot<'data>,
    options: &ImportOptions,
    context: &mut Context,
    opaque: &mut Vec<OpaquePayload<'a>>,
) -> Result<(), Error> {
    for chunk in source.common_chunks() {
        let unknown = match chunk {
            RootChunk::Unknown { tag, data } => Some((*tag, data.as_ref())),
            RootChunk::M3Materials(data) | RootChunk::DetailDoodads(data) => {
                Some((chunk.tag(), data.as_ref()))
            }
            RootChunk::ParticulateVolumes(rows) => Some((chunk.tag(), rows.as_bytes())),
            _ => None,
        };
        if let Some((tag, bytes)) = unknown {
            super::handle_unknown(opaque, context, options, FileKey::Root, tag, bytes)?;
        }
    }
    Ok(())
}
fn collect_group<'a, 'data: 'a, B>(
    chunks: &'a [crate::model::wmo::legacy::LegacyGroupChunk<'data, B>],
    file: FileKey,
    options: &ImportOptions,
    context: &mut Context,
    opaque: &mut Vec<OpaquePayload<'a>>,
) -> Result<(), Error> {
    use crate::model::wmo::group::GroupChunk;
    use crate::model::wmo::legacy::LegacyGroupChunk;
    for chunk in chunks {
        if let LegacyGroupChunk::Common(chunk) = chunk {
            let unknown = match chunk {
                GroupChunk::Unknown { tag, data } => Some((*tag, data.as_ref())),
                GroupChunk::SpotLights(rows) => Some((chunk.tag(), rows.as_bytes())),
                GroupChunk::SpotLightAnimations(rows) => Some((chunk.tag(), rows.as_bytes())),
                GroupChunk::PrepassBatches(rows) => Some((chunk.tag(), rows.as_bytes())),
                GroupChunk::TangentsUnresolved(data) => Some((chunk.tag(), data.as_ref())),
                GroupChunk::QueryGeometry(query) if query.extension.len() > 1 => {
                    Some((chunk.tag(), &query.extension[1..]))
                }
                _ => None,
            };
            if let Some((tag, bytes)) = unknown {
                super::handle_unknown(opaque, context, options, file.clone(), tag, bytes)?;
            }
        }
    }
    Ok(())
}
