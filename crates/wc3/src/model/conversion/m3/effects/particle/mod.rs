//! Particle controls mapped into active WEM native payloads.
use super::emitter_fields::{property, squirt};
use super::metadata::{self, native_property};
use super::{install, native, nr, read_error, revision, wem, wr, Context, Envelope, Error, Ref};
use std::collections::BTreeMap;

mod copies;
mod decode;
mod encode;
use decode::decode;
use encode::encode;
fn shape_sections<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: Ref<nr::targets::U32>,
) -> Result<wem::Ref<wr::targets::U32>, Error> {
    let rows = source
        .resolve::<nr::U32>(reference.cast())
        .map_err(read_error)?
        .iter()
        .map(|r| r.map(|r| wr::U32 { value: r.value }))
        .collect::<Result<Vec<_>, _>>()
        .map_err(read_error)?;
    Ok(metadata::put(doc, &rows)?.cast())
}
fn native_shape_sections<E: Envelope>(
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    reference: wem::Ref<wr::targets::U32>,
) -> Result<Ref<nr::targets::U32>, Error> {
    let rows = source
        .resolve::<wr::U32>(reference.cast())?
        .iter()
        .map(|r| r.map(|r| nr::U32 { value: r.value }))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(super::super::append(doc, &rows)?.cast())
}
fn spline_points<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: Ref<nr::targets::AnimationValueVector3>,
    values: &mut BTreeMap<String, i64>,
) -> Result<wem::Ref<wr::targets::Sc2PropertyVector3>, Error> {
    let mut rows = Vec::new();
    for (index, row) in source
        .resolve::<nr::AnimationValueVector3>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        rows.push(property(
            values,
            &format!("particle.spline_points[{index}]"),
            row.map_err(read_error)?.value,
        ));
    }
    Ok(metadata::put(doc, &rows)?.cast())
}
fn native_spline_points<E: Envelope>(
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    reference: wem::Ref<wr::targets::Sc2PropertyVector3>,
    values: &BTreeMap<String, i64>,
) -> Result<Ref<nr::targets::AnimationValueVector3>, Error> {
    let rows = source
        .resolve::<wr::Sc2Property<[f32; 3]>>(reference.cast())?
        .iter()
        .enumerate()
        .map(|(index, r)| {
            r.map(|r| nr::AnimationValueVector3 {
                value: native_property(values, &format!("particle.spline_points[{index}]"), r),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(super::super::append(doc, &rows)?.cast())
}
fn models<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: Ref<nr::targets::StringReference>,
) -> Result<wem::Ref<wr::targets::AssetKey>, Error> {
    let mut rows = Vec::new();
    for row in source
        .resolve::<nr::StringReference>(reference.cast())
        .map_err(read_error)?
        .iter()
    {
        let row = row.map_err(read_error)?;
        rows.push(wr::AssetKey {
            group: 0,
            id: u32::MAX,
            path: metadata::text(
                doc,
                source
                    .string(row.value)
                    .map_err(read_error)?
                    .unwrap_or_default(),
            )?,
        });
    }
    Ok(metadata::put(doc, &rows)?.cast())
}
fn native_models<E: Envelope>(
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    reference: wem::Ref<wr::targets::AssetKey>,
) -> Result<Ref<nr::targets::StringReference>, Error> {
    let mut rows = Vec::new();
    for row in source.resolve::<wr::AssetKey>(reference.cast())?.iter() {
        let row = row?;
        if row.id != u32::MAX {
            return Err(Error::new(
                "particle.models",
                "M3 particle model references require filenames",
            ));
        }
        rows.push(nr::StringReference {
            value: super::super::append_string(doc, source.string(row.path)?.unwrap_or_default())?,
        });
    }
    Ok(super::super::append(doc, &rows)?.cast())
}

pub(super) fn attach<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::ParticleEmitter>,
    copies: Ref<nr::targets::ParticleEmitterCopy>,
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut Vec<wr::Node<wr::V19>>,
) -> Result<(), Error> {
    if reference.entries == 0 {
        return Ok(());
    }
    let mut source_index = 0u32;
    macro_rules! read {
        ($v:ty) => {
            for row in source
                .resolve::<nr::ParticleEmitter<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let bone = row.bone_index;
                let copy_indices = row.copy_indices;
                let mut values = BTreeMap::new();
                let payload = decode(row, source, doc, &mut values)?;
                let owner = nodes.len() as u32;
                values.insert("m3.particle.source_index".into(), i64::from(source_index));
                source_index += 1;
                install(
                    doc,
                    nodes,
                    bone,
                    wr::NodePayload::Sc2ParticleEmitter(Box::new(payload.clone())),
                    wr::NodeKind::SC2_PARTICLE_EMITTER,
                    values.clone(),
                )?;
                for copy_index in source
                    .resolve::<nr::U32>(copy_indices.cast())
                    .map_err(read_error)?
                    .iter()
                {
                    let copy_index = copy_index.map_err(read_error)?.value;
                    let copy = source
                        .resolve::<nr::ParticleEmitterCopy>(copies.cast())
                        .map_err(read_error)?
                        .get(copy_index as usize)
                        .ok_or_else(|| {
                            Error::new("particle.copy", "copy index outside copy table")
                        })?
                        .map_err(read_error)?;
                    let mut payload = payload.clone();
                    payload.copy_of = owner;
                    payload.emission_rate =
                        property(&mut values, "particle.emission_rate", copy.emission_rate);
                    payload.squirt_amount =
                        squirt(&mut values, "particle.squirt_amount", copy.squirt_amount);
                    values.remove("m3.particle.source_index");
                    install(
                        doc,
                        nodes,
                        copy.bone_index,
                        wr::NodePayload::Sc2ParticleEmitter(Box::new(payload)),
                        wr::NodeKind::SC2_PARTICLE_EMITTER,
                        values.clone(),
                    )?;
                }
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        10 => read!(nr::V10),
        11 => read!(nr::V11),
        12 => read!(nr::V12),
        13 => read!(nr::V13),
        14 => read!(nr::V14),
        15 => read!(nr::V15),
        16 => read!(nr::V16),
        17 => read!(nr::V17),
        18 => read!(nr::V18),
        19 => read!(nr::V19),
        20 => read!(nr::V20),
        21 => read!(nr::V21),
        22 => read!(nr::V22),
        23 => read!(nr::V23),
        24 => read!(nr::V24),
        _ => return Err(Error::new("m3.particles", "unsupported particle revision")),
    }
    Ok(())
}
pub(super) fn export<E: Envelope>(
    rows: &[(wr::Sc2ParticleEmitterPayload, BTreeMap<String, i64>, u32)],
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    context: &mut Context,
) -> Result<
    (
        Ref<nr::targets::ParticleEmitter>,
        Ref<nr::targets::ParticleEmitterCopy>,
    ),
    Error,
> {
    if rows.is_empty() {
        return Ok((Ref::default(), Ref::default()));
    }
    let (rows, copy_indices, copy_ref) = copies::encode(rows, doc, context)?;
    let version = rows[0]
        .1
        .get("m3.particle.source_version")
        .copied()
        .unwrap_or(24) as u32;
    if rows
        .iter()
        .any(|r| r.1.get("m3.particle.source_version").copied().unwrap_or(24) as u32 != version)
    {
        return Err(Error::new(
            "particles.revision",
            "mixed source particle revisions need an explicit layout normalization",
        ));
    }
    macro_rules! write {
        ($v:ty) => {{
            let mut encoded = Vec::new();
            for (row_index, (payload, values, bone)) in rows.iter().enumerate() {
                encoded.push(encode::<E, $v>(
                    payload.clone(),
                    source,
                    doc,
                    values,
                    *bone,
                    copy_indices[row_index],
                    context,
                )?);
            }
            Ok((super::super::append(doc, &encoded)?.cast(), copy_ref))
        }};
    }
    match version {
        10 => write!(nr::V10),
        11 => write!(nr::V11),
        12 => write!(nr::V12),
        13 => write!(nr::V13),
        14 => write!(nr::V14),
        15 => write!(nr::V15),
        16 => write!(nr::V16),
        17 => write!(nr::V17),
        18 => write!(nr::V18),
        19 => write!(nr::V19),
        20 => write!(nr::V20),
        21 => write!(nr::V21),
        22 => write!(nr::V22),
        23 => write!(nr::V23),
        24 => write!(nr::V24),
        _ => Err(Error::new(
            "particles.revision",
            "unsupported particle revision",
        )),
    }
}
