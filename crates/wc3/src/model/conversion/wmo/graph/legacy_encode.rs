//! Direct construction of monolithic and split historical world documents.
use super::{
    alpha_materials, encode, encode_scene, environment, legacy_encode_geometry, materials,
    relationships, root_output::RootOutput, visibility,
};
use crate::model::common::FileKey;
use crate::model::conversion::{Asset, Context, Error, ExportOptions, IssueKind, Plan};
use crate::model::semantic::profiles::wmo::{
    Group as WorldGroup, ModelControls, RegionControls, WorldSystem,
};
use crate::model::semantic::profiles::Wmo;
use crate::model::semantic::{InstanceId, Model, Quaternion, Resource, Resources};
use crate::model::wmo::legacy::{
    Batch14, Batch16, Bundle16, Group14, Group16, GroupInfo14, HeaderBatchRange, Root14, Root16,
};
use crate::model::wmo::records::{GroupFlags2, GroupInfo, HeaderTailField, RootFlags};
use crate::model::wmo::root::RootChunk;
use crate::model::wmo::Classic;
use std::collections::BTreeMap;

pub(super) fn alpha<'a>(
    asset: &'a Asset<'_, Wmo>,
    options: &ExportOptions,
) -> Result<Plan<Root14<'a>>, Error> {
    let model = asset.model(options)?;
    let mut context = Context::export(options);
    if let Some((model, resources)) = super::normalize::prepare(model, asset.resources())? {
        context.issue("world.groups.geometry", IssueKind::Normalized, "compatible geometry placements joined with relocated historical vertex and lighting selections");
        alpha_model(&model, &resources, options, context)
    } else {
        alpha_model(model, asset.resources(), options, context)
    }
}
fn alpha_model<'a>(
    model: &Model<'_, Wmo>,
    resources: &Resources<'_, Wmo>,
    options: &ExportOptions,
    mut context: Context,
) -> Result<Plan<Root14<'a>>, Error> {
    let world = world(model)?;
    let mut root = Root14::new();
    let controls = model.controls();
    if model.bounds().is_some() || controls.options != Default::default() {
        context.unsupported(
            "world.root",
            "historical monolithic root has no authored global bounds or later rendering options",
        )?;
    }
    for (_, feature) in model.features() {
        if let crate::model::semantic::profiles::wmo::Node::Light(light) = &feature.payload {
            let node = model.node(feature.node).expect("validated light anchor");
            if node.transform.rotation != Quaternion::default() || light.modern.is_some() {
                context.unsupported(
                    "world.light",
                    "historical monolithic light has no orientation or later light extension",
                )?;
            }
        }
    }
    root.header_mut().wmo_id = controls.id;
    root.header_mut().ambient_color = super::rgba(controls.ambient);
    skybox(controls, &mut root, &mut context)?;
    visibility::write(world, &mut root)?;
    environment::write(world, &mut root)?;
    let actors = encode_scene::write(model, resources, world, &mut root, &mut context)?;
    let material_ids = alpha_materials::write(model, resources, &mut root, &mut context)?;
    let names = names(model, world)?;
    root.append(RootChunk::GroupNames(encode::array(&names.bytes)?))?;
    let mut portals = Vec::new();
    for (index, catalog) in world.groups.iter().enumerate() {
        let instance_id = placement(catalog, model)?;
        let instance = model.instance(instance_id).expect("checked placement");
        let mesh = model.mesh(instance.mesh).expect("validated mesh");
        let region = &instance.controls;
        let body = legacy_encode_geometry::write::<Batch14>(
            model,
            resources,
            instance_id,
            &material_ids,
            options.look,
            &mut context,
        )?;
        let mut group = Group14::new();
        group.chunks = body.chunks;
        group.header.flags = body.flags;
        group.header.name_offset = names.regions[&instance_id].0;
        group.header.descriptive_name_offset = names.regions[&instance_id].1;
        group.header.bounds_minimum = mesh.bounds.minimum;
        group.header.bounds_maximum = mesh.bounds.maximum;
        group.header.unique_id = i32::from_ne_bytes(region.area_id.to_ne_bytes());
        group.header.liquid_type = liquid_type(region)?;
        group.header.fog_indices = fogs(region)?;
        group.header.interior_batches = region.interior_windows.unwrap_or_default().map(
            |[vertex_start, vertex_count, batch_start, batch_count]| HeaderBatchRange {
                vertex_start,
                vertex_count,
                batch_start,
                batch_count,
            },
        );
        group.header.exterior_batches = region.exterior_windows.unwrap_or_default().map(
            |[vertex_start, vertex_count, batch_start, batch_count]| HeaderBatchRange {
                vertex_start,
                vertex_count,
                batch_start,
                batch_count,
            },
        );
        if region.interior_windows.is_none() && region.exterior_windows.is_none() {
            context.unsupported(
                "world.group.lighting",
                "historical monolithic header requires explicit interior/exterior geometry windows",
            )?;
        }
        if region.options.supplemental.bits() != 0
            || region.batch_control != 0
            || region.supplemental_controls.len() > 2
        {
            context.unsupported("world.group.controls","historical monolithic header cannot encode extra classification or more than two supplemental controls")?;
        }
        for (bytes, value) in group
            .header
            .reserved_tail
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(&region.supplemental_controls)
        {
            bytes.copy_from_slice(&value.to_le_bytes());
        }
        relationships::write(&mut group, region, world, &actors, &mut portals)?;
        root.push_group(
            group,
            GroupInfo14 {
                flags: super::group_flags::catalog(model, catalog),
                bounds: catalog.bounds,
                name_offset: i32::try_from(names.catalog[index]).map_err(error)?,
                ..Default::default()
            },
        )
        .map_err(error)?;
    }
    if !portals.is_empty() {
        root.append(RootChunk::PortalRefs(encode::array(&portals)?))?;
    }
    root.synchronize_counts().map_err(error)?;
    root.validate().map_err(error)?;
    Ok(Plan::new(root, context.report))
}
pub(super) fn beta<'a>(
    asset: &'a Asset<'_, Wmo>,
    options: &ExportOptions,
) -> Result<Plan<Bundle16<'a>>, Error> {
    let model = asset.model(options)?;
    let mut context = Context::export(options);
    if let Some((model, resources)) = super::normalize::prepare(model, asset.resources())? {
        context.issue("world.groups.geometry", IssueKind::Normalized, "compatible geometry placements joined with relocated historical vertex and lighting selections");
        beta_model(&model, &resources, options, context)
    } else {
        beta_model(model, asset.resources(), options, context)
    }
}
fn beta_model<'a>(
    model: &Model<'_, Wmo>,
    resources: &Resources<'_, Wmo>,
    options: &ExportOptions,
    mut context: Context,
) -> Result<Plan<Bundle16<'a>>, Error> {
    let world = world(model)?;
    if model.bounds().is_none() {
        context.unsupported("world.bounds", "destination world header requires a bounding declaration; absent source bounds cannot be represented")?;
    }
    let mut root = Root16::new();
    let controls = model.controls();
    root.header_mut().wmo_id = controls.id;
    root.header_mut().ambient_color = super::rgba(controls.ambient);
    root.header_mut().group_count = u32::try_from(world.groups.len()).map_err(error)?;
    if let Some(bounds) = model.bounds() {
        root.header_mut().bounds = [bounds.minimum, bounds.maximum];
    }
    let mut flags = RootFlags(controls.options.unclassified.bits());
    flags.set_no_vertex_attenuation(controls.options.no_vertex_attenuation);
    flags.set_unified_render_path(controls.options.unified_rendering);
    flags.set_liquid_type_database_id(controls.options.database_liquids);
    flags.set_preserve_vertex_alpha(controls.options.preserve_vertex_alpha);
    flags.set_default_max_lod(controls.options.default_max_lod);
    root.header_mut().tail.set_flags(flags);
    skybox(controls, &mut root, &mut context)?;
    visibility::write(world, &mut root)?;
    environment::write(world, &mut root)?;
    let actors = encode_scene::write(model, resources, world, &mut root, &mut context)?;
    let material_ids = materials::write(model, resources, &mut root, &mut context)?;
    let names = names(model, world)?;
    root.append(RootChunk::GroupNames(encode::array(&names.bytes)?))?;
    let infos = world
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| {
            Ok(GroupInfo {
                flags: super::group_flags::catalog(model, group),
                bounds: group.bounds,
                name_offset: i32::try_from(names.catalog[index]).map_err(error)?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    root.append(RootChunk::Groups(encode::array(&infos)?))?;
    let mut bundle = Bundle16::from_root(root);
    let mut portals = Vec::new();
    for (index, catalog) in world.groups.iter().enumerate() {
        let instance_id = placement(catalog, model)?;
        let instance = model.instance(instance_id).expect("checked placement");
        let mesh = model.mesh(instance.mesh).expect("validated mesh");
        let region = &instance.controls;
        if region.interior_windows.is_some()
            || region.exterior_windows.is_some()
            || region.supplemental_controls.len() > 1
        {
            context.unsupported("world.group.controls","historical split header cannot encode vertex lighting windows or multiple supplemental controls")?;
        }
        let body = legacy_encode_geometry::write::<Batch16>(
            model,
            resources,
            instance_id,
            &material_ids,
            options.look,
            &mut context,
        )?;
        let mut group = Group16::new();
        group.chunks = body.chunks;
        group.header.flags = body.flags;
        group.header.name_offset = names.regions[&instance_id].0;
        group.header.descriptive_name_offset = names.regions[&instance_id].1;
        group.header.bounds_minimum = mesh.bounds.minimum;
        group.header.bounds_maximum = mesh.bounds.maximum;
        group.header.unique_id = i32::from_ne_bytes(region.area_id.to_ne_bytes());
        group.header.liquid_type = liquid_type(region)?;
        group.header.fog_indices = fogs(region)?;
        group.header.flags2 = GroupFlags2(region.options.supplemental.bits());
        group.header.reserved_tail = region.supplemental_controls.first().copied().unwrap_or(0);
        group.header.reserved_batch_count = region.batch_control;
        [
            group.header.transition_batch_count,
            group.header.interior_batch_count,
            group.header.exterior_batch_count,
        ] = body.counts;
        relationships::write(&mut group, region, world, &actors, &mut portals)?;
        bundle.insert_group(
            index,
            FileKey::Root.sibling(format!("_{index:03}.wmo")),
            group,
        );
    }
    if !portals.is_empty() {
        bundle
            .root
            .append(RootChunk::PortalRefs(encode::array(&portals)?))?;
    }
    bundle.root.synchronize_counts().map_err(error)?;
    bundle.validate_complete().map_err(error)?;
    Ok(Plan::new(bundle, context.report))
}
struct Names {
    bytes: Vec<u8>,
    catalog: Vec<u32>,
    regions: BTreeMap<InstanceId, (u32, u32)>,
}
fn names(model: &Model<'_, Wmo>, world: &WorldSystem<'_>) -> Result<Names, Error> {
    let mut result = Names {
        bytes: vec![0],
        catalog: Vec::new(),
        regions: BTreeMap::new(),
    };
    for group in &world.groups {
        result.catalog.push(name(&mut result.bytes, &group.name)?);
        for id in &group.instances {
            let instance = model.instance(*id).expect("validated placement");
            let mesh = model.mesh(instance.mesh).expect("validated mesh");
            let display = name(&mut result.bytes, &mesh.name)?;
            let description = name(&mut result.bytes, &instance.controls.description)?;
            result.regions.insert(*id, (display, description));
        }
    }
    Ok(result)
}
fn name(bytes: &mut Vec<u8>, name: &str) -> Result<u32, Error> {
    if name.as_bytes().contains(&0) {
        return Err(Error::new(
            "world.name",
            "name contains an interior terminator",
        ));
    }
    let start = u32::try_from(bytes.len()).map_err(error)?;
    bytes.extend_from_slice(name.as_bytes());
    bytes.push(0);
    Ok(start)
}
fn skybox<'a>(
    controls: &ModelControls<'_>,
    root: &mut impl RootOutput<'a, Classic>,
    context: &mut Context,
) -> Result<(), Error> {
    if let Some(resource) = &controls.skybox {
        match resource {
            Resource::Path(path) => {
                if path.contains(&0) {
                    return Err(Error::new(
                        "world.skybox",
                        "skybox path contains an interior terminator",
                    ));
                }
                let mut bytes = path.to_vec();
                bytes.push(0);
                root.append(RootChunk::SkyboxName(encode::array(&bytes)?))?;
            }
            _ => context.unsupported(
                "world.skybox",
                "historical skybox requires a model file path",
            )?,
        }
    }
    Ok(())
}
fn world<'a, 'data>(model: &'a Model<'data, Wmo>) -> Result<&'a WorldSystem<'data>, Error> {
    model
        .systems()
        .next()
        .map(|(_, system)| system)
        .ok_or_else(|| Error::new("world.system", "world system is missing"))
}
fn placement(group: &WorldGroup<'_>, model: &Model<'_, Wmo>) -> Result<InstanceId, Error> {
    let [instance] = group.instances.as_slice() else {
        return Err(Error::new(
            "world.group.instances",
            "historical group requires one base geometry placement",
        ));
    };
    if model
        .instance(*instance)
        .expect("validated placement")
        .lod
        .is_some_and(|lod| lod != 0)
    {
        return Err(Error::new(
            "world.group.lod",
            "historical group cannot encode additional detail levels",
        ));
    }
    Ok(*instance)
}
fn liquid_type(region: &RegionControls<'_>) -> Result<u32, Error> {
    super::liquids::category(region, false)
}
fn fogs(region: &RegionControls<'_>) -> Result<[u8; 4], Error> {
    if region.fogs.len() > 4 {
        return Err(Error::new(
            "world.group.fogs",
            "historical header cannot encode more than four fog selections",
        ));
    }
    let mut fogs = [u8::MAX; 4];
    for (destination, source) in fogs.iter_mut().zip(&region.fogs) {
        *destination = u8::try_from(source.index).map_err(error)?;
    }
    Ok(fogs)
}
fn error(error: impl std::fmt::Display) -> Error {
    Error::new("world.historical", error.to_string())
}
