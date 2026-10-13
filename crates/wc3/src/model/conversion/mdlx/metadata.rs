//! Field-specific native metadata without source-file snapshots.
use super::*;
pub(super) fn append<T: wem::Record + wem::Write, U: wem::Target>(
    document: &mut RawDocument<'static>,
    values: &[T],
) -> Result<Ref<U>, Error> {
    let id = document.append(values)?;
    Ok(Ref::from_section(values.len() as u32, id.index()))
}

pub(super) fn append_string(
    document: &mut RawDocument<'static>,
    value: &str,
) -> Result<Ref<wr::targets::Character>, Error> {
    let values: Vec<_> = value
        .as_bytes()
        .iter()
        .map(|byte| wr::Character { value: *byte })
        .collect();
    append(document, &values)
}

pub(super) fn base<T: TrackValue>(value: &native::Animatable<'_, T>) -> T {
    value.value().copied().unwrap_or_default()
}

pub(super) fn set_metadata(
    raw: &mut RawDocument<'static>,
    bag: &mut wr::NativeBag,
    name: &str,
    value: i64,
    body: &str,
) -> Result<(), Error> {
    let mut entries = raw
        .resolve::<wr::NativeEntry<wr::V2>>(bag.entries.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let entry = wr::NativeEntry::<wr::V2> {
        name: append_string(raw, name)?,
        value,
        text: append_string(raw, body)?,
    };
    entries.push(entry);
    bag.entries = append(raw, &entries)?;
    Ok(())
}

pub(super) fn metadata(
    raw: &RawDocument<'_>,
    bag: wr::NativeBag,
) -> Result<BTreeMap<String, (i64, String)>, Error> {
    raw.resolve::<wr::NativeEntry<wr::V2>>(bag.entries.cast())?
        .iter()
        .map(|entry| {
            let entry = entry?;
            Ok((
                raw.string(entry.name)?.unwrap_or_default().to_owned(),
                (
                    entry.value,
                    raw.string(entry.text)?.unwrap_or_default().to_owned(),
                ),
            ))
        })
        .collect()
}

pub(super) fn values_text(values: impl IntoIterator<Item = f32>) -> String {
    values
        .into_iter()
        .map(|value| format!("{:08x}", value.to_bits()))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn parse_values(text: &str) -> Result<Vec<f32>, Error> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    text.split(',')
        .map(|value| {
            u32::from_str_radix(value, 16)
                .map(f32::from_bits)
                .map_err(|error| failure("metadata.float", error))
        })
        .collect()
}

pub(super) fn extent_text(extent: native::GeosetExtent) -> String {
    values_text(
        extent
            .minimum
            .into_iter()
            .chain(extent.maximum)
            .chain([extent.bounds_radius]),
    )
}

pub(super) fn extent_from_text(value: &str) -> Result<native::GeosetExtent, Error> {
    let values = parse_values(value)?;
    if values.len() != 7 {
        return Err(Error::new(
            "metadata.extent",
            "extent needs seven components",
        ));
    }
    Ok(native::GeosetExtent {
        minimum: values[..3].try_into().unwrap(),
        maximum: values[3..6].try_into().unwrap(),
        bounds_radius: values[6],
    })
}

pub(super) fn decorate_metadata<D: ModelDialect>(
    document: wem::Document<'static>,
    source: &native::Model<'_, D>,
) -> Result<wem::Document<'static>, Error> {
    let mut raw = document.into_raw();
    let root = raw
        .resolve::<wr::DocumentRecord<wr::V1>>(Ref::new(1, raw.root_reference().index, 0))?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("root", "missing root"))?;
    let model = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("model", "missing model"))?;
    let mut profiles = raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let profile = &mut profiles[0];
    set_metadata(
        &mut raw,
        &mut profile.native,
        "mdlx.source_version",
        i64::from(D::Version::NUMBER),
        "",
    )?;
    if let Some(info) = source.model_info() {
        set_metadata(
            &mut raw,
            &mut profile.native,
            "mdlx.animation_file_name",
            0,
            &text(&info.animation_file_name),
        )?;
        set_metadata(
            &mut raw,
            &mut profile.native,
            "mdlx.blend_time",
            i64::from(info.blend_time),
            "",
        )?;
    }
    set_metadata(
        &mut raw,
        &mut profile.native,
        "mdlx.global_sequences",
        0,
        &source
            .global_sequences()
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(","),
    )?;
    set_metadata(
        &mut raw,
        &mut profile.native,
        "mdlx.gliders",
        0,
        &source
            .gliders()
            .map(|record| record.geoset_id.to_string())
            .collect::<Vec<_>>()
            .join(","),
    )?;
    let id = raw.section_id(model.profile_sets.index).unwrap();
    raw.replace(id, &profiles)?;
    let mut clips = raw
        .resolve::<wr::Clip<wr::V7>>(root.clips.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for (clip, sequence) in clips.iter_mut().zip(source.sequences()) {
        clip.bounds = extent(sequence.extent);
        set_metadata(
            &mut raw,
            &mut clip.native,
            "mdlx.move_speed",
            i64::from(sequence.move_speed.to_bits()),
            "",
        )?;
        set_metadata(
            &mut raw,
            &mut clip.native,
            "mdlx.rarity",
            i64::from(sequence.rarity.to_bits()),
            "",
        )?;
        set_metadata(
            &mut raw,
            &mut clip.native,
            "mdlx.sync_point",
            i64::from(sequence.sync_point),
            "",
        )?;
    }
    if !root.clips.is_empty() {
        raw.replace(raw.section_id(root.clips.index).unwrap(), &clips)?;
    }
    let mut meshes = raw
        .resolve::<wr::Mesh<wr::V2>>(model.meshes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for (index, (mesh, source_mesh)) in meshes.iter_mut().zip(source.geosets()).enumerate() {
        let mut sections = raw
            .resolve::<wr::MeshSection>(mesh.sections.cast())?
            .iter()
            .collect::<Result<Vec<_>, _>>()?;
        let section = &mut sections[0];
        set_metadata(
            &mut raw,
            &mut section.native,
            "mdlx.selection_raw",
            i64::from(source_mesh.raw_unselectable()),
            "",
        )?;
        set_metadata(
            &mut raw,
            &mut section.native,
            "mdlx.selection_group",
            i64::from(source_mesh.selection_group),
            "",
        )?;
        set_metadata(
            &mut raw,
            &mut section.native,
            "mdlx.sequence_extents",
            0,
            &source_mesh
                .sequence_extents
                .iter()
                .map(extent_text)
                .collect::<Vec<_>>()
                .join(";"),
        )?;
        for animation in source
            .geoset_animations()
            .filter(|animation| animation.geoset_id == index as u32)
        {
            set_metadata(
                &mut raw,
                &mut section.native,
                "mdlx.animation.flags",
                i64::from(animation.flags.bits()),
                "",
            )?;
            set_metadata(
                &mut raw,
                &mut section.native,
                "mdlx.animation.alpha",
                i64::from(base(&animation.alpha).to_bits()),
                "",
            )?;
            set_metadata(
                &mut raw,
                &mut section.native,
                "mdlx.animation.color",
                0,
                &values_text(base(&animation.color)),
            )?;
        }
        raw.replace(raw.section_id(mesh.sections.index).unwrap(), &sections)?;
    }
    let mut nodes = raw
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let identities: BTreeMap<_, _> = source
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, node)| (node.object_id, index))
        .collect();
    for bone in source.bones() {
        let node = &mut nodes[identities[&bone.node.object_id]];
        if let wr::NodePayload::Bone(data) = &mut node.payload {
            data.gate_mesh = bone.geoset_id;
        }
        set_metadata(
            &mut raw,
            &mut node.native,
            "mdlx.geoset_animation_id",
            i64::from(bone.geoset_animation_id),
            "",
        )?;
    }
    for attachment in source.attachments() {
        set_metadata(
            &mut raw,
            &mut nodes[identities[&attachment.node.object_id]].native,
            "mdlx.attachment_id",
            i64::from(attachment.id),
            "",
        )?;
    }
    for particle in source.particle_emitters2() {
        set_metadata(
            &mut raw,
            &mut nodes[identities[&particle.node.object_id]].native,
            "mdlx.squirt",
            i64::from(particle.squirt),
            "",
        )?;
    }
    for light in source.lights() {
        let node = &mut nodes[identities[&light.node.object_id]];
        set_metadata(
            &mut raw,
            &mut node.native,
            "mdlx.ambient_color",
            0,
            &values_text(base(&light.ambient_color)),
        )?;
        set_metadata(
            &mut raw,
            &mut node.native,
            "mdlx.ambient_intensity",
            i64::from(base(&light.ambient_intensity).to_bits()),
            "",
        )?;
        set_metadata(
            &mut raw,
            &mut node.native,
            "mdlx.shadow_intensity",
            i64::from(light.try_shadow_intensity().unwrap_or(0.).to_bits()),
            "",
        )?;
        if let wr::NodePayload::Light(data) = &mut node.payload {
            data.shadow_casting = wem::Bool::from(light.try_shadow_casting().unwrap_or(false));
            if let Ok(range) = light.try_shadow_casting_range() {
                data.shadow_casting_start = base(&range.start);
                data.shadow_casting_end = base(&range.end);
            }
            let falloff = light.falloff();
            data.quadratic_falloff = base(&falloff.quadratic);
            data.linear_falloff = base(&falloff.linear);
            data.damping = base(&falloff.damping);
        }
    }
    for (index, camera) in source.cameras().enumerate() {
        let node = &mut nodes[identities.len() + index];
        let prefix = match camera.variant {
            native::CameraVariant::Variant1(prefix) | native::CameraVariant::Variant2(prefix) => {
                prefix.iter().map(|byte| format!("{byte:02x}")).collect()
            }
            _ => String::new(),
        };
        set_metadata(
            &mut raw,
            &mut node.native,
            "mdlx.camera.variant",
            i64::from(camera.variant.value()),
            &prefix,
        )?;
    }
    if let Ok(poses) = source.try_bind_poses() {
        for (index, pose) in poses.enumerate() {
            if let Some(node) = nodes.get_mut(index) {
                set_metadata(
                    &mut raw,
                    &mut node.native,
                    "mdlx.bind_pose",
                    0,
                    &values_text(pose.0),
                )?;
            } else {
                set_metadata(
                    &mut raw,
                    &mut profiles[0].native,
                    &format!("mdlx.bind_pose.{index}"),
                    0,
                    &values_text(pose.0),
                )?;
            }
        }
    }
    if !nodes.is_empty() {
        raw.replace(raw.section_id(model.nodes.nodes.index).unwrap(), &nodes)?;
    }
    raw.replace(raw.section_id(model.profile_sets.index).unwrap(), &profiles)?;
    Ok(wem::Document::from_raw(raw)?)
}

pub(super) fn overlay_metadata<D: ModelDialect>(
    model: &mut native::Model<'static, D>,
    asset: &Asset,
    options: &ExportOptions,
    active_nodes: &[wr::Node<wr::V19>],
    projections: &[s::MeshProjection],
    ids: &[u32],
    context: &mut Context,
) -> Result<(), Error> {
    let raw = asset.document.raw();
    let root = asset.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let graph = graph(raw, root.models, options.model as usize)?;
    let profile = raw
        .resolve::<wr::ProfileMaterialSet>(graph.profile_sets.cast())?
        .iter()
        .find_map(|record| match record {
            Ok(record) if record.profile == options.profile.unwrap_or(root.default_profile) => {
                Some(Ok(record))
            }
            Err(error) => Some(Err(error)),
            _ => None,
        })
        .transpose()?;
    let profile_meta = profile
        .map(|profile| metadata(raw, profile.native))
        .transpose()?
        .unwrap_or_default();
    if let Some(mut info) = model.model_info() {
        if let Some((_, value)) = profile_meta.get("mdlx.animation_file_name") {
            info.animation_file_name
                .set_text(value)
                .map_err(|error| failure("model.animation_file_name", error))?;
        }
        if let Some((value, _)) = profile_meta.get("mdlx.blend_time") {
            info.blend_time =
                u32::try_from(*value).map_err(|error| failure("model.blend_time", error))?;
        }
        model.set_model_info(&info);
    }
    if let Some((_, value)) = profile_meta.get("mdlx.gliders") {
        let values = if value.is_empty() {
            Vec::new()
        } else {
            value
                .split(',')
                .map(|value| {
                    value
                        .parse::<u32>()
                        .map(|geoset_id| native::Glider { geoset_id })
                        .map_err(|error| failure("model.gliders", error))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        let values = if projections.is_empty() {
            values
        } else {
            values
                .into_iter()
                .flat_map(|glider| {
                    projections
                        .iter()
                        .enumerate()
                        .filter_map(move |(index, projection)| {
                            (projection.mesh == glider.geoset_id).then_some(native::Glider {
                                geoset_id: index as u32,
                            })
                        })
                })
                .collect()
        };
        model.set_gliders(values);
    }
    let source_clips = raw
        .resolve::<wr::Clip<wr::V7>>(root.clips.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for (sequence, clip) in model.sequences_mut().zip(source_clips) {
        let values = metadata(raw, clip.native)?;
        sequence.extent = native_extent(clip.bounds);
        if let Some((value, _)) = values.get("mdlx.move_speed") {
            sequence.move_speed = f32::from_bits(*value as u32);
        }
        if let Some((value, _)) = values.get("mdlx.rarity") {
            sequence.rarity = f32::from_bits(*value as u32);
        }
        if let Some((value, _)) = values.get("mdlx.sync_point") {
            sequence.sync_point =
                u32::try_from(*value).map_err(|error| failure("sequence.sync_point", error))?;
        }
    }
    let meshes = raw
        .resolve::<wr::Mesh<wr::V2>>(graph.meshes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut animations = Vec::new();
    for (index, geoset) in model.geosets_mut().enumerate() {
        let mapping = projections.get(index);
        let mesh_index = mapping.map_or(index, |map| map.mesh as usize);
        let Some(mesh) = meshes.get(mesh_index) else {
            continue;
        };
        let Some(section) = raw
            .resolve::<wr::MeshSection>(mesh.sections.cast())?
            .get(mapping.map_or(0, |map| map.section as usize))
            .transpose()?
        else {
            continue;
        };
        let values = metadata(raw, section.native)?;
        if let Some((value, _)) = values.get("mdlx.selection_raw") {
            geoset.set_raw_unselectable(*value as u32);
        }
        if let Some((value, _)) = values.get("mdlx.selection_group") {
            let original =
                u32::try_from(*value).map_err(|error| failure("mesh.selection_group", error))?;
            if geoset.selection_group == original & 0xffff {
                geoset.selection_group = original;
            }
        }
        if let Some((_, value)) = values.get("mdlx.sequence_extents") {
            let extents = if value.is_empty() {
                Vec::new()
            } else {
                value
                    .split(';')
                    .map(extent_from_text)
                    .collect::<Result<Vec<_>, _>>()?
            };
            geoset.sequence_extents = extents.into_iter().collect();
        }
        if let Some((flags, _)) = values.get("mdlx.animation.flags") {
            let mut animation = native::GeosetAnimation::new(index as u32);
            animation.flags = native::GeosetAnimationFlags(*flags as u32);
            if let Some((value, _)) = values.get("mdlx.animation.alpha") {
                animation.alpha.set_value(f32::from_bits(*value as u32));
            }
            if let Some((_, value)) = values.get("mdlx.animation.color") {
                let components = parse_values(value)?;
                animation
                    .color
                    .set_value(components.as_slice().try_into().map_err(|_| {
                        Error::new("geoset_animation.color", "color needs three components")
                    })?);
            }
            animations.push(animation);
        }
    }
    model.set_geoset_animations(animations);
    let mut bind_poses = Vec::new();
    for (index, node) in active_nodes.iter().enumerate() {
        let values = metadata(raw, node.native.clone())?;
        let id = ids[index];
        if let Some((_, value)) = values.get("mdlx.bind_pose") {
            let components = parse_values(value)?;
            let pose = native::BindPoseMatrix(
                components
                    .as_slice()
                    .try_into()
                    .map_err(|_| Error::new("bind_pose", "matrix needs twelve components"))?,
            );
            while bind_poses.len() < index {
                bind_poses.push(native::BindPoseMatrix([0.; 12]));
            }
            bind_poses.push(pose);
        }
        if let Some(bone) = model.bones_mut().find(|bone| bone.node.object_id == id) {
            if let wr::NodePayload::Bone(data) = &node.payload {
                bone.geoset_id = data.gate_mesh;
                if data.gate_mesh != u32::MAX && !projections.is_empty() {
                    let targets = projections
                        .iter()
                        .enumerate()
                        .filter(|(_, map)| map.mesh == data.gate_mesh)
                        .map(|(index, _)| index as u32)
                        .collect::<Vec<_>>();
                    if targets.len() == 1 {
                        bone.geoset_id = targets[0];
                    } else {
                        context.unsupported(
                            "bone.gate_mesh_projection",
                            "one native bone gate cannot select multiple projected geosets",
                        )?;
                        bone.geoset_id = u32::MAX;
                    }
                }
            }
            if let Some((value, _)) = values.get("mdlx.geoset_animation_id") {
                bone.geoset_animation_id = *value as u32;
                if *value as u32 != u32::MAX
                    && projections
                        .iter()
                        .enumerate()
                        .any(|(index, map)| map.mesh as usize != index || map.section != 0)
                {
                    context.unsupported("bone.geoset_animation_projection", "projected draw sections require explicit geoset-animation identity remapping")?;
                    bone.geoset_animation_id = u32::MAX;
                }
            }
        }
        if let Some(attachment) = model
            .attachments_mut()
            .find(|record| record.node.object_id == id)
        {
            if let Some((value, _)) = values.get("mdlx.attachment_id") {
                attachment.id = *value as u32;
            }
        }
        if let Some(particle) = model
            .particle_emitters2_mut()
            .find(|record| record.node.object_id == id)
        {
            if let Some((value, _)) = values.get("mdlx.squirt") {
                if (*value != 0) == particle.squirt_enabled() {
                    particle.squirt = *value as u32;
                }
            }
        }
        if let Some(light) = model
            .lights_mut()
            .find(|record| record.node.object_id == id)
        {
            if let Some((_, value)) = values.get("mdlx.ambient_color") {
                let components = parse_values(value)?;
                light
                    .ambient_color
                    .set_value(components.as_slice().try_into().map_err(|_| {
                        Error::new("light.ambient_color", "color needs three components")
                    })?);
            }
            if let Some((value, _)) = values.get("mdlx.ambient_intensity") {
                light
                    .ambient_intensity
                    .set_value(f32::from_bits(*value as u32));
            }
            if let Some((value, _)) = values.get("mdlx.shadow_intensity") {
                let value = f32::from_bits(*value as u32);
                if let Err(error) = light.try_set_shadow_intensity(value) {
                    if value != 0. {
                        context.unsupported("light.shadow_intensity", error.to_string())?;
                    }
                }
            }
        }
    }
    if !bind_poses.is_empty() {
        if let Err(error) = model.try_set_bind_poses(bind_poses) {
            context.unsupported("bind_pose", error.to_string())?;
        }
    }
    Ok(())
}
