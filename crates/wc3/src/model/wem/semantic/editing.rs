//! Preservation of active native records while replacing mapped authoring values.
use super::*;

pub(super) struct Original {
    pub root: r::DocumentRecord<r::V1>,
    pub model: r::ModelRecord<r::V9>,
    pub model_rows: Vec<r::ModelRecord<r::V9>>,
    pub nodes: Vec<r::Node<r::V19>>,
    pub materials: Vec<r::MaterialRecord>,
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
        native.name = replacement.name;
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
        native.native = metadata(doc, &entries)?;
        replacement = native;
    }
    Ok(replacement)
}
pub(super) fn material(
    doc: &RawDocument<'_>,
    mut replacement: r::MaterialRecord,
    old: Option<&r::MaterialRecord>,
) -> r::MaterialRecord {
    if let Some(old) = old {
        let changed = old.common.blend != replacement.common.blend
            || old.common.flags != replacement.common.flags
            || common_texture(doc, &old.common.body)
                != common_texture(doc, &replacement.common.body);
        let mut native = old.clone();
        native.name = replacement.name;
        if changed {
            native.common.blend = replacement.common.blend;
            native.common.flags = replacement.common.flags;
            native.common.body = replacement.common.body;
            native.sync = r::NativeSync::COMMON_EDITED;
        }
        replacement = native;
    }
    replacement
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
    profile.materials = new_profile.materials;
    profiles[original.profile_index] = profile;
    let mut saved = original.model;
    saved.name = new_model.name;
    saved.bounds = new_model.bounds;
    saved.meshes = new_model.meshes;
    saved.nodes.nodes = new_model.nodes.nodes;
    saved.anim_channels = new_model.anim_channels;
    saved.profile_sets = put(doc, &profiles)?.cast();
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
            clip.name = updated.name;
            clip.duration = updated.duration;
            clip.flags = updated.flags;
            clip.looping = updated.looping;
            clip.containers = updated.containers;
        }
    }
    let mut root = original.root;
    root.models = put(doc, &model_rows)?.cast();
    root.textures = new_textures;
    root.clips = put(doc, &clips)?.cast();
    let root_ref = put(doc, &[root])?;
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
    current.native = metadata(doc, &entries)?;
    Ok(current)
}
