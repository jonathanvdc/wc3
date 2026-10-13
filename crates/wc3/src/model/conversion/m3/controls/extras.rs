//! Turret revisions and trailing-model property controls.
use super::*;
pub(super) fn import_turret_behaviors<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::TurretBehavior>,
    fields: &mut Fields,
) -> Result<(), Error> {
    if reference.entries == 0 {
        return Ok(());
    }
    let version = revision(doc, reference).map_err(read_error)?;
    fields.uint("m3.controls.turret_behaviors.count", reference.entries);
    fields.uint("m3.controls.turret_behaviors.version", version);
    macro_rules! read {
        ($v:ty) => {
            for (index, row) in doc
                .resolve::<nr::TurretBehavior<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let r = row.map_err(read_error)?;
                let prefix = format!("m3.controls.turret_behaviors[{index}]");
                fields.floats(
                    &format!("{prefix}.transform"),
                    r.transform.into_iter().flatten(),
                );
                let unknown1: [f32; 4] = r.unknown1.value().unwrap_or_default();
                fields.floats(&format!("{prefix}.unknown1"), unknown1);
                let unknown2: [f32; 4] = r.unknown2.value().unwrap_or_default();
                fields.floats(&format!("{prefix}.unknown2"), unknown2);
                fields.uint(&format!("{prefix}.bone_index"), u32::from(r.bone_index));
                fields.uint(
                    &format!("{prefix}.use_as_main_turret"),
                    u32::from(r.use_as_main_turret),
                );
                fields.uint(
                    &format!("{prefix}.turret_group_id"),
                    u32::from(r.turret_group_id),
                );
                fields.uint(&format!("{prefix}.yaw_limited"), u32::from(r.yaw_limited));
                fields.float(&format!("{prefix}.yaw_min"), r.yaw_min);
                fields.float(&format!("{prefix}.yaw_max"), r.yaw_max);
                let yaw_weight: f32 = r.yaw_weight.value().unwrap_or_default();
                fields.float(&format!("{prefix}.yaw_weight"), yaw_weight);
                fields.uint(
                    &format!("{prefix}.pitch_limited"),
                    u32::from(r.pitch_limited),
                );
                fields.float(&format!("{prefix}.pitch_min"), r.pitch_min);
                fields.float(&format!("{prefix}.pitch_max"), r.pitch_max);
                let pitch_weight: f32 = r.pitch_weight.value().unwrap_or_default();
                fields.float(&format!("{prefix}.pitch_weight"), pitch_weight);
                fields.float(&format!("{prefix}.unknown3"), r.unknown3);
                fields.float(&format!("{prefix}.unknown4"), r.unknown4);
                fields.floats(
                    &format!("{prefix}.main_bone_offset"),
                    r.main_bone_offset.into_iter(),
                );
            }
        };
    }
    match version {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        4 => read!(nr::V4),
        _ => {
            return Err(Error::new(
                "controls.turret_behaviors",
                "unsupported behavior revision",
            ));
        }
    }
    Ok(())
}
pub(super) fn export_turret_behaviors<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<Ref<nr::targets::TurretBehavior>, Error> {
    let version = fields.uint("m3.controls.turret_behaviors.version");
    if version < 4 {
        for index in 0..fields.uint("m3.controls.turret_behaviors.count") {
            let prefix = format!("m3.controls.turret_behaviors[{index}]");
            for key in ["unknown1", "unknown2"] {
                if fields
                    .floats(&format!("{prefix}.{key}"))?
                    .iter()
                    .any(|v| *v != 0.)
                {
                    context.unsupported(
                        format!("{prefix}.{key}"),
                        "selected turret revision omits this control",
                    )?;
                }
            }
            for key in ["yaw_weight", "pitch_weight"] {
                if fields.float(&format!("{prefix}.{key}")) != 0. {
                    context.unsupported(
                        format!("{prefix}.{key}"),
                        "selected turret revision omits this control",
                    )?;
                }
            }
        }
    }
    macro_rules! write {
        ($v:ty) => {{
            let mut rows = Vec::new();
            for index in 0..fields.uint("m3.controls.turret_behaviors.count") {
                let prefix = format!("m3.controls.turret_behaviors[{index}]");
                rows.push(nr::TurretBehavior::<$v> {
                    transform: {
                        let a = fields.array::<16>(&format!("{prefix}.transform"))?;
                        [
                            a[0..4].try_into().unwrap(),
                            a[4..8].try_into().unwrap(),
                            a[8..12].try_into().unwrap(),
                            a[12..16].try_into().unwrap(),
                        ]
                    },
                    unknown1: assign(fields.array::<4>(&format!("{prefix}.unknown1"))?),
                    unknown2: assign(fields.array::<4>(&format!("{prefix}.unknown2"))?),
                    bone_index: u16::try_from(u32::from(bone(
                        fields.uint(&format!("{prefix}.bone_index")),
                        bone_map,
                        &format!("{prefix}.bone_index"),
                    )?))
                    .map_err(|_| {
                        Error::new(format!("{prefix}.bone_index"), "value exceeds native width")
                    })?,
                    use_as_main_turret: u8::try_from(
                        fields.uint(&format!("{prefix}.use_as_main_turret")),
                    )
                    .map_err(|_| {
                        Error::new(
                            format!("{prefix}.use_as_main_turret"),
                            "value exceeds native width",
                        )
                    })?,
                    turret_group_id: u8::try_from(
                        fields.uint(&format!("{prefix}.turret_group_id")),
                    )
                    .map_err(|_| {
                        Error::new(
                            format!("{prefix}.turret_group_id"),
                            "value exceeds native width",
                        )
                    })?,
                    yaw_limited: fields.uint(&format!("{prefix}.yaw_limited")),
                    yaw_min: fields.float(&format!("{prefix}.yaw_min")),
                    yaw_max: fields.float(&format!("{prefix}.yaw_max")),
                    yaw_weight: assign(fields.float(&format!("{prefix}.yaw_weight"))),
                    pitch_limited: fields.uint(&format!("{prefix}.pitch_limited")),
                    pitch_min: fields.float(&format!("{prefix}.pitch_min")),
                    pitch_max: fields.float(&format!("{prefix}.pitch_max")),
                    pitch_weight: assign(fields.float(&format!("{prefix}.pitch_weight"))),
                    unknown3: fields.float(&format!("{prefix}.unknown3")),
                    unknown4: fields.float(&format!("{prefix}.unknown4")),
                    main_bone_offset: fields.array(&format!("{prefix}.main_bone_offset"))?,
                });
            }
            Ok(append(doc, &rows)?.cast())
        }};
    }
    match version {
        0 => write!(nr::V0),
        1 => write!(nr::V1),
        2 => write!(nr::V2),
        3 => write!(nr::V3),
        4 => write!(nr::V4),
        _ => Err(Error::new(
            "controls.turret_behaviors",
            "unknown behavior revision",
        )),
    }
}
pub(super) fn import_trailing_models<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::TrailingModel>,
    fields: &mut Fields,
) -> Result<(), Error> {
    if reference.entries == 0 {
        return Ok(());
    }
    let version = revision(doc, reference).map_err(read_error)?;
    fields.uint("m3.controls.trailing_models.count", reference.entries);
    fields.uint("m3.controls.trailing_models.version", version);
    macro_rules! read {
        ($v:ty) => {
            for (index, row) in doc
                .resolve::<nr::TrailingModel<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let r = row.map_err(read_error)?;
                let prefix = format!("m3.controls.trailing_models[{index}]");
                let vectors: Ref<nr::targets::Vector3> = r.vectors.value().unwrap_or_default();
                fields.floats(
                    &format!("{prefix}.vectors"),
                    doc.resolve::<nr::Vector3>(vectors.cast())
                        .map_err(read_error)?
                        .iter()
                        .map(|r| r.map(|r| r.value))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(read_error)?
                        .into_iter()
                        .flatten(),
                );
                fields.float(&format!("{prefix}.param0"), r.param0);
                fields.float(&format!("{prefix}.param1"), r.param1);
                metadata::header(
                    &mut fields.values,
                    &format!("controls.trailing_models[{index}].anim_float0"),
                    &r.anim_float0,
                );
                fields.float(
                    &format!("{prefix}.anim_float0.initial"),
                    r.anim_float0.initial_value,
                );
                fields.float(
                    &format!("{prefix}.anim_float0.null"),
                    r.anim_float0.null_value,
                );
                metadata::header(
                    &mut fields.values,
                    &format!("controls.trailing_models[{index}].anim_float1"),
                    &r.anim_float1,
                );
                fields.float(
                    &format!("{prefix}.anim_float1.initial"),
                    r.anim_float1.initial_value,
                );
                fields.float(
                    &format!("{prefix}.anim_float1.null"),
                    r.anim_float1.null_value,
                );
                fields.uint(&format!("{prefix}.flag"), u32::from(r.flag));
                fields.uint(&format!("{prefix}.reserved0"), u32::from(r.reserved0));
                fields.uint(&format!("{prefix}.reserved1"), u32::from(r.reserved1));
            }
        };
    }
    match version {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        _ => {
            return Err(Error::new(
                "controls.trailing_models",
                "unsupported behavior revision",
            ));
        }
    }
    Ok(())
}
pub(super) fn export_trailing_models<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    _bone_map: &[u32],
) -> Result<Ref<nr::targets::TrailingModel>, Error> {
    let version = fields.uint("m3.controls.trailing_models.version");
    macro_rules! write {
        ($v:ty) => {{
            let mut rows = Vec::new();
            for index in 0..fields.uint("m3.controls.trailing_models.count") {
                let prefix = format!("m3.controls.trailing_models[{index}]");
                rows.push(nr::TrailingModel::<$v> {
                    vectors: assign({
                        let values = fields.floats(&format!("{prefix}.vectors"))?;
                        if !values.len().is_multiple_of(3) {
                            return Err(Error::new(
                                format!("{prefix}.vectors"),
                                "vector triples required",
                            ));
                        }
                        let rows = values
                            .as_chunks::<3>()
                            .0
                            .iter()
                            .map(|v| nr::Vector3 {
                                value: [v[0], v[1], v[2]],
                            })
                            .collect::<Vec<_>>();
                        append(doc, &rows)?.cast()
                    }),
                    param0: fields.float(&format!("{prefix}.param0")),
                    param1: fields.float(&format!("{prefix}.param1")),
                    anim_float0: metadata::native_property(
                        &fields.values,
                        &format!("controls.trailing_models[{index}].anim_float0"),
                        wr::Sc2Property {
                            init_value: fields.float(&format!("{prefix}.anim_float0.initial")),
                            null_value: fields.float(&format!("{prefix}.anim_float0.null")),
                        },
                    ),
                    anim_float1: metadata::native_property(
                        &fields.values,
                        &format!("controls.trailing_models[{index}].anim_float1"),
                        wr::Sc2Property {
                            init_value: fields.float(&format!("{prefix}.anim_float1.initial")),
                            null_value: fields.float(&format!("{prefix}.anim_float1.null")),
                        },
                    ),
                    flag: fields.uint(&format!("{prefix}.flag")),
                    reserved0: fields.uint(&format!("{prefix}.reserved0")),
                    reserved1: fields.uint(&format!("{prefix}.reserved1")),
                });
            }
            Ok(append(doc, &rows)?.cast())
        }};
    }
    match version {
        0 => write!(nr::V0),
        1 => write!(nr::V1),
        _ => Err(Error::new(
            "controls.trailing_models",
            "unknown behavior revision",
        )),
    }
}
