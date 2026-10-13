//! Typed native metadata, string resources, and range-checked scalar helpers.
use super::*;

pub(super) fn failure(error: impl ToString) -> Error {
    Error::new("m2", error.to_string())
}

pub(super) fn extent(value: native::records::Extent) -> wr::Extent {
    wr::Extent {
        minimum: value.minimum,
        maximum: value.maximum,
        sphere_radius: value.radius,
    }
}

pub(super) fn native_extent(value: wr::Extent) -> native::records::Extent {
    native::records::Extent {
        minimum: value.minimum,
        maximum: value.maximum,
        radius: value.sphere_radius,
    }
}

pub(super) fn text(bytes: impl Iterator<Item = u8>, path: &str) -> Result<String, Error> {
    let mut bytes: Vec<_> = bytes.collect();
    if bytes.last() == Some(&0) {
        bytes.pop();
    }
    String::from_utf8(bytes).map_err(|_| {
        Error::new(
            path,
            "filename is not UTF-8; supply an explicit resource mapping",
        )
    })
}

pub(super) fn put<T: wem::Record + wem::Write>(
    doc: &mut wem::RawDocument<'static>,
    values: &[T],
) -> Result<wem::Ref<T>, Error> {
    if values.is_empty() {
        return Ok(wem::Ref::default());
    }
    let id = doc.append(values)?;
    Ok(wem::Ref::new(values.len() as u32, id.index(), 0))
}

pub(super) fn bag(
    doc: &mut wem::RawDocument<'static>,
    entries: &[(&str, i64)],
) -> Result<wr::NativeBag, Error> {
    let mut values = Vec::new();
    for (name, value) in entries {
        let chars: Vec<_> = name.bytes().map(|value| wr::Character { value }).collect();
        values.push(wr::NativeEntry::<wr::V2> {
            name: put(doc, &chars)?.cast(),
            value: *value,
            ..Default::default()
        });
    }
    Ok(wr::NativeBag {
        entries: put(doc, &values)?.cast(),
    })
}

pub(super) fn metadata(
    doc: &wem::RawDocument<'_>,
    bag: &wr::NativeBag,
) -> Result<BTreeMap<String, i64>, Error> {
    let mut entries = BTreeMap::new();
    for row in doc
        .resolve::<wr::NativeEntry<wr::V2>>(bag.entries.cast())?
        .iter()
    {
        let row = row?;
        entries.insert(
            doc.string(row.name)?.unwrap_or_default().to_owned(),
            row.value,
        );
    }
    Ok(entries)
}

pub(super) fn byte(value: &[f32], path: &str) -> Result<u8, Error> {
    if value.len() != 1
        || !value[0].is_finite()
        || value[0] < 0.
        || value[0] > 255.
        || value[0].fract() != 0.
    {
        Err(Error::new(
            path,
            "toggle value cannot be represented as a native byte",
        ))
    } else {
        Ok(value[0] as u8)
    }
}

pub(super) fn add_texts(
    doc: &mut wem::RawDocument<'static>,
    native: &mut wr::NativeBag,
    texts: &[(&str, String)],
) -> Result<(), Error> {
    let mut entries: Vec<_> = doc
        .resolve::<wr::NativeEntry<wr::V2>>(native.entries.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    for (name, text) in texts {
        let name = put(
            doc,
            &name
                .bytes()
                .map(|value| wr::Character { value })
                .collect::<Vec<_>>(),
        )?
        .cast();
        let text = put(
            doc,
            &text
                .bytes()
                .map(|value| wr::Character { value })
                .collect::<Vec<_>>(),
        )?
        .cast();
        entries.push(wr::NativeEntry::<wr::V2> {
            name,
            value: 0,
            text,
        });
    }
    native.entries = put(doc, &entries)?.cast();
    Ok(())
}

pub(super) fn metadata_text(
    doc: &wem::RawDocument<'_>,
    bag: &wr::NativeBag,
    name: &str,
) -> Result<Option<String>, Error> {
    for row in doc
        .resolve::<wr::NativeEntry<wr::V2>>(bag.entries.cast())?
        .iter()
    {
        let row = row?;
        if doc.string(row.name)? == Some(name) {
            return Ok(doc.string(row.text)?.map(str::to_owned));
        }
    }
    Ok(None)
}

pub(super) fn index_words(value: Option<String>) -> Result<ZeroVec<'static, u16>, Error> {
    let Some(value) = value else {
        return Ok(ZeroVec::new());
    };
    if value.is_empty() {
        return Ok(ZeroVec::new());
    }
    let words = value
        .split(',')
        .map(|v| {
            v.parse::<u16>()
                .map_err(|_| Error::new("ribbons.indices", "native ribbon index list is invalid"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ZeroVec::alloc_from_slice(&words))
}

pub(super) fn profile_metadata(model: &sm::Model<'_>) -> Result<BTreeMap<String, i64>, Error> {
    let Some(native) = &model.native else {
        return Ok(BTreeMap::new());
    };
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model_record = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    for profile in raw
        .resolve::<wr::ProfileMaterialSet>(model_record.profile_sets.cast())?
        .iter()
    {
        let profile = profile?;
        if profile.profile == wr::ProfileId::WOW {
            return metadata(raw, &profile.native);
        }
    }
    Ok(BTreeMap::new())
}
pub(super) fn patch_native_metadata<D: Dialect>(
    document: wem::Document<'static>,
    source: &Bundle<'_, D>,
) -> Result<wem::Document<'static>, Error>
where
    <D::Layout as native::records::BoneLayout>::NameCrc: NameCrcField,
    for<'data> <D::Layout as native::HeaderLayout>::Flipbooks<'data>:
        FlipbookField<'data, D::Layout>,
{
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model = document
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut profile = document
        .raw()
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.profile", "missing profile"))??;
    let mut nodes: Vec<_> = document
        .raw()
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let (_, bones) = source.bones().map_err(failure)?;
    let mut raw = document.into_raw();
    for (node, bone) in nodes.iter_mut().zip(bones) {
        node.native = bag(
            &mut raw,
            &[
                ("m2.bone.key", i64::from(bone.key_bone_id)),
                ("m2.bone.flags", i64::from(bone.flags)),
                ("m2.bone.mesh", i64::from(bone.submesh_id)),
                ("m2.bone.name_crc", i64::from(bone.name_crc.get())),
            ],
        )?;
    }
    if !nodes.is_empty() {
        let id = raw
            .section_id(model.nodes.nodes.index)
            .ok_or_else(|| Error::new("wem.nodes", "missing nodes"))?;
        raw.replace(id, &nodes)?;
    }
    profile.native = bag(
        &mut raw,
        &[
            ("m2.flags", i64::from(source.model.core().flags)),
            ("m2.colors.count", source.model.core().colors.len() as i64),
            (
                "m2.flipbooks.count",
                source.model.core().texture_flipbooks.records().len() as i64,
            ),
            (
                "m2.weights.count",
                source.model.core().texture_weights.len() as i64,
            ),
            (
                "m2.transforms.count",
                source.model.core().texture_transforms.len() as i64,
            ),
            ("m2.source_version", i64::from(D::VERSION)),
        ],
    )?;
    let mut texts = Vec::new();
    let (owner, _) = source.bones().map_err(failure)?;
    let sequences = sequence_tables(source, owner)?;
    let mut key_bones = source.model.core().key_bone_lookup.as_slice();
    if let Some(File::Skeleton(skeleton)) = source.file(owner) {
        let typed = skeleton
            .as_typed::<D::Layout>()
            .ok_or_else(|| Error::new("skeleton.layout", "skeleton layout differs from model"))?;
        for chunk in &typed.chunks {
            if let native::companions::SkeletonChunk::Bones { lookup, .. } = chunk {
                key_bones = lookup.as_slice();
                break;
            }
        }
    }
    for (name, array) in [
        ("m2.sequence_lookup", sequences.lookup),
        ("m2.key_bone_lookup", key_bones),
        (
            "m2.texture_lookup",
            source.model.core().texture_lookup.as_slice(),
        ),
        (
            "m2.attachment_lookup",
            source.model.core().attachment_lookup.as_slice(),
        ),
        (
            "m2.camera_lookup",
            source.model.core().camera_lookup.as_slice(),
        ),
    ] {
        texts.push((
            name,
            array
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ));
    }
    match &source.key {
        native::FileKey::Path(path) => texts.push(("m2.source_path", path.clone())),
        native::FileKey::FileId(id) => {
            let mut entries: Vec<_> = raw
                .resolve::<wr::NativeEntry<wr::V2>>(profile.native.entries.cast())?
                .iter()
                .collect::<Result<_, _>>()?;
            let name = put(
                &mut raw,
                &"m2.source_id"
                    .bytes()
                    .map(|value| wr::Character { value })
                    .collect::<Vec<_>>(),
            )?
            .cast();
            entries.push(wr::NativeEntry::<wr::V2> {
                name,
                value: i64::from(*id),
                ..Default::default()
            });
            profile.native.entries = put(&mut raw, &entries)?.cast();
        }
        _ => {}
    }
    add_texts(&mut raw, &mut profile.native, &texts)?;
    let id = raw
        .section_id(model.profile_sets.index)
        .ok_or_else(|| Error::new("wem.profile", "missing profile table"))?;
    raw.replace(id, &[profile])?;
    Ok(wem::Document::from_raw(raw)?)
}
/// The selected layout's optional bone-name checksum.
pub trait NameCrcField: Sized {
    /// Returns the stored checksum, zero when the field is absent.
    fn get(&self) -> u32;
    /// Constructs the native field from its stored checksum.
    fn make(value: u32, context: &mut Context) -> Result<Self, Error>;
}
impl NameCrcField for () {
    fn get(&self) -> u32 {
        0
    }
    fn make(value: u32, context: &mut Context) -> Result<Self, Error> {
        if value != 0 {
            context.unsupported(
                "bones.name_crc",
                "target classic layout omits bone-name CRCs",
            )?;
        }
        Ok(())
    }
}
impl NameCrcField for u32 {
    fn get(&self) -> u32 {
        *self
    }
    fn make(value: u32, _: &mut Context) -> Result<Self, Error> {
        Ok(value)
    }
}
pub(super) fn restore_native_metadata<V: ModelLayout>(
    model: &sm::Model<'_>,
    core: &mut Core<'static, V>,
) -> Result<native::FileKey, Error> {
    let Some(native) = &model.native else {
        return Ok(native::FileKey::Root);
    };
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    for profile in raw
        .resolve::<wr::ProfileMaterialSet>(record.profile_sets.cast())?
        .iter()
    {
        let profile = profile?;
        if profile.profile != wr::ProfileId::WOW {
            continue;
        }
        let data = metadata(raw, &profile.native)?;
        core.flags = data.get("m2.flags").copied().unwrap_or(0) as u32;
        core.sequence_lookup =
            index_words(metadata_text(raw, &profile.native, "m2.sequence_lookup")?)?;
        core.key_bone_lookup =
            index_words(metadata_text(raw, &profile.native, "m2.key_bone_lookup")?)?;
        core.texture_lookup =
            index_words(metadata_text(raw, &profile.native, "m2.texture_lookup")?)?;
        core.attachment_lookup =
            index_words(metadata_text(raw, &profile.native, "m2.attachment_lookup")?)?;
        core.camera_lookup = index_words(metadata_text(raw, &profile.native, "m2.camera_lookup")?)?;
        if let Some(path) = metadata_text(raw, &profile.native, "m2.source_path")? {
            return Ok(native::FileKey::Path(path));
        }
        if let Some(id) = data.get("m2.source_id") {
            return Ok(native::FileKey::FileId(*id as u32));
        }
        break;
    }
    Ok(native::FileKey::Root)
}

/// The selected sequence-transition duration representation.
pub trait BlendField: Sized {
    /// Returns incoming and outgoing durations in native milliseconds.
    fn parts(&self) -> [u32; 2];
    /// Constructs the selected native transition fields.
    fn make(parts: [u32; 2], context: &mut Context) -> Result<Self, Error>;
}
impl BlendField for u32 {
    fn parts(&self) -> [u32; 2] {
        [*self; 2]
    }
    fn make(parts: [u32; 2], context: &mut Context) -> Result<Self, Error> {
        if parts[0] != parts[1] {
            context.unsupported(
                "clips.blend",
                "target layout has one shared transition duration",
            )?;
        }
        Ok(parts[0])
    }
}
impl BlendField for native::records::SplitBlendTime {
    fn parts(&self) -> [u32; 2] {
        [u32::from(self.incoming), u32::from(self.outgoing)]
    }
    fn make(parts: [u32; 2], _: &mut Context) -> Result<Self, Error> {
        Ok(Self {
            incoming: u16::try_from(parts[0])
                .map_err(|_| Error::new("clips.blend_in", "transition exceeds u16"))?,
            outgoing: u16::try_from(parts[1])
                .map_err(|_| Error::new("clips.blend_out", "transition exceeds u16"))?,
        })
    }
}
pub(super) fn patch_clips<D: Dialect>(
    document: wem::Document<'static>,
    source: &Bundle<'_, D>,
) -> Result<wem::Document<'static>, Error>
where
    <D::Layout as native::records::SequenceLayout>::BlendTime: BlendField,
{
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    if root.clips.entries == 0 {
        return Ok(document);
    }
    let mut clips: Vec<_> = document
        .raw()
        .resolve::<wr::Clip<wr::V7>>(root.clips.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let (owner, _) = source.bones().map_err(failure)?;
    let sequences = sequence_tables(source, owner)?.sequences;
    let mut raw = document.into_raw();
    for (clip, sequence) in clips.iter_mut().zip(sequences) {
        let blend = sequence.blend_time.parts();
        let extra = [
            ("m2.variation", i64::from(sequence.variation_index)),
            ("m2.move_speed", i64::from(sequence.move_speed.to_bits())),
            ("m2.frequency", i64::from(sequence.frequency)),
            ("m2.padding", i64::from(sequence.padding)),
            ("m2.replay_min", i64::from(sequence.replay_min)),
            ("m2.replay_max", i64::from(sequence.replay_max)),
            ("m2.blend_in", i64::from(blend[0])),
            ("m2.blend_out", i64::from(blend[1])),
            ("m2.variation_next", i64::from(sequence.variation_next)),
        ];
        let mut entries: Vec<_> = raw
            .resolve::<wr::NativeEntry<wr::V2>>(clip.native.entries.cast())?
            .iter()
            .collect::<Result<_, _>>()?;
        let additional = bag(&mut raw, &extra)?;
        entries.extend(
            raw.resolve::<wr::NativeEntry<wr::V2>>(additional.entries.cast())?
                .iter()
                .collect::<Result<Vec<_>, _>>()?,
        );
        clip.native.entries = put(&mut raw, &entries)?.cast();
        clip.bounds = extent(sequence.bounding);
    }
    let id = raw
        .section_id(root.clips.index)
        .ok_or_else(|| Error::new("wem.clips", "missing clip table"))?;
    raw.replace(id, &clips)?;
    Ok(wem::Document::from_raw(raw)?)
}
pub(super) fn clip_metadata(
    model: &sm::Model<'_>,
    index: usize,
) -> Result<BTreeMap<String, i64>, Error> {
    let Some(native) = &model.native else {
        return Ok(BTreeMap::new());
    };
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let clip = native
        .document
        .raw()
        .resolve::<wr::Clip<wr::V7>>(root.clips.cast())?
        .get(index)
        .transpose()?;
    match clip {
        Some(clip) => metadata(native.document.raw(), &clip.native),
        None => Ok(BTreeMap::new()),
    }
}
