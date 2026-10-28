//! Direct native root and companion construction from typed world semantics.
use super::{geometry, materials};
use crate::model::common::FileKey;
use crate::model::conversion::{Context, Conversion, Error, ExportOptions, IssueKind};
use crate::model::error::Location;
use crate::model::formats::wmo::codec::Array;
use crate::model::formats::wmo::records::{HeaderTailField, RootFlags};
use crate::model::formats::wmo::root::RootChunk;
use crate::model::formats::wmo::{Bundle, Dialect, GroupKey, Root};
use crate::model::formats::wmo::{RecordLayout, Write};
use crate::model::profiles::Wmo;
use crate::model::Model;
use crate::model::WorldModelResource;

pub(super) fn write<'a, D: Dialect>(
    asset: &'a Model<'_, Wmo>,
    options: &ExportOptions,
) -> Result<Conversion<Bundle<'a, D>>, Error> {
    asset.validate()?;
    let model = asset;
    let mut context = Context::export(options);
    if let Some(normalized) = super::normalize::prepare(model)? {
        context.issue("world.groups.geometry",IssueKind::Normalized,"compatible geometry placements at the same detail level joined while preserving draw order and material assignments");
        write_model(&normalized, options, context)
    } else {
        write_model(model, options, context)
    }
}
fn write_model<'a, D: Dialect>(
    model: &Model<'_, Wmo>,
    options: &ExportOptions,
    mut context: Context,
) -> Result<Conversion<Bundle<'a, D>>, Error> {
    if model.bounds().is_none() {
        context.unsupported(Location::new().field("world").field("bounds"), "destination world header requires a bounding declaration; absent source bounds cannot be represented")?;
    }
    let mut root = Root::<D>::new();
    let controls = model.metadata();
    let mut flags = RootFlags(0);
    flags.set_no_vertex_attenuation(controls.options.no_vertex_attenuation);
    flags.set_unified_render_path(controls.options.skip_ambient_in_vertex_color);
    flags.set_liquid_type_database_id(controls.options.database_liquids);
    flags.set_vertex_color_alpha_only(controls.options.vertex_color_alpha_only);
    flags.set_default_max_lod(controls.options.default_max_lod);
    root.header_mut().wmo_id = controls.id;
    root.header_mut().ambient_color = super::rgba(controls.ambient);
    if let Some(skybox) = controls.skybox.as_ref() {
        match model.model_resource(*skybox).expect("validated skybox") {
            WorldModelResource::Path(bytes) => {
                if bytes.contains(&0) {
                    return Err(Error::requirement(
                        Location::new().field("world").field("skybox"),
                        "skybox path contains an interior terminator",
                    ));
                }
                let mut bytes = bytes.to_vec();
                bytes.push(0);
                append(&mut root, RootChunk::SkyboxName(array(&bytes)?))?;
            }
            _ => context.unsupported(
                Location::new().field("world").field("skybox"),
                "native skybox selection requires a path",
            )?,
        }
    }
    let world = model
        .systems()
        .iter()
        .next()
        .map(|(_, world)| world)
        .ok_or_else(|| {
            Error::requirement(
                Location::new().field("world").field("system"),
                "world export requires a world-system collection",
            )
        })?;
    if model.systems().len() != 1 {
        return Err(Error::requirement(
            Location::new().field("world").field("systems"),
            "one root cannot contain independent world-system collections",
        ));
    }
    super::visibility::write(world, &mut root)?;
    super::environment::write(world, &mut root)?;
    let actor_indices = super::encode_scene::write(model, world, &mut root, &mut context)?;
    let material_ids = materials::write(model, &mut root, &mut context)?;
    let mut names = vec![0u8];
    let mut name_offsets = Vec::new();
    let mut region_names = std::collections::BTreeMap::new();
    for group in &world.groups {
        let name = names.len() as u32;
        names.extend_from_slice(group.name.as_bytes());
        names.push(0);
        name_offsets.push((name, 0));
        for id in &group.instances {
            let instance = model.instance(*id).expect("validated placement");
            let mesh = model.mesh(instance.mesh).expect("validated geometry");
            let region = &instance.properties;
            let name = names.len() as u32;
            names.extend_from_slice(mesh.name.as_bytes());
            names.push(0);
            let description = names.len() as u32;
            names.extend_from_slice(region.description.as_bytes());
            names.push(0);
            region_names.insert(*id, (name, description));
        }
    }
    append(&mut root, RootChunk::GroupNames(array(&names)?))?;
    let infos = world
        .groups
        .iter()
        .zip(&name_offsets)
        .map(|(group, (name, _))| {
            let flags = super::group_flags::catalog(model, group);
            crate::model::formats::wmo::records::GroupInfo {
                flags,
                bounds: (group.bounds).map(|v| v.to_array()),
                name_offset: *name as i32,
            }
        })
        .collect::<Vec<_>>();
    append(&mut root, RootChunk::Groups(array(&infos)?))?;
    if world
        .groups
        .iter()
        .any(|group| group.supplemental.is_some() || group.detail_selection.is_some())
    {
        let rows = world
            .groups
            .iter()
            .map(|group| crate::model::formats::wmo::records::GroupInfo2 {
                flags2: crate::model::formats::wmo::records::GroupFlags2(
                    group.supplemental.unwrap_or_default().bits(),
                ),
                lod_index: group.detail_selection.unwrap_or(0),
            })
            .collect::<Vec<_>>();
        append(&mut root, RootChunk::Groups2(array(&rows)?))?;
    }
    root.header_mut().group_count = world.groups.len() as u32;
    if let Some(bounds) = model.bounds() {
        root.header_mut().bounds = [(bounds.minimum).to_array(), (bounds.maximum).to_array()];
    }
    let max_lod = world
        .groups
        .iter()
        .flat_map(|group| {
            group.instances.iter().map(|id| {
                model
                    .instance(*id)
                    .expect("validated placement")
                    .lod
                    .unwrap_or(0)
            })
        })
        .max()
        .unwrap_or(0);
    flags.set_has_group_lods(max_lod != 0);
    root.header_mut().tail.set_flags(flags);
    if !root
        .header_mut()
        .tail
        .set_lod_count(u16::try_from(max_lod).map_err(|_| {
            Error::requirement(
                Location::new().field("world").field("lod"),
                "LOD exceeds header capacity",
            )
        })?)
    {
        context.unsupported(
            Location::new().field("world").field("lod"),
            "selected native root tail has no LOD declaration",
        )?;
    }
    let use_file_ids = world
        .groups
        .iter()
        .flat_map(|group| &group.instances)
        .any(|id| {
            model
                .instance(*id)
                .expect("validated placement")
                .properties
                .file_id
                .is_some()
        });
    let mut file_ids = vec![
        0u32;
        world
            .groups
            .len()
            .checked_mul(max_lod as usize + 1)
            .ok_or_else(|| Error::requirement(
                Location::new().field("world").field("groups"),
                "resource table size overflows"
            ))?
    ];
    if max_lod != 0 && !use_file_ids {
        return Err(Error::requirement(
            Location::new()
                .field("world")
                .field("groups")
                .field("resources"),
            "additional native LOD geometry requires assigned database resource identities",
        ));
    }
    if use_file_ids {
        for (index, group) in world.groups.iter().enumerate() {
            for id in &group.instances {
                let instance = model.instance(*id).expect("validated placement");
                let region = &instance.properties;
                let id=region.file_id.filter(|id| *id!=0).ok_or_else(||Error::requirement(Location::new().field("world").field("groups").field("resources"),"database-addressed group table requires a nonzero identity for every placed region"))?;
                file_ids[instance.lod.unwrap_or(0) as usize * world.groups.len() + index] = id;
            }
        }
        append(&mut root, RootChunk::GroupFileIds(array(&file_ids)?))?;
    }
    let mut bundle = Bundle::from_root(root);
    let mut portal_refs = Vec::new();
    for (index, group) in world.groups.iter().enumerate() {
        let mut lods = std::collections::BTreeSet::new();
        for mesh in &group.instances {
            let lod = model
                .instance(*mesh)
                .expect("validated placement")
                .lod
                .unwrap_or(0);
            if !lods.insert(lod) {
                return Err(Error::requirement(
                    Location::new()
                        .field("world")
                        .field("group")
                        .field("instances"),
                    "multiple placements at one detail level require geometry merging",
                ));
            }
            let mut document = geometry::write::<D>(
                model,
                *mesh,
                lod,
                region_names[mesh],
                &material_ids,
                options.look,
                &mut context,
            )?;
            let region = &model
                .instance(*mesh)
                .expect("validated placement")
                .properties;
            super::relationships::write(
                &mut document,
                region,
                world,
                &actor_indices,
                &mut portal_refs,
            )?;
            super::region_lights::write(model, region, &mut document)?;
            let key = GroupKey {
                index,
                lod: lod as usize,
            };
            bundle.insert_group(
                key,
                if let Some(id) = region.file_id {
                    FileKey::FileId(id)
                } else {
                    FileKey::Sibling {
                        owner: Box::new(FileKey::Root),
                        suffix: format!("_{index:03}.wmo"),
                    }
                },
                document,
            );
        }
    }
    if !portal_refs.is_empty() {
        append(
            &mut bundle.root,
            RootChunk::PortalRefs(array(&portal_refs)?),
        )?;
    }
    bundle.validate_complete().map_err(|error| {
        Error::from_source(Location::new().field("world").field("output"), error)
    })?;
    Ok(Conversion {
        value: bundle,
        report: context.report,
    })
}
pub(super) fn array<'a, T: Write + RecordLayout>(rows: &[T]) -> Result<Array<'a, T>, Error> {
    Array::from_records(rows)
        .map_err(|error| Error::from_source(Location::new().field("world").field("array"), error))
}
pub(super) fn append<'a, D: Dialect>(
    root: &mut impl super::root_output::RootOutput<'a, D>,
    chunk: RootChunk<'a, D>,
) -> Result<(), Error> {
    root.append(chunk)
}
