//! Preservation of active native records while replacing mapped authoring values.
use super::*;
use crate::model::wem::{ReferenceFlags, Target};

pub(super) struct Original {
    pub root: r::DocumentRecord<r::V1>,
    pub model: r::ModelRecord<r::V9>,
    pub model_rows: Vec<r::ModelRecord<r::V9>>,
    pub nodes: Vec<r::Node<r::V19>>,
    pub materials: Vec<r::MaterialRecord>,
    pub textures: Vec<r::TextureRecord>,
    pub profiles: Vec<r::ProfileMaterialSet>,
    pub profile_index: usize,
    pub clips: Vec<r::Clip<r::V7>>,
    pub channels: Vec<r::AnimChannel>,
    pub model_index: usize,
}
pub(super) fn prepare(
    model: &Model<'_>,
    options: Options,
) -> Result<(RawDocument<'static>, Option<Original>), WriteError> {
    let Some(native) = &model.native else {
        return Ok((RawDocument::new_with_options(options), None));
    };
    let doc = native.document.raw();
    let root = native
        .document
        .root::<r::DocumentRecord<r::V1>>()
        .map_err(|_| WriteError::invalid("native root"))?;
    if model.space != root.space || model.unit_scale != root.unit_scale {
        return Err(WriteError::invalid(
            "coordinate metadata is shared by all models; edit the whole document explicitly",
        ));
    }
    let model_rows = rows::<r::ModelRecord<r::V9>>(doc, root.models.cast())
        .map_err(|_| WriteError::invalid("editing requires current model revision"))?;
    let original = model_rows
        .get(native.model as usize)
        .ok_or_else(|| WriteError::invalid("native model selection"))?
        .clone();
    let nodes = rows::<r::Node<r::V19>>(doc, original.nodes.nodes.cast())
        .map_err(|_| WriteError::invalid("editing requires current node revision"))?;
    let profiles = rows::<r::ProfileMaterialSet>(doc, original.profile_sets.cast())
        .map_err(|_| WriteError::invalid("native material profiles"))?;
    let profile_index = profiles
        .iter()
        .position(|p| p.profile == model.profile)
        .ok_or_else(|| WriteError::invalid("native profile selection"))?;
    let materials = rows::<r::MaterialRecord>(doc, profiles[profile_index].materials.cast())
        .map_err(|_| WriteError::invalid("native material table"))?;
    let textures = rows::<r::TextureRecord>(doc, root.textures.cast())
        .map_err(|_| WriteError::invalid("native textures"))?;
    let clips = rows::<r::Clip<r::V7>>(doc, root.clips.cast())
        .map_err(|_| WriteError::invalid("editing requires current clip revision"))?;
    if (native.mesh_projections.is_empty()
        && original.meshes.entries as usize != model.meshes.len())
        || (!native.mesh_projections.is_empty()
            && native.mesh_projections.len() != model.meshes.len())
        || nodes.len() != model.nodes.len()
        || materials.len() != model.materials.len()
        || root.textures.entries as usize != model.textures.len()
        || clips.iter().filter(|c| c.model == native.model).count() != model.clips.len()
    {
        return Err(WriteError::invalid(
            "native-dependent table cardinality changed; edit indexed records or explicitly construct a new generic model",
        ));
    }
    let channels = rows::<r::AnimChannel>(doc, original.anim_channels.channels.cast())
        .map_err(|_| WriteError::invalid("native animation channels"))?;
    let mut retained = doc.clone();
    retained.prune_unreachable()?;
    Ok((
        retained.into_owned(),
        Some(Original {
            root,
            model: original,
            model_rows,
            nodes,
            materials,
            textures,
            profiles,
            profile_index,
            clips,
            channels,
            model_index: native.model as usize,
        }),
    ))
}
pub(super) fn node(
    doc: &mut RawDocument<'static>,
    mut replacement: r::Node<r::V19>,
    old: Option<&r::Node<r::V19>>,
) -> Result<r::Node<r::V19>, WriteError> {
    if let Some(old) = old {
        let mut native = old.clone();
        native.name = retain_flags(replacement.name, old.name);
        native.parent = replacement.parent;
        native.flags = replacement.flags;
        native.pivot = replacement.pivot;
        native.local = replacement.local;
        let mut values = metadata_read(doc, old.native.clone())
            .map_err(|_| WriteError::invalid("node metadata"))?;
        let updated = metadata_read(doc, replacement.native)
            .map_err(|_| WriteError::invalid("updated node metadata"))?;
        values.remove("conversion.inverse_bind");
        values.extend(updated);
        let entries: Vec<_> = values
            .iter()
            .map(|(name, (value, text))| (name.as_str(), *value, text.clone()))
            .collect();
        native.native = metadata_with_flags(doc, &entries, &old.native)?;
        replacement = native;
    }
    Ok(replacement)
}
pub(super) fn material(
    doc: &mut RawDocument<'static>,
    mut replacement: r::MaterialRecord,
    old: Option<&r::MaterialRecord>,
) -> Result<r::MaterialRecord, WriteError> {
    if let Some(old) = old {
        let changed = old.common.blend != replacement.common.blend
            || old.common.flags != replacement.common.flags
            || common_texture(doc, &old.common.body)
                != common_texture(doc, &replacement.common.body);
        let mut native = old.clone();
        native.name = retain_flags(replacement.name, old.name);
        if changed {
            native.common.blend = replacement.common.blend;
            native.common.flags = replacement.common.flags;
            let mut body = replacement.common.body;
            if let (r::MaterialBody::Composite(new), r::MaterialBody::Composite(old)) =
                (&mut body, &old.common.body)
            {
                let layer_flags = match doc
                    .referenced_section(*old)
                    .map_err(|_| WriteError::invalid("source composite body"))?
                    .map(|section| section.revision())
                {
                    Some(1) => rows::<r::CompositeBody<r::V1>>(doc, old.cast())
                        .map_err(|_| WriteError::invalid("source composite body"))?
                        .first()
                        .map(|body| body.layers.flags),
                    Some(2) => rows::<r::CompositeBody<r::V2>>(doc, old.cast())
                        .map_err(|_| WriteError::invalid("source composite body"))?
                        .first()
                        .map(|body| body.layers.flags),
                    None => None,
                    _ => return Err(WriteError::invalid("source composite revision")),
                };
                if let Some(flags) = layer_flags {
                    let mut bodies = rows::<r::CompositeBody<r::V2>>(doc, new.cast())
                        .map_err(|_| WriteError::invalid("updated composite body"))?;
                    for body in &mut bodies {
                        body.layers.flags = flags;
                    }
                    *new = put(doc, &bodies)?.cast();
                }
                *new = retain_flags(*new, *old);
            }
            native.common.body = body;
            native.sync = r::NativeSync::CommonEdited;
        }
        replacement = native;
    }
    Ok(replacement)
}
pub(super) fn finish(
    doc: &mut RawDocument<'static>,
    mut new_model: r::ModelRecord<r::V9>,
    new_profile: r::ProfileMaterialSet,
    new_textures: Ref<r::targets::TextureRef>,
    new_clips: Vec<r::Clip<r::V7>>,
    original: Original,
) -> Result<(), WriteError> {
    let mut profiles = original.profiles;
    let old_profile = &profiles[original.profile_index];
    let mut profile = old_profile.clone();
    profile.materials = retain_flags(new_profile.materials, old_profile.materials);
    profiles[original.profile_index] = profile;
    let mut saved = original.model;
    saved.name = retain_flags(new_model.name, saved.name);
    saved.bounds = new_model.bounds;
    saved.meshes = retain_flags(new_model.meshes, saved.meshes);
    saved.nodes.nodes = retain_flags(new_model.nodes.nodes, saved.nodes.nodes);
    saved.anim_channels.channels = retain_flags(
        new_model.anim_channels.channels,
        saved.anim_channels.channels,
    );
    saved.profile_sets = retain_flags(put(doc, &profiles)?.cast(), saved.profile_sets);
    new_model = saved;
    let mut model_rows = original.model_rows;
    model_rows[original.model_index] = new_model;
    let mut fresh = new_clips.into_iter();
    let mut clips = original.clips;
    for clip in &mut clips {
        if clip.model as usize == original.model_index {
            let updated = fresh
                .next()
                .ok_or_else(|| WriteError::invalid("clip cardinality"))?;
            clip.name = retain_flags(updated.name, clip.name);
            clip.duration = updated.duration;
            clip.flags = updated.flags;
            if clip.looping.get() != updated.looping.get() {
                clip.looping = updated.looping;
            }
            clip.containers = retain_flags(updated.containers, clip.containers);
        }
    }
    let mut root = original.root;
    root.models = retain_flags(put(doc, &model_rows)?.cast(), root.models);
    root.textures = retain_flags(new_textures, root.textures);
    root.clips = retain_flags(put(doc, &clips)?.cast(), root.clips);
    let mut root_ref = put(doc, &[root])?;
    root_ref.flags = doc.root_reference().flags;
    doc.set_root(root_ref);
    Ok(())
}

fn common_texture(doc: &RawDocument<'_>, body: &r::MaterialBody) -> Option<u32> {
    let r::MaterialBody::Composite(reference) = body else {
        return None;
    };
    let revision = doc
        .referenced_section(*reference)
        .ok()
        .flatten()?
        .revision();
    let layers = match revision {
        1 => {
            rows::<r::CompositeBody<r::V1>>(doc, reference.cast())
                .ok()?
                .first()?
                .layers
        }
        2 => {
            rows::<r::CompositeBody<r::V2>>(doc, reference.cast())
                .ok()?
                .first()?
                .layers
        }
        _ => return None,
    };
    rows::<r::CompositeLayer>(doc, layers.cast())
        .ok()?
        .first()
        .map(|l| l.input.texture)
}

/// Keeps logical slot identity while a semantic view exposes physical materials.
pub(super) fn material_slot(
    model: &Model<'_>,
    mesh: usize,
    material: u32,
) -> Result<u32, WriteError> {
    let Some(native) = &model.native else {
        return Ok(material);
    };
    if let Some(&slot) = native.mesh_slots.get(mesh) {
        if native.slot_materials.get(slot as usize) == Some(&material) {
            return Ok(slot);
        }
    }
    if material == u32::MAX {
        return Ok(u32::MAX);
    }
    native
        .slot_materials
        .iter()
        .position(|&m| m == material)
        .map(|slot| slot as u32)
        .ok_or_else(|| WriteError::invalid("material has no binding in selected look"))
}

pub(super) fn container(
    doc: &mut RawDocument<'static>,
    mut current: r::SubTrackContainer,
    layer: &TrackLayer,
    original: Option<&Original>,
    clip: usize,
) -> Result<r::SubTrackContainer, WriteError> {
    let Some((original, index)) = original.zip(layer.source) else {
        return Ok(current);
    };
    let Some(clip) = original
        .clips
        .iter()
        .filter(|c| c.model as usize == original.model_index)
        .nth(clip)
    else {
        return Err(WriteError::invalid("native clip layer"));
    };
    let containers = rows::<r::SubTrackContainer>(doc, clip.containers.cast())
        .map_err(|_| WriteError::invalid("native controllers"))?;
    let old = containers
        .get(index as usize)
        .ok_or_else(|| WriteError::invalid("native controller identity"))?;
    let mut values = metadata_read(doc, old.native.clone())
        .map_err(|_| WriteError::invalid("native controller metadata"))?;
    values.remove("conversion.global_duration");
    values.extend(
        metadata_read(doc, current.native)
            .map_err(|_| WriteError::invalid("updated controller metadata"))?,
    );
    let entries: Vec<_> = values
        .iter()
        .map(|(name, (value, text))| (name.as_str(), *value, text.clone()))
        .collect();
    current.name = retain_flags(current.name, old.name);
    let old_tracks = match doc
        .referenced_section(old.sub_tracks)
        .map_err(|_| WriteError::invalid("native tracks"))?
        .map(|section| section.revision())
    {
        Some(1) => rows::<r::SubTrack<r::V1>>(doc, old.sub_tracks.cast())
            .map_err(|_| WriteError::invalid("native tracks"))?
            .into_iter()
            .map(|old| r::SubTrack::<r::V2> {
                channel: old.channel,
                interp: old.interp,
                times: old.times,
                values: old.values,
                tcb: Ref::default(),
            })
            .collect(),
        Some(2) => rows::<r::SubTrack<r::V2>>(doc, old.sub_tracks.cast())
            .map_err(|_| WriteError::invalid("native tracks"))?,
        None => Vec::new(),
        _ => return Err(WriteError::invalid("native track revision")),
    };
    let mut tracks = rows::<r::SubTrack<r::V2>>(doc, current.sub_tracks.cast())
        .map_err(|_| WriteError::invalid("updated track revision"))?;
    for track in &mut tracks {
        if let Some(old) = old_tracks
            .iter()
            .find(|old| old.channel == track.channel)
            .or_else(|| (old_tracks.len() == 1).then(|| &old_tracks[0]))
        {
            track.times = retain_flags(track.times, old.times);
            track.values = retain_flags(track.values, old.values);
            track.tcb = retain_flags(track.tcb, old.tcb);
        }
    }
    current.sub_tracks = retain_flags(put(doc, &tracks)?.cast(), old.sub_tracks);
    current.native = metadata_with_flags(doc, &entries, &old.native)?;
    Ok(current)
}

/// Retains source metadata while changing a reference's target/count.
pub(super) fn retain_flags<T: Target, U: Target>(mut replacement: Ref<T>, old: Ref<U>) -> Ref<T> {
    replacement.flags = old.flags;
    replacement
}
fn metadata_with_flags(
    doc: &mut RawDocument<'static>,
    values: &[(&str, i64, String)],
    old: &r::NativeBag,
) -> Result<r::NativeBag, WriteError> {
    let mut flags = BTreeMap::new();
    if let Some(section) = doc
        .referenced_section(old.entries)
        .map_err(|_| WriteError::invalid("native metadata"))?
    {
        match section.revision() {
            1 => {
                for entry in rows::<r::NativeEntry<r::V1>>(doc, old.entries.cast())
                    .map_err(|_| WriteError::invalid("native metadata"))?
                {
                    let name = doc
                        .string(entry.name)
                        .map_err(|_| WriteError::invalid("metadata name"))?
                        .unwrap_or_default()
                        .to_owned();
                    flags.insert(name, (entry.name.flags, ReferenceFlags::default()));
                }
            }
            2 => {
                for entry in rows::<r::NativeEntry<r::V2>>(doc, old.entries.cast())
                    .map_err(|_| WriteError::invalid("native metadata"))?
                {
                    let name = doc
                        .string(entry.name)
                        .map_err(|_| WriteError::invalid("metadata name"))?
                        .unwrap_or_default()
                        .to_owned();
                    flags.insert(name, (entry.name.flags, entry.text.flags));
                }
            }
            _ => return Err(WriteError::invalid("native metadata revision")),
        }
    }
    let mut result = metadata(doc, values)?;
    let mut entries = rows::<r::NativeEntry<r::V2>>(doc, result.entries.cast())
        .map_err(|_| WriteError::invalid("updated metadata"))?;
    for entry in &mut entries {
        if let Some((name_flags, text_flags)) = flags.get(
            doc.string(entry.name)
                .map_err(|_| WriteError::invalid("metadata name"))?
                .unwrap_or_default(),
        ) {
            entry.name.flags = *name_flags;
            entry.text.flags = *text_flags;
        }
    }
    result.entries = retain_flags(put(doc, &entries)?.cast(), old.entries);
    Ok(result)
}

pub(super) fn texture(
    mut replacement: r::TextureRecord,
    old: Option<&r::TextureRecord>,
) -> r::TextureRecord {
    if let Some(old) = old {
        replacement.path = retain_flags(replacement.path, old.path);
        if let (r::TextureKey::Path(new), r::TextureKey::Path(old)) =
            (&mut replacement.key, &old.key)
        {
            new.value = retain_flags(new.value, old.value);
        }
        replacement.slot_type = old.slot_type;
        replacement.declared_space = old.declared_space;
    }
    replacement
}
