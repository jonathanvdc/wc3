//! Native Warcraft III export planning over independent logical models.
use super::billboard;
use super::encode_animation::{unsigned_frame, Encoder};
use crate::model::conversion::{Context, Error, ExportOptions, Plan};
use crate::model::error::Location;
use crate::model::formats::mdlx::mdx::FixedText;
use crate::model::formats::mdlx::scene::{Node as NativeNode, NodeFlags};
use crate::model::formats::mdlx::{
    Dialect, Document as NativeModel, ModelInfo, Sequence, SequenceFlags, Texture, TextureFlags,
};
use crate::model::profiles::Mdlx;
use crate::model::MdlxImageResource;
use crate::model::Model;
use crate::model::TransformSemantics;
use crate::model::{AddressMode, Billboard, NodeId, Quat};
use std::collections::BTreeMap;

pub(in crate::model::io::mdlx) fn write<'a, 'data, T: Dialect>(
    asset: &'a Model<'data, Mdlx>,
    options: &ExportOptions,
) -> Result<Plan<NativeModel<'a, T>>, Error> {
    asset.validate()?;
    let model = asset;
    // These fields share the native MODL declaration with its mandatory extent.
    // Their independent semantic authorship does not imply an authored bound.
    if model.bounds().is_none()
        && (!model.name().is_empty()
            || !model.metadata().animation_file.is_empty()
            || model.metadata().blend_time != 0.)
    {
        return Err(Error::requirement(Location::new().field("model").field("info"),
            "the native model-info declaration requires bounds to encode the model name, animation filename, or blend time",
        ));
    }
    let mut context = Context::export(options);
    if model.systems().iter().next().is_some() {
        context.unsupported(
            Location::new().field("model").field("systems"),
            "Warcraft III cannot encode the authored global system without semantic translation",
        )?;
    }
    let mut result = NativeModel::<T>::new();
    let mut globals = BTreeMap::new();
    let mut periods = Vec::new();
    for (id, clock) in model.clocks() {
        globals.insert(id, periods.len() as u32);
        periods.push(unsigned_frame(
            clock.period().as_seconds(),
            &mut context,
            &Location::new().field("clock").field("period"),
        )?);
    }
    result.set_global_sequences(&periods);
    let mut sequences = Vec::new();
    for (_, clip) in model.clips() {
        let metadata = &clip.properties;
        let start = clip.source_start.as_seconds();
        let first = unsigned_frame(
            start,
            &mut context,
            &Location::new().field("clip").field("start"),
        )?;
        let last = unsigned_frame(
            start + clip.duration().as_seconds(),
            &mut context,
            &Location::new().field("clip").field("end"),
        )?;
        let mut sequence = Sequence::new(clip.name(), [first, last]).map_err(|error| {
            Error::from_source(Location::new().field("clip").field("name"), error)
        })?;
        sequence.flags.set_non_looping(!clip.is_looping());
        {
            sequence.flags = SequenceFlags(0);
            sequence.flags.set_non_looping(!clip.is_looping());
            sequence.move_speed = metadata.move_speed;
            sequence.rarity = metadata.rarity;
            sequence.sync_point = metadata.sync_point;
            sequence.extent = super::extents::write(metadata.extent);
        }
        sequences.push(sequence);
    }
    result.set_sequences(sequences);
    let mut textures = Vec::new();
    let mut texture_ids = BTreeMap::new();
    for source in asset.sampled_images() {
        let id = source;
        let path: &[u8] = match asset
            .image_resource(source.resource)
            .expect("validated image")
        {
            MdlxImageResource::Path(bytes) => bytes.as_ref(),
            MdlxImageResource::Replaceable { filename, .. } => filename.as_ref(),
        };
        if path.len() > 260 || path.contains(&0) {
            return Err(Error::requirement(
                Location::new().field("texture").field("path"),
                "resource path exceeds the native field or contains NUL",
            ));
        }
        let mut value = Texture::new("").map_err(|error| {
            Error::from_source(Location::new().field("texture").field("path"), error)
        })?;
        let mut field = [0u8; 260];
        field[..path.len()].copy_from_slice(path);
        value.path = FixedText::from_bytes(field);
        if let MdlxImageResource::Replaceable { id: category, .. } = asset
            .image_resource(source.resource)
            .expect("validated image")
        {
            value.replaceable_id = *category;
        }
        value.flags = TextureFlags::default();
        if !matches!(source.sampler.u, AddressMode::Repeat | AddressMode::Clamp)
            || !matches!(source.sampler.v, AddressMode::Repeat | AddressMode::Clamp)
        {
            context.unsupported(
                Location::new().field("texture").field("sampler"),
                "Warcraft III texture addressing supports only repeat and clamp",
            )?;
        }
        value
            .flags
            .set_wrap_width(source.sampler.u == AddressMode::Repeat);
        value
            .flags
            .set_wrap_height(source.sampler.v == AddressMode::Repeat);
        let index = textures
            .iter()
            .position(|existing| existing == &value)
            .unwrap_or_else(|| {
                textures.push(value);
                textures.len() - 1
            });
        texture_ids.insert(id, index as u32);
    }
    result.set_textures(textures);
    let mut encoder = Encoder {
        model,
        globals,
        context: &mut context,
        used: Default::default(),
    };
    let mut materials = Vec::new();
    let mut transforms = Vec::new();
    let mut material_ids = BTreeMap::new();
    for (id, source) in model.materials() {
        material_ids.insert(id, materials.len() as u32);
        materials.push(super::encode_materials::write::<T::Version>(
            &source.surface,
            &texture_ids,
            &mut encoder,
            &mut transforms,
            &Location::new().field("material"),
        )?);
    }
    result.set_materials(materials);
    result.set_texture_animations(transforms);
    let node_ids: BTreeMap<NodeId, u32> = model
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id, index as u32))
        .collect();
    let mut nodes = BTreeMap::new();
    let mut pivots = Vec::new();
    for (id, source) in model.nodes() {
        let mut node = NativeNode::new(&source.name, node_ids[&id]).map_err(|error| {
            Error::from_source(Location::new().field("node").field("name"), error)
        })?;
        node.parent_id = source.parent.map_or(u32::MAX, |parent| node_ids[&parent]);
        node.flags = NodeFlags(0);
        node.flags
            .set_dont_inherit_translation(!source.inheritance.translation);
        node.flags
            .set_dont_inherit_rotation(!source.inheritance.rotation);
        node.flags
            .set_dont_inherit_scaling(!source.inheritance.scale);
        node.flags.set_camera_anchored(source.camera_anchored);
        if source.transform_semantics != TransformSemantics::PivotLocal
            && (!source.inheritance.translation
                || !source.inheritance.rotation
                || !source.inheritance.scale
                || source.billboard != Billboard::None
                || source.camera_anchored)
        {
            encoder.context.unsupported(Location::new().field("node").field("transform_semantics"), "Warcraft III requires pivot-local composition for selective inheritance and camera controls")?;
        }
        let flags = billboard::encode(
            source.billboard,
            source.properties.billboard_flags,
            encoder.context,
        )?;
        node.flags.set_billboarded(flags.enabled);
        node.flags.set_billboard_lock_x(flags.x);
        node.flags.set_billboard_lock_y(flags.y);
        node.flags.set_billboard_lock_z(flags.z);
        if !encoder.constant(source.visible) {
            encoder.context.unsupported(
                Location::new().field("node").field("visible"),
                "generic Warcraft III hierarchy nodes have no static hidden flag",
            )?;
        }
        node.translation = encoder.optional(
            source.transform.translation,
            |v| Ok(v.to_array()),
            [0.; 3],
            &Location::new().field("node").field("translation"),
        )?;
        node.rotation = encoder.optional(
            source.transform.rotation,
            |value: Quat| Ok(value.to_array()),
            Quat::default().to_array(),
            &Location::new().field("node").field("rotation"),
        )?;
        node.scaling = encoder.optional(
            source.transform.scale,
            |v| Ok(v.to_array()),
            [1.; 3],
            &Location::new().field("node").field("scale"),
        )?;
        pivots.push(source.transform.pivot);
        nodes.insert(id, node);
    }
    result.set_pivot_points(&pivots.iter().map(|v| v.to_array()).collect::<Vec<_>>());
    let meshes = super::encode_geometry::write(
        &mut result,
        &mut encoder,
        &node_ids,
        &material_ids,
        options.look,
    )?;
    super::encode_scene::write(
        &mut result,
        &mut encoder,
        nodes,
        &texture_ids,
        &material_ids,
        &meshes,
    )?;
    if let Some(bounds) = model.bounds() {
        let mut info = ModelInfo::new(model.name()).map_err(|error| {
            Error::from_source(Location::new().field("model").field("name"), error)
        })?;
        info.minimum_extent = (bounds.minimum).to_array();
        info.maximum_extent = (bounds.maximum).to_array();
        info.bounds_radius = bounds.sphere_radius;
        {
            let metadata = model.metadata();
            info.animation_file_name
                .set_text(&metadata.animation_file)
                .map_err(|error| {
                    Error::from_source(
                        Location::new().field("model").field("animation_file"),
                        error,
                    )
                })?;
            info.blend_time = unsigned_frame(
                metadata.blend_time,
                encoder.context,
                &Location::new().field("model").field("blend_time"),
            )?;
        }
        result.set_model_info(&info);
    }
    encoder.audit()?;
    Ok(Plan::new(result, context.report))
}
