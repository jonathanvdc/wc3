//! Native Warcraft III export planning over independent logical models.
use super::encode_animation::{unsigned_frame, Encoder};
use crate::model::conversion::{Context, Error, ExportOptions, Plan};
use crate::model::mdlx::animation::Track;
use crate::model::mdlx::mdx::FixedText;
use crate::model::mdlx::scene::{Node as NativeNode, NodeFlags};
use crate::model::mdlx::{
    GeosetExtent, Model as NativeModel, ModelDialect, ModelInfo, Sequence, SequenceFlags, Texture,
    TextureFlags,
};
use crate::model::semantic::profiles::Mdlx;
use crate::model::semantic::Model;
use crate::model::semantic::{AddressMode, Billboard, NodeId, Quaternion, Resource};
use std::collections::BTreeMap;

pub(in crate::model::conversion::mdlx) fn write<'a, 'data, T: ModelDialect>(
    asset: &'a Model<'data, Mdlx>,
    options: &ExportOptions,
) -> Result<Plan<NativeModel<'a, T>>, Error> {
    asset.validate()?;
    let model = asset;
    let mut context = Context::export(options);
    if model.systems().next().is_some() {
        context.unsupported(
            "model.systems",
            "Warcraft III cannot encode the authored global system without semantic translation",
        )?;
    }
    let mut result = NativeModel::<T>::new();
    let mut globals = BTreeMap::new();
    let mut periods = Vec::new();
    for (id, clock) in model.clocks() {
        globals.insert(id, periods.len() as u32);
        periods.push(unsigned_frame(
            clock.period.as_seconds(),
            &mut context,
            "clock.period",
        )?);
    }
    result.set_global_sequences(&periods);
    let mut sequences = Vec::new();
    let mut starts = BTreeMap::new();
    for (id, clip) in model.clips() {
        let metadata = &clip.controls;
        let start = metadata.start.as_seconds();
        let first = unsigned_frame(start, &mut context, "clip.start")?;
        let last = unsigned_frame(
            start + clip.duration().as_seconds(),
            &mut context,
            "clip.end",
        )?;
        let mut sequence = Sequence::new(clip.name(), [first, last])
            .map_err(|error| Error::new("clip.name", error.to_string()))?;
        sequence.flags.set_non_looping(!clip.is_looping());
        {
            sequence.flags = SequenceFlags(metadata.unclassified.bits());
            sequence.flags.set_non_looping(!clip.is_looping());
            sequence.move_speed = metadata.move_speed;
            sequence.rarity = metadata.rarity;
            sequence.sync_point = metadata.sync_point;
            sequence.extent = GeosetExtent {
                minimum: metadata.bounds[0],
                maximum: metadata.bounds[1],
                bounds_radius: metadata.radius,
            };
        }
        starts.insert(id, start);
        sequences.push(sequence);
    }
    result.set_sequences(sequences);
    let mut textures = Vec::new();
    let mut texture_ids = BTreeMap::new();
    for (id, source) in asset.resources().textures() {
        texture_ids.insert(id, textures.len() as u32);
        let metadata = &source.controls;
        let path: &[u8] = match &source.resource {
            Resource::Path(bytes) => bytes.as_ref(),
            Resource::Replaceable(_) => metadata.fallback_path.as_ref(),
            _ => {
                context.unsupported(
                    "texture.resource",
                    "Warcraft III textures require a filename or replaceable category",
                )?;
                &[]
            }
        };
        if path.len() > 260 || path.contains(&0) {
            return Err(Error::new(
                "texture.path",
                "resource path exceeds the native field or contains NUL",
            ));
        }
        let mut value =
            Texture::new("").map_err(|error| Error::new("texture.path", error.to_string()))?;
        let mut field = [0u8; 260];
        field[..path.len()].copy_from_slice(path);
        value.path = FixedText::from_bytes(field);
        if let Resource::Replaceable(category) = source.resource {
            value.replaceable_id = category;
        }
        value.flags = TextureFlags(metadata.unclassified.bits());
        if !matches!(source.sampler.u, AddressMode::Repeat | AddressMode::Clamp)
            || !matches!(source.sampler.v, AddressMode::Repeat | AddressMode::Clamp)
        {
            context.unsupported(
                "texture.sampler",
                "Warcraft III texture addressing supports only repeat and clamp",
            )?;
        }
        value
            .flags
            .set_wrap_width(source.sampler.u == AddressMode::Repeat);
        value
            .flags
            .set_wrap_height(source.sampler.v == AddressMode::Repeat);
        textures.push(value);
    }
    result.set_textures(textures);
    let mut encoder = Encoder {
        model,
        starts,
        globals,
        context: &mut context,
        used: Default::default(),
    };
    let mut materials = Vec::new();
    let mut material_ids = BTreeMap::new();
    for (id, source) in model.materials() {
        material_ids.insert(id, materials.len() as u32);
        materials.push(super::encode_materials::write::<T::Version>(
            &source.surface,
            &texture_ids,
            &mut encoder,
            "material",
        )?);
    }
    result.set_materials(materials);
    let node_ids: BTreeMap<NodeId, u32> = model
        .nodes()
        .enumerate()
        .map(|(index, (id, _))| (id, index as u32))
        .collect();
    let mut nodes = BTreeMap::new();
    let mut pivots = Vec::new();
    for (id, source) in model.nodes() {
        let mut node = NativeNode::new(&source.name, node_ids[&id])
            .map_err(|error| Error::new("node.name", error.to_string()))?;
        node.parent_id = source.parent.map_or(u32::MAX, |parent| node_ids[&parent]);
        node.flags = NodeFlags(source.controls.unclassified.bits());
        node.flags
            .set_dont_inherit_translation(!source.inheritance.translation);
        node.flags
            .set_dont_inherit_rotation(!source.inheritance.rotation);
        node.flags
            .set_dont_inherit_scaling(!source.inheritance.scale);
        node.flags.set_camera_anchored(source.camera_anchored);
        match source.billboard {
            Billboard::None => {}
            Billboard::Spherical => node.flags.set_billboarded(true),
            Billboard::Locked { enabled, x, y, z } => {
                node.flags.set_billboarded(enabled);
                node.flags.set_billboard_lock_x(x);
                node.flags.set_billboard_lock_y(y);
                node.flags.set_billboard_lock_z(z);
            }
        }
        if !source.visible {
            encoder.context.unsupported(
                "node.visible",
                "generic Warcraft III hierarchy nodes have no static hidden flag",
            )?;
        }
        node.translation = encoder.track(&id.translation(), Ok, "node.translation")?;
        node.rotation = encoder.track(
            &id.rotation(),
            |value: Quaternion| Ok(value.components()),
            "node.rotation",
        )?;
        node.scaling = encoder.track(&id.scale(), Ok, "node.scale")?;
        if node.translation.is_none() && source.transform.translation != [0.; 3] {
            node.translation = Some(Track::constant(source.transform.translation));
        }
        if node.rotation.is_none() && source.transform.rotation != Quaternion::default() {
            node.rotation = Some(Track::constant(source.transform.rotation.components()));
        }
        if node.scaling.is_none() && source.transform.scale != [1.; 3] {
            node.scaling = Some(Track::constant(source.transform.scale));
        }
        pivots.push(source.transform.pivot);
        nodes.insert(id, node);
    }
    result.set_pivot_points(&pivots);
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
        let mut info = ModelInfo::new(model.name())
            .map_err(|error| Error::new("model.name", error.to_string()))?;
        info.minimum_extent = bounds.minimum;
        info.maximum_extent = bounds.maximum;
        info.bounds_radius = bounds.sphere_radius;
        {
            let metadata = model.metadata();
            info.animation_file_name
                .set_text(&metadata.animation_file)
                .map_err(|error| Error::new("model.animation_file", error.to_string()))?;
            info.blend_time =
                unsigned_frame(metadata.blend_time, encoder.context, "model.blend_time")?;
        }
        result.set_model_info(&info);
    }
    encoder.audit()?;
    Ok(Plan::new(result, context.report))
}
