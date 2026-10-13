//! Projector nodes with individually editable native projection and envelope controls.
use super::{
    install, metadata, native, nr, read_error, revision, wem, wr, BTreeMap, Envelope, Error, Ref,
};
use metadata::{animated, restore};
pub(super) fn attach<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::Projector>,
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut Vec<wr::Node<wr::V19>>,
) -> Result<(), Error> {
    if reference.entries == 0 {
        return Ok(());
    }
    macro_rules! read {
        ($v:ty) => {
            for row in source
                .resolve::<nr::Projector<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                let mut values = BTreeMap::new();
                values.insert(
                    "m3.projector.source_version".into(),
                    i64::from(<nr::Projector<$v> as native::Record>::VERSION),
                );
                values.insert(
                    "m3.projector.projection_type".into(),
                    i64::from(r.projection_type),
                );
                values.insert(
                    "m3.projector.material_reference_index".into(),
                    i64::from(r.material_reference_index),
                );
                animated(&mut values, "projector.offset", r.offset);
                for (index, value) in r.offset.initial_value.into_iter().enumerate() {
                    values.insert(
                        format!("m3.projector.offset.initial[{index}]"),
                        i64::from(value.to_bits()),
                    );
                }
                animated(&mut values, "projector.pitch", r.pitch);
                values.insert(
                    "m3.projector.pitch.initial".into(),
                    i64::from(r.pitch.initial_value.to_bits()),
                );
                animated(&mut values, "projector.yaw", r.yaw);
                values.insert(
                    "m3.projector.yaw.initial".into(),
                    i64::from(r.yaw.initial_value.to_bits()),
                );
                animated(&mut values, "projector.roll", r.roll);
                values.insert(
                    "m3.projector.roll.initial".into(),
                    i64::from(r.roll.initial_value.to_bits()),
                );
                animated(&mut values, "projector.field_of_view", r.field_of_view);
                values.insert(
                    "m3.projector.field_of_view.initial".into(),
                    i64::from(r.field_of_view.initial_value.to_bits()),
                );
                animated(&mut values, "projector.aspect_ratio", r.aspect_ratio);
                values.insert(
                    "m3.projector.aspect_ratio.initial".into(),
                    i64::from(r.aspect_ratio.initial_value.to_bits()),
                );
                animated(&mut values, "projector.near", r.near);
                values.insert(
                    "m3.projector.near.initial".into(),
                    i64::from(r.near.initial_value.to_bits()),
                );
                animated(&mut values, "projector.far", r.far);
                values.insert(
                    "m3.projector.far.initial".into(),
                    i64::from(r.far.initial_value.to_bits()),
                );
                animated(
                    &mut values,
                    "projector.box_offset_z_bottom",
                    r.box_offset_z_bottom,
                );
                values.insert(
                    "m3.projector.box_offset_z_bottom.initial".into(),
                    i64::from(r.box_offset_z_bottom.initial_value.to_bits()),
                );
                animated(
                    &mut values,
                    "projector.box_offset_z_top",
                    r.box_offset_z_top,
                );
                values.insert(
                    "m3.projector.box_offset_z_top.initial".into(),
                    i64::from(r.box_offset_z_top.initial_value.to_bits()),
                );
                animated(
                    &mut values,
                    "projector.box_offset_x_left",
                    r.box_offset_x_left,
                );
                values.insert(
                    "m3.projector.box_offset_x_left.initial".into(),
                    i64::from(r.box_offset_x_left.initial_value.to_bits()),
                );
                animated(
                    &mut values,
                    "projector.box_offset_x_right",
                    r.box_offset_x_right,
                );
                values.insert(
                    "m3.projector.box_offset_x_right.initial".into(),
                    i64::from(r.box_offset_x_right.initial_value.to_bits()),
                );
                animated(
                    &mut values,
                    "projector.box_offset_y_front",
                    r.box_offset_y_front,
                );
                values.insert(
                    "m3.projector.box_offset_y_front.initial".into(),
                    i64::from(r.box_offset_y_front.initial_value.to_bits()),
                );
                animated(
                    &mut values,
                    "projector.box_offset_y_back",
                    r.box_offset_y_back,
                );
                values.insert(
                    "m3.projector.box_offset_y_back.initial".into(),
                    i64::from(r.box_offset_y_back.initial_value.to_bits()),
                );
                values.insert(
                    "m3.projector.falloff".into(),
                    i64::from(r.falloff.to_bits()),
                );
                values.insert(
                    "m3.projector.alpha_init".into(),
                    i64::from(r.alpha_init.to_bits()),
                );
                values.insert(
                    "m3.projector.alpha_mid".into(),
                    i64::from(r.alpha_mid.to_bits()),
                );
                values.insert(
                    "m3.projector.alpha_end".into(),
                    i64::from(r.alpha_end.to_bits()),
                );
                values.insert(
                    "m3.projector.lifetime_attack".into(),
                    i64::from(r.lifetime_attack.to_bits()),
                );
                values.insert(
                    "m3.projector.lifetime_attack_to".into(),
                    i64::from(r.lifetime_attack_to.to_bits()),
                );
                values.insert(
                    "m3.projector.lifetime_hold".into(),
                    i64::from(r.lifetime_hold.to_bits()),
                );
                values.insert(
                    "m3.projector.lifetime_hold_to".into(),
                    i64::from(r.lifetime_hold_to.to_bits()),
                );
                values.insert(
                    "m3.projector.lifetime_decay".into(),
                    i64::from(r.lifetime_decay.to_bits()),
                );
                values.insert(
                    "m3.projector.lifetime_decay_to".into(),
                    i64::from(r.lifetime_decay_to.to_bits()),
                );
                values.insert(
                    "m3.projector.attenuation_distance".into(),
                    i64::from(r.attenuation_distance.to_bits()),
                );
                animated(&mut values, "projector.active", r.active);
                values.insert(
                    "m3.projector.active.initial".into(),
                    i64::from(r.active.initial_value),
                );
                values.insert("m3.projector.layer".into(), i64::from(r.layer));
                values.insert("m3.projector.lod_reduce".into(), i64::from(r.lod_reduce));
                values.insert("m3.projector.lod_cut".into(), i64::from(r.lod_cut));
                values.insert("m3.projector.flags".into(), i64::from(r.flags));
                install(
                    doc,
                    nodes,
                    r.bone,
                    wr::NodePayload::Helper(Box::default()),
                    wr::NodeKind::HELPER,
                    values,
                )?;
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        _ => {
            return Err(Error::new(
                "projector.revision",
                "unknown projector revision",
            ));
        }
    }
    Ok(())
}
pub(super) fn export<E: Envelope>(
    rows: &[(BTreeMap<String, i64>, u32)],
    doc: &mut native::Document<'static, E>,
) -> Result<Ref<nr::targets::Projector>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let version = rows[0]
        .0
        .get("m3.projector.source_version")
        .copied()
        .unwrap_or(5);
    if rows
        .iter()
        .any(|(v, _)| v.get("m3.projector.source_version").copied().unwrap_or(5) != version)
    {
        return Err(Error::new(
            "projector.revision",
            "mixed projector revisions require explicit normalization",
        ));
    }
    macro_rules! write {
        ($v:ty) => {{
            let mut output = Vec::new();
            for (values, bone) in rows {
                let get = |key: &str| values.get(key).copied().unwrap_or(0) as u32;
                output.push(nr::Projector::<$v> {
                    projection_type: get("m3.projector.projection_type"),
                    bone: *bone,
                    material_reference_index: get("m3.projector.material_reference_index"),
                    offset: restore(
                        values,
                        "projector.offset",
                        [0, 1, 2].map(|i| {
                            f32::from_bits(get(&format!("m3.projector.offset.initial[{i}]")))
                        }),
                    ),
                    pitch: restore(
                        values,
                        "projector.pitch",
                        f32::from_bits(get("m3.projector.pitch.initial")),
                    ),
                    yaw: restore(
                        values,
                        "projector.yaw",
                        f32::from_bits(get("m3.projector.yaw.initial")),
                    ),
                    roll: restore(
                        values,
                        "projector.roll",
                        f32::from_bits(get("m3.projector.roll.initial")),
                    ),
                    field_of_view: restore(
                        values,
                        "projector.field_of_view",
                        f32::from_bits(get("m3.projector.field_of_view.initial")),
                    ),
                    aspect_ratio: restore(
                        values,
                        "projector.aspect_ratio",
                        f32::from_bits(get("m3.projector.aspect_ratio.initial")),
                    ),
                    near: restore(
                        values,
                        "projector.near",
                        f32::from_bits(get("m3.projector.near.initial")),
                    ),
                    far: restore(
                        values,
                        "projector.far",
                        f32::from_bits(get("m3.projector.far.initial")),
                    ),
                    box_offset_z_bottom: restore(
                        values,
                        "projector.box_offset_z_bottom",
                        f32::from_bits(get("m3.projector.box_offset_z_bottom.initial")),
                    ),
                    box_offset_z_top: restore(
                        values,
                        "projector.box_offset_z_top",
                        f32::from_bits(get("m3.projector.box_offset_z_top.initial")),
                    ),
                    box_offset_x_left: restore(
                        values,
                        "projector.box_offset_x_left",
                        f32::from_bits(get("m3.projector.box_offset_x_left.initial")),
                    ),
                    box_offset_x_right: restore(
                        values,
                        "projector.box_offset_x_right",
                        f32::from_bits(get("m3.projector.box_offset_x_right.initial")),
                    ),
                    box_offset_y_front: restore(
                        values,
                        "projector.box_offset_y_front",
                        f32::from_bits(get("m3.projector.box_offset_y_front.initial")),
                    ),
                    box_offset_y_back: restore(
                        values,
                        "projector.box_offset_y_back",
                        f32::from_bits(get("m3.projector.box_offset_y_back.initial")),
                    ),
                    falloff: f32::from_bits(get("m3.projector.falloff")),
                    alpha_init: f32::from_bits(get("m3.projector.alpha_init")),
                    alpha_mid: f32::from_bits(get("m3.projector.alpha_mid")),
                    alpha_end: f32::from_bits(get("m3.projector.alpha_end")),
                    lifetime_attack: f32::from_bits(get("m3.projector.lifetime_attack")),
                    lifetime_attack_to: f32::from_bits(get("m3.projector.lifetime_attack_to")),
                    lifetime_hold: f32::from_bits(get("m3.projector.lifetime_hold")),
                    lifetime_hold_to: f32::from_bits(get("m3.projector.lifetime_hold_to")),
                    lifetime_decay: f32::from_bits(get("m3.projector.lifetime_decay")),
                    lifetime_decay_to: f32::from_bits(get("m3.projector.lifetime_decay_to")),
                    attenuation_distance: f32::from_bits(get("m3.projector.attenuation_distance")),
                    active: restore(
                        values,
                        "projector.active",
                        get("m3.projector.active.initial"),
                    ),
                    layer: get("m3.projector.layer"),
                    lod_reduce: get("m3.projector.lod_reduce"),
                    lod_cut: get("m3.projector.lod_cut"),
                    flags: get("m3.projector.flags"),
                });
            }
            Ok(super::super::append(doc, &output)?.cast())
        }};
    }
    match version {
        4 => write!(nr::V4),
        5 => write!(nr::V5),
        _ => Err(Error::new(
            "projector.revision",
            "unknown projector revision",
        )),
    }
}
