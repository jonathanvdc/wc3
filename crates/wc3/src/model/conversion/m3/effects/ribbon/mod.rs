//! Ribbon controls mapped into active WEM native payloads.
use super::emitter_fields::property;
use super::metadata::{self, native_property};
use super::{install, native, nr, read_error, revision, wem, wr, Context, Envelope, Error, Ref};
use std::collections::BTreeMap;
mod decode;
mod encode;
use decode::decode;
use encode::encode;
fn spline_points<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: Ref<nr::targets::SplineRibbon>,
    values: &mut BTreeMap<String, i64>,
) -> Result<wem::Ref<wr::targets::Sc2RibbonSplinePoint>, Error> {
    let mut rows = Vec::new();
    for (index, row) in source
        .resolve::<nr::SplineRibbon>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("ribbon.spline_points[{index}]");
        rows.push(wr::Sc2RibbonSplinePoint {
            node: r.bone_index,
            emission_offset: r.emission_offset,
            emission_vector: r.emission_vector,
            reserved: r.reserved,
            emission_vector_norm_factor: r.emission_vector_norm_factor,
            velocity_norm_factor: r.velocity_norm_factor,
            velocity: property(values, &format!("{prefix}.velocity"), r.velocity),
            velocity_base_factor: property(
                values,
                &format!("{prefix}.velocity_base_factor"),
                r.velocity_base_factor,
            ),
            velocity_end_factor: property(
                values,
                &format!("{prefix}.velocity_end_factor"),
                r.velocity_end_factor,
            ),
            yaw: property(values, &format!("{prefix}.yaw"), r.yaw),
            pitch: property(values, &format!("{prefix}.pitch"), r.pitch),
            yaw_variation: wr::Sc2Variation {
                type_: wr::Sc2VariationKind::from_raw((r.yaw_type).raw()),
                amplitude: property(values, &format!("{prefix}.yaw_amplitude"), r.yaw_amplitude),
                frequency: property(values, &format!("{prefix}.yaw_frequency"), r.yaw_frequency),
            },
            pitch_variation: wr::Sc2Variation {
                type_: wr::Sc2VariationKind::from_raw((r.pitch_type).raw()),
                amplitude: property(
                    values,
                    &format!("{prefix}.pitch_amplitude"),
                    r.pitch_amplitude,
                ),
                frequency: property(
                    values,
                    &format!("{prefix}.pitch_frequency"),
                    r.pitch_frequency,
                ),
            },
            velocity_variation: wr::Sc2Variation {
                type_: wr::Sc2VariationKind::from_raw(r.velocity_type.raw()),
                amplitude: property(
                    values,
                    &format!("{prefix}.velocity_amplitude"),
                    r.velocity_amplitude,
                ),
                frequency: property(
                    values,
                    &format!("{prefix}.velocity_frequency"),
                    r.velocity_frequency,
                ),
            },
        });
    }
    Ok(metadata::put(doc, &rows)?.cast())
}
fn native_spline_points<E: Envelope>(
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    reference: wem::Ref<wr::targets::Sc2RibbonSplinePoint>,
    values: &BTreeMap<String, i64>,
    bone_map: &[u32],
) -> Result<Ref<nr::targets::SplineRibbon>, Error> {
    let mut rows = Vec::new();
    for (index, row) in source
        .resolve::<wr::Sc2RibbonSplinePoint>(reference.cast())?
        .iter()
        .enumerate()
    {
        let r = row?;
        let prefix = format!("ribbon.spline_points[{index}]");
        rows.push(nr::SplineRibbon {
            bone_index: *bone_map
                .get(r.node as usize)
                .ok_or_else(|| Error::new("ribbon.spline.node", "missing spline anchor node"))?,
            emission_offset: r.emission_offset,
            emission_vector: r.emission_vector,
            reserved: r.reserved,
            emission_vector_norm_factor: r.emission_vector_norm_factor,
            velocity_norm_factor: r.velocity_norm_factor,
            velocity: native_property(values, &format!("{prefix}.velocity"), r.velocity),
            velocity_base_factor: native_property(
                values,
                &format!("{prefix}.velocity_base_factor"),
                r.velocity_base_factor,
            ),
            velocity_end_factor: native_property(
                values,
                &format!("{prefix}.velocity_end_factor"),
                r.velocity_end_factor,
            ),
            yaw: native_property(values, &format!("{prefix}.yaw"), r.yaw),
            pitch: native_property(values, &format!("{prefix}.pitch"), r.pitch),
            yaw_type: nr::Variation::from_raw(r.yaw_variation.type_.raw()),
            yaw_amplitude: native_property(
                values,
                &format!("{prefix}.yaw_amplitude"),
                r.yaw_variation.amplitude,
            ),
            yaw_frequency: native_property(
                values,
                &format!("{prefix}.yaw_frequency"),
                r.yaw_variation.frequency,
            ),
            pitch_type: nr::Variation::from_raw(r.pitch_variation.type_.raw()),
            pitch_amplitude: native_property(
                values,
                &format!("{prefix}.pitch_amplitude"),
                r.pitch_variation.amplitude,
            ),
            pitch_frequency: native_property(
                values,
                &format!("{prefix}.pitch_frequency"),
                r.pitch_variation.frequency,
            ),
            velocity_type: nr::Variation::from_raw(r.velocity_variation.type_.raw()),
            velocity_amplitude: native_property(
                values,
                &format!("{prefix}.velocity_amplitude"),
                r.velocity_variation.amplitude,
            ),
            velocity_frequency: native_property(
                values,
                &format!("{prefix}.velocity_frequency"),
                r.velocity_variation.frequency,
            ),
        });
    }
    Ok(super::super::append(doc, &rows)?.cast())
}
pub(super) fn attach<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::RibbonEmitter>,
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
                .resolve::<nr::RibbonEmitter<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let bone = u32::from(row.bone_index);
                let mut values = BTreeMap::new();
                let payload = decode(row, source, doc, &mut values)?;
                values.insert("m3.ribbon.source_index".into(), i64::from(source_index));
                source_index += 1;
                install(
                    doc,
                    nodes,
                    bone,
                    wr::NodePayload::Sc2RibbonEmitter(Box::new(payload)),
                    wr::NodeKind::Sc2RibbonEmitter,
                    values,
                )?;
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        6 => read!(nr::V6),
        7 => read!(nr::V7),
        8 => read!(nr::V8),
        9 => read!(nr::V9),
        _ => return Err(Error::new("m3.ribbons", "unsupported ribbon revision")),
    }
    Ok(())
}
pub(super) fn export<E: Envelope>(
    rows: &[(wr::Sc2RibbonEmitterPayload, BTreeMap<String, i64>, u32)],
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    context: &mut Context,
    bone_map: &[u32],
) -> Result<Ref<nr::targets::RibbonEmitter>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let version = rows[0]
        .1
        .get("m3.ribbon.source_version")
        .copied()
        .unwrap_or(9) as u32;
    if rows
        .iter()
        .any(|r| r.1.get("m3.ribbon.source_version").copied().unwrap_or(9) as u32 != version)
    {
        return Err(Error::new(
            "ribbons.revision",
            "mixed source ribbon revisions need an explicit layout normalization",
        ));
    }
    macro_rules! write {
        ($v:ty) => {{
            let mut encoded = Vec::new();
            for (payload, values, bone) in rows {
                encoded.push(encode::<E, $v>(
                    payload.clone(),
                    source,
                    doc,
                    values,
                    *bone,
                    bone_map,
                    context,
                )?);
            }
            Ok(super::super::append(doc, &encoded)?.cast())
        }};
    }
    match version {
        4 => write!(nr::V4),
        5 => write!(nr::V5),
        6 => write!(nr::V6),
        7 => write!(nr::V7),
        8 => write!(nr::V8),
        9 => write!(nr::V9),
        _ => Err(Error::new(
            "ribbons.revision",
            "unsupported ribbon revision",
        )),
    }
}
