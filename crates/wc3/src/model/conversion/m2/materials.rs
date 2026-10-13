//! Native shader and texture-unit records linked to WEM material slots.
use super::*;

pub(super) fn patch_materials(
    document: wem::Document<'static>,
    native: &[wr::M2Material],
) -> Result<wem::Document<'static>, Error> {
    if native.is_empty() {
        return Ok(document);
    }
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model = document
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let profile = document
        .raw()
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.profile", "missing profile"))??;
    let mut records: Vec<_> = document
        .raw()
        .resolve::<wr::MaterialRecord>(profile.materials.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    if records.len() != native.len() {
        return Err(Error::new(
            "materials",
            "native/shared material table sizes differ",
        ));
    }
    let mut raw = document.into_raw();
    for (record, native) in records.iter_mut().zip(native) {
        record.native = wr::NativeMaterial::M2(put(&mut raw, from_ref(native))?.cast());
        record.sync = wr::NativeSync::IN_SYNC;
        record.common.flags = wr::MaterialFlags(u32::from(native.render_flags & 3));
        record.common.cull = if native.render_flags & 4 != 0 {
            wr::CullMode::NONE
        } else {
            wr::CullMode::BACK
        };
        record.common.depth.test = wem::Bool::from(native.render_flags & 8 == 0);
        record.common.depth.write = wem::Bool::from(native.render_flags & 16 == 0);
    }
    let id = raw
        .section_id(profile.materials.index)
        .ok_or_else(|| Error::new("wem.materials", "missing material table"))?;
    raw.replace(id, &records)?;
    Ok(wem::Document::from_raw(raw)?)
}
pub(super) fn active_material(
    model: &sm::Model<'_>,
    slot: u32,
) -> Result<Option<(wr::M2Material, bool)>, Error> {
    let Some(native) = &model.native else {
        return Ok(None);
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
        if profile.profile != model.profile {
            continue;
        }
        let index = slot;
        let record = raw
            .resolve::<wr::MaterialRecord>(profile.materials.cast())?
            .get(index as usize)
            .ok_or_else(|| Error::new("wem.material", "binding references missing material"))??;
        if let wr::NativeMaterial::M2(reference) = record.native {
            return Ok(raw
                .resolve::<wr::M2Material>(reference.cast())?
                .get(0)
                .transpose()?
                .map(|mut value| {
                    let authoritative = record.sync == wr::NativeSync::NATIVE_AUTHORITATIVE;
                    if !authoritative {
                        value.render_flags = (value.render_flags & !0x1f)
                            | (model.materials[slot as usize].flags as u16 & 3)
                            | if record.common.cull == wr::CullMode::NONE {
                                4
                            } else {
                                0
                            }
                            | if !record.common.depth.test.get() {
                                8
                            } else {
                                0
                            }
                            | if !record.common.depth.write.get() {
                                16
                            } else {
                                0
                            };
                    }
                    (value, authoritative)
                }));
        }
        return Ok(None);
    }
    Ok(None)
}

pub(super) fn import_animation<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
) -> Result<(), Error>
where
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    for sequence in 0..model.clips.len() {
        let start = core
            .sequences
            .get(sequence)
            .map_or(0, |s| s.timing.window().0);
        for (index, color) in core.colors.iter().enumerate() {
            if let Some(track) = import_track(
                source,
                &color.color,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2000,
                },
                |v| v.to_vec(),
            )? {
                model.clips[sequence].tracks.push(track);
            }
            if let Some(track) = import_track(
                source,
                &color.alpha,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2001,
                },
                |v| vec![f32::from(v) / 32767.],
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
        for (index, weight) in core.texture_weights.iter().enumerate() {
            if let Some(track) = import_track(
                source,
                &weight.weight,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2100,
                },
                |v| vec![f32::from(v) / 32767.],
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
        for (index, transform) in core.texture_transforms.iter().enumerate() {
            for (property, track) in [(2200, &transform.translation), (2202, &transform.scaling)] {
                if let Some(track) = import_track(
                    source,
                    track,
                    &source.key,
                    sequence,
                    start,
                    sm::Target::Property {
                        owner: index as u32,
                        channel: property,
                    },
                    |v| v.to_vec(),
                )? {
                    model.clips[sequence].tracks.push(track);
                }
            }
            if let Some(track) = import_track(
                source,
                &transform.rotation,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2201,
                },
                |v| v.to_vec(),
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
    }
    Ok(())
}
pub(super) fn export_animation<V: ModelLayout>(
    model: &sm::Model<'_>,
    core: &mut Core<'static, V>,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
) -> Result<(), Error>
where
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, [f32; 3]>: ValuesFactory<[f32; 3]>,
    V::TrackValues<'static, [f32; 4]>: ValuesFactory<[f32; 4]>,
    V::TrackValues<'static, i16>: ValuesFactory<i16>,
{
    let data = profile_metadata(model)?;
    let count = |key: &str, codes: &[u32]| -> usize {
        let authored = model
            .clips
            .iter()
            .flat_map(|c| &c.tracks)
            .filter_map(|t| match t.target {
                sm::Target::Property { owner, channel } if codes.contains(&channel) => {
                    Some(owner as usize + 1)
                }
                _ => None,
            })
            .max()
            .unwrap_or(0);
        authored.max(data.get(key).copied().unwrap_or(0) as usize)
    };
    let fixed = |v: &[f32], context: &mut Context, path: &str| -> Result<i16, Error> {
        let value = v
            .first()
            .copied()
            .filter(|_| v.len() == 1)
            .ok_or_else(|| Error::new(path, "opacity needs one component"))?;
        if !value.is_finite()
            || value * 32767. < f32::from(i16::MIN)
            || value * 32767. > f32::from(i16::MAX)
        {
            return Err(Error::new(path, "opacity exceeds signed fixed16"));
        }
        let result = (value * 32767.).round() as i16;
        context.approximate(
            path,
            (value - f32::from(result) / 32767.).abs(),
            "material opacity quantization",
        )?;
        Ok(result)
    };
    for index in 0..count("m2.colors.count", &[2000, 2001]) {
        core.colors.push(native::records::ColorAnimation {
            color: export_track::<V, [f32; 3]>(
                model,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2000,
                },
                starts,
                globals,
                context,
                |v, _, path| {
                    v.try_into()
                        .map_err(|_| Error::new(path, "color needs three components"))
                },
            )?,
            alpha: export_track::<V, i16>(
                model,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2001,
                },
                starts,
                globals,
                context,
                fixed,
            )?,
        });
    }
    for index in 0..count("m2.weights.count", &[2100]) {
        core.texture_weights.push(native::records::TextureWeight {
            weight: export_track::<V, i16>(
                model,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2100,
                },
                starts,
                globals,
                context,
                fixed,
            )?,
        });
    }
    for index in 0..count("m2.transforms.count", &[2200, 2201, 2202]) {
        core.texture_transforms
            .push(native::records::TextureTransform {
                translation: export_track::<V, [f32; 3]>(
                    model,
                    sm::Target::Property {
                        owner: index as u32,
                        channel: 2200,
                    },
                    starts,
                    globals,
                    context,
                    |v, _, path| {
                        v.try_into()
                            .map_err(|_| Error::new(path, "UV translation needs three components"))
                    },
                )?,
                rotation: export_track::<V, [f32; 4]>(
                    model,
                    sm::Target::Property {
                        owner: index as u32,
                        channel: 2201,
                    },
                    starts,
                    globals,
                    context,
                    |v, _, path| {
                        v.try_into()
                            .map_err(|_| Error::new(path, "UV rotation needs four components"))
                    },
                )?,
                scaling: export_track::<V, [f32; 3]>(
                    model,
                    sm::Target::Property {
                        owner: index as u32,
                        channel: 2202,
                    },
                    starts,
                    globals,
                    context,
                    |v, _, path| {
                        v.try_into()
                            .map_err(|_| Error::new(path, "UV scaling needs three components"))
                    },
                )?,
            });
    }
    Ok(())
}

/// Optional legacy texture-frame collections selected by a header layout.
pub trait FlipbookField<'data, V: ModelLayout>: Sized {
    /// Borrows the legacy frame-selection records, empty when the field is absent.
    fn records(&self) -> &[native::records::TextureFlipbook<'data, V>];
    /// Installs records in a target supporting this field.
    fn make(
        records: Vec<native::records::TextureFlipbook<'data, V>>,
        context: &mut Context,
    ) -> Result<Self, Error>;
}
impl<'data, V: ModelLayout> FlipbookField<'data, V> for () {
    fn records(&self) -> &[native::records::TextureFlipbook<'data, V>] {
        &[]
    }
    fn make(
        records: Vec<native::records::TextureFlipbook<'data, V>>,
        context: &mut Context,
    ) -> Result<Self, Error> {
        if !records.is_empty() {
            context.unsupported(
                "textures.flipbooks",
                "target core layout has no legacy texture-frame array",
            )?;
        }
        Ok(())
    }
}
impl<'data, V: ModelLayout> FlipbookField<'data, V>
    for Vec<native::records::TextureFlipbook<'data, V>>
{
    fn records(&self) -> &[native::records::TextureFlipbook<'data, V>] {
        self
    }
    fn make(
        records: Vec<native::records::TextureFlipbook<'data, V>>,
        _: &mut Context,
    ) -> Result<Self, Error> {
        Ok(records)
    }
}
pub(super) fn import_flipbooks<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
) -> Result<(), Error>
where
    for<'data> <D::Layout as native::HeaderLayout>::Flipbooks<'data>:
        FlipbookField<'data, D::Layout>,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    for sequence in 0..model.clips.len() {
        let start = core
            .sequences
            .get(sequence)
            .map_or(0, |s| s.timing.window().0);
        for (index, flipbook) in core.texture_flipbooks.records().iter().enumerate() {
            if let Some(track) = import_track(
                source,
                &flipbook.frames,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2300,
                },
                |v| vec![f32::from(v)],
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
    }
    Ok(())
}
pub(super) fn export_flipbooks<V: ModelLayout>(
    model: &sm::Model<'_>,
    core: &mut Core<'static, V>,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
) -> Result<(), Error>
where
    V::Flipbooks<'static>: FlipbookField<'static, V>,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, u16>: ValuesFactory<u16>,
{
    let data = profile_metadata(model)?;
    let count = (data.get("m2.flipbooks.count").copied().unwrap_or(0) as usize).max(
        model
            .clips
            .iter()
            .flat_map(|clip| &clip.tracks)
            .filter_map(|track| match track.target {
                sm::Target::Property {
                    owner,
                    channel: 2300,
                } => Some(owner as usize + 1),
                _ => None,
            })
            .max()
            .unwrap_or(0),
    );
    let mut records = Vec::new();
    for index in 0..count {
        records.push(native::records::TextureFlipbook {
            frames: export_track::<V, u16>(
                model,
                sm::Target::Property {
                    owner: index as u32,
                    channel: 2300,
                },
                starts,
                globals,
                context,
                |v, _, path| {
                    let value = v
                        .first()
                        .copied()
                        .filter(|_| v.len() == 1)
                        .ok_or_else(|| Error::new(path, "frame key needs one component"))?;
                    if !value.is_finite()
                        || value < 0.
                        || value > f32::from(u16::MAX)
                        || value.fract() != 0.
                    {
                        return Err(Error::new(path, "frame selector exceeds u16"));
                    }
                    Ok(value as u16)
                },
            )?,
        });
    }
    core.texture_flipbooks = FlipbookField::make(records, context)?;
    Ok(())
}

pub(super) fn foreign_native_material(model: &sm::Model<'_>, slot: u32) -> Result<bool, Error> {
    let Some(native) = &model.native else {
        return Ok(false);
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
        if profile.profile != model.profile {
            continue;
        }
        let record = raw
            .resolve::<wr::MaterialRecord>(profile.materials.cast())?
            .get(slot as usize)
            .transpose()?;
        return Ok(record.is_some_and(|record| {
            !matches!(
                record.native,
                wr::NativeMaterial::None | wr::NativeMaterial::M2(_)
            )
        }));
    }
    Ok(false)
}
