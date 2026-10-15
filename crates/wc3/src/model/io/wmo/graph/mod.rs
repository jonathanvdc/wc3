//! Direct typed world-model authoring graph codecs.
use crate::model::Model;
mod alpha_materials;
mod draw;
mod encode;
mod encode_scene;
mod environment;
mod geometry;
mod group_flags;
mod group_output;
mod legacy_encode;
mod legacy_encode_geometry;
mod legacy_geometry;
mod legacy_import;
mod lighting_windows;
mod liquids;
mod materials;
mod normalize;
mod region_lights;
mod relationships;
mod root_output;
mod root_view;
mod scene;
mod spatial_queries;
mod visibility;
use crate::model::common::FileKey;
use crate::model::conversion::{Context, Conversion, Error, ImportOptions, IssueKind};
use crate::model::formats::wmo::{Bundle, Dialect, GroupKey};
use crate::model::profiles::wmo::{ModelMetadata, WorldOptions, WorldSystem};
use crate::model::profiles::Wmo;
use crate::model::{Bounds, ModelBuilder};
use std::collections::BTreeSet;

pub(super) fn import<'a, D: Dialect>(
    source: &'a Bundle<'_, D>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, Wmo>>, Error> {
    source
        .validate_complete()
        .map_err(|error| Error::new("world", error.to_string()))?;
    let mut context = Context::import(options);
    for chunk in source.root.chunks() {
        use crate::model::formats::wmo::root::RootChunk;
        let unresolved = match chunk {
            RootChunk::Unknown { tag, .. } => Some(*tag),
            RootChunk::M3Materials(_) | RootChunk::DetailDoodads(_) => Some(chunk.tag()),
            RootChunk::ParticulateVolumes(_) => Some(chunk.tag()),
            _ => None,
        };
        if let Some(tag) = unresolved {
            handle_unknown(&mut context, FileKey::Root, tag)?;
        }
    }
    let mut builder = ModelBuilder::<Wmo>::with_profile("World model");
    let header = source.root.header();
    builder.set_bounds(Some(bounds(header.bounds)));
    let flags = header.flags();
    let mut unknown = flags;
    unknown.set_no_vertex_attenuation(false);
    unknown.set_unified_render_path(false);
    unknown.set_liquid_type_database_id(false);
    unknown.set_preserve_vertex_alpha(false);
    unknown.set_default_max_lod(false);
    unknown.set_has_group_lods(false);
    builder.set_metadata(ModelMetadata {
        id: header.wmo_id,
        ambient: rgba(header.ambient_color),
        options: WorldOptions {
            default_max_lod: flags.default_max_lod(),
            no_vertex_attenuation: flags.no_vertex_attenuation(),
            unified_rendering: flags.unified_render_path(),
            database_liquids: flags.liquid_type_database_id(),
            preserve_vertex_alpha: flags.preserve_vertex_alpha(),
            unclassified: unknown.bits().into(),
        },
        skybox: source.root.chunks().find_map(|chunk| {
            if let crate::model::formats::wmo::root::RootChunk::SkyboxName(bytes) = chunk {
                let bytes = bytes.as_bytes();
                Some(crate::model::Resource::Path(std::borrow::Cow::Borrowed(
                    &bytes[..bytes
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(bytes.len())],
                )))
            } else {
                None
            }
        }),
    });
    let materials = materials::read(&source.root, &mut builder)?;
    let mut world = WorldSystem::default();
    let owner = builder.add_system(WorldSystem::default())?;
    let identities = scene::read(&source.root, &mut builder, &mut world, owner, &mut context)?;
    environment::read(&source.root, &mut world, owner)?;
    visibility::read(&source.root, &mut world, owner)?;
    let mut keys: BTreeSet<_> = (0..header.group_count as usize)
        .map(|index| GroupKey { index, lod: 0 })
        .collect();
    for dependency in source
        .dependencies()
        .map_err(|error| Error::new("world.dependencies", error.to_string()))?
    {
        if let crate::model::formats::wmo::DependencyKind::Group { index, lod } = dependency.kind {
            keys.insert(GroupKey { index, lod });
        }
    }
    for key in keys {
        let group = source
            .group(key)
            .ok_or_else(|| Error::new("world.group", "required group companion is missing"))?;
        for chunk in group.chunks() {
            use crate::model::formats::wmo::group::GroupChunk;
            let unresolved = match chunk {
                GroupChunk::Unknown { tag, .. } => Some(*tag),
                GroupChunk::SpotLights(_) => Some(chunk.tag()),
                GroupChunk::PrepassBatches(_) => Some(chunk.tag()),
                GroupChunk::QueryGeometry(query) if query.extension.len() > 1 => Some(chunk.tag()),
                GroupChunk::SpotLightAnimations(_) => Some(chunk.tag()),
                GroupChunk::LightMapMaterials(_)
                | GroupChunk::LightMapData(_)
                | GroupChunk::LegacyIndices(_)
                | GroupChunk::LightMapCoordinates(_)
                | GroupChunk::LegacyBatchVertices(_)
                | GroupChunk::LegacyBatchIndices(_)
                | GroupChunk::LegacyBatchGeometry(_)
                | GroupChunk::LegacyBatchPlanes(_) => Some(chunk.tag()),
                _ => None,
            };
            if let Some(tag) = unresolved {
                handle_unknown(
                    &mut context,
                    source
                        .group_file_key(key)
                        .expect("resolved group identity")
                        .clone(),
                    tag,
                )?;
            }
        }
        let instance_id = geometry::read(
            group,
            (key, &source.root),
            &mut builder,
            &materials,
            &mut world,
            owner,
            &mut context,
        )?;
        let mut instance = builder
            .model()
            .instance(instance_id)
            .expect("created placement")
            .clone();
        let region = &mut instance.controls;
        region.file_id = source.group_file_key(key).and_then(|key| match key {
            FileKey::FileId(id) => Some(*id),
            _ => None,
        });
        relationships::read(&source.root, group, region, &identities, owner)?;
        region_lights::read(group, &mut builder, region)?;
        if world.fogs.is_empty() {
            region.fogs.clear();
        }
        region.liquids = liquids::read(group, &materials, flags.liquid_type_database_id())?;
        if group.header().flags.has_liquid() && region.liquids.is_empty() {
            region.implicit_liquid = Some(group.header().liquid_type);
        }
        if flags.liquid_type_database_id()
            && !region.liquids.is_empty()
            && group.header().liquid_type != 0
        {
            context.issue(
                "world.liquid.category",
                IssueKind::Normalized,
                "unused legacy header category omitted for database-addressed surface grids",
            );
        }
        builder.set_instance(instance_id, instance)?;
    }
    let mut catalog_index = 0usize;
    for chunk in source.root.chunks() {
        match chunk {
            crate::model::formats::wmo::root::RootChunk::Groups(rows) => {
                for row in rows.iter() {
                    let row =
                        row.map_err(|error| Error::new("world.catalog", error.to_string()))?;
                    let group = world.groups.get_mut(catalog_index).ok_or_else(|| {
                        Error::new("world.catalog", "catalog exceeds group count")
                    })?;
                    catalog_index += 1;
                    group.name = if row.name_offset < 0 {
                        std::borrow::Cow::Borrowed("")
                    } else {
                        relationships::name(&source.root, row.name_offset as u32)?
                    };
                    group.bounds = row.bounds;
                    let flags = row.flags;
                    group.options = group_flags::read(flags, 0);
                }
            }
            crate::model::formats::wmo::root::RootChunk::Groups2(rows) => {
                for (index, row) in rows.iter().enumerate() {
                    let row =
                        row.map_err(|error| Error::new("world.catalog", error.to_string()))?;
                    let group = world.groups.get_mut(index).ok_or_else(|| {
                        Error::new("world.catalog", "supplemental catalog exceeds group count")
                    })?;
                    group.supplemental = Some(row.flags2.bits().into());
                    group.detail_selection = Some(row.lod_index);
                }
            }
            _ => {}
        }
    }
    builder.set_system(owner, world)?;
    Ok(Conversion {
        value: builder.finish()?,
        report: context.report,
    })
}
fn rgba([b, g, r, a]: [u8; 4]) -> [u8; 4] {
    [r, g, b, a]
}
fn bounds([minimum, maximum]: [[f32; 3]; 2]) -> Bounds {
    Bounds {
        minimum,
        maximum,
        sphere_radius: minimum
            .iter()
            .zip(maximum)
            .map(|(a, b)| ((b - a) / 2.).powi(2))
            .sum::<f32>()
            .sqrt(),
    }
}

pub(super) fn plan<'a, D: Dialect>(
    asset: &'a Model<'_, Wmo>,
    options: &crate::model::conversion::ExportOptions,
) -> Result<crate::model::conversion::Plan<Bundle<'a, D>>, Error> {
    encode::write(asset, options)
}

fn handle_unknown(context: &mut Context, file: FileKey, tag: [u8; 4]) -> Result<(), Error> {
    context.unknown_data(
        format!("world.{file:?}.unknown.{}", String::from_utf8_lossy(&tag)),
        "unknown native payload has no semantic representation",
    )
}

pub(super) fn import_alpha<'a>(
    source: &'a crate::model::formats::wmo::legacy::Root14<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, Wmo>>, Error> {
    legacy_import::alpha(source, options)
}
pub(super) fn import_beta<'a>(
    source: &'a crate::model::formats::wmo::legacy::Bundle16<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, Wmo>>, Error> {
    legacy_import::beta(source, options)
}

pub(super) fn plan_alpha<'a>(
    asset: &'a Model<'_, Wmo>,
    options: &crate::model::conversion::ExportOptions,
) -> Result<crate::model::conversion::Plan<crate::model::formats::wmo::legacy::Root14<'a>>, Error> {
    legacy_encode::alpha(asset, options)
}
pub(super) fn plan_beta<'a>(
    asset: &'a Model<'_, Wmo>,
    options: &crate::model::conversion::ExportOptions,
) -> Result<crate::model::conversion::Plan<crate::model::formats::wmo::legacy::Bundle16<'a>>, Error>
{
    legacy_encode::beta(asset, options)
}
