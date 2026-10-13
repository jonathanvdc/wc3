//! Target PHCL revisions selected from explicit source provenance and field capabilities.
use super::*;
fn field<T: PartialEq + Default, F: Assign<T>>(
    value: T,
    key: &str,
    context: &mut Context,
) -> Result<F, Error> {
    if !F::PRESENT && value != T::default() {
        context.unsupported(key, "selected PHCL revision omits this cloth control")?;
    }
    Ok(F::assign(value))
}
pub(super) fn append<E: Envelope>(
    doc: &mut native::Document<'static, E>,
    sources: &[SourceCloth],
    context: &mut Context,
) -> Result<Ref<nr::targets::ClothPhysics>, Error> {
    let version = sources
        .first()
        .and_then(|row| row.values.get("m3.cloth.version"))
        .copied()
        .unwrap_or(4) as u32;
    if sources
        .iter()
        .any(|row| row.values.get("m3.cloth.version").copied().unwrap_or(4) as u32 != version)
    {
        return Err(Error::new(
            "cloth.version",
            "one native cloth array cannot mix record revisions",
        ));
    }
    macro_rules! encode {
        ($v:ty) => {{
            let mut rows = Vec::new();
            for source in sources {
                let r = source.record;
                let values = &source.values;
                if <$v as nr::ClothPhysicsDialect>::VERSION < 3
                    && (r.active.initial_value != 1 || r.active.animation_id != 0)
                {
                    context.unsupported(
                        "cloth.active",
                        "early PHCL revisions omit cloth-enable controls",
                    )?;
                }
                let record = nr::ClothPhysics::<$v> {
                    cage_region: r.cage_region,
                    skin_bone_count: r.skin_bone_count,
                    skin_bones: r.skin_bones,
                    sim_enabled: r.sim_enabled,
                    vertex_bones: r.vertex_bones,
                    vertex_weights: r.vertex_weights,
                    colliders: r.colliders,
                    density: r.density,
                    tracking: r.tracking,
                    stretch_stiffness: r.stretch_stiffness,
                    horizontal_stiffness: r.horizontal_stiffness,
                    bending_stiffness: r.bending_stiffness,
                    damping: r.damping,
                    friction: r.friction,
                    gravity: r.gravity,
                    explosion_scale: r.explosion_scale,
                    wind_scale: r.wind_scale,
                    proxies: field::<Ref<nr::targets::ClothProxy>, _>(
                        r.proxies,
                        "cloth.proxies",
                        context,
                    )?,
                    shear_stiffness: field::<f32, _>(
                        r.shear_stiffness,
                        "cloth.shear_stiffness",
                        context,
                    )?,
                    drag_factor: field::<f32, _>(r.drag_factor, "cloth.drag_factor", context)?,
                    lift_factor: field::<f32, _>(r.lift_factor, "cloth.lift_factor", context)?,
                    sphere_stiffness: field::<f32, _>(
                        r.sphere_stiffness,
                        "cloth.sphere_stiffness",
                        context,
                    )?,
                    flatten: field::<u32, _>(r.flatten, "cloth.flatten", context)?,
                    use_skin_collision: field::<u32, _>(
                        r.use_skin_collision,
                        "cloth.use_skin_collision",
                        context,
                    )?,
                    skin_offset: field::<f32, _>(r.skin_offset, "cloth.skin_offset", context)?,
                    skin_exponent: field::<f32, _>(
                        r.skin_exponent,
                        "cloth.skin_exponent",
                        context,
                    )?,
                    skin_stiffness: field::<f32, _>(
                        r.skin_stiffness,
                        "cloth.skin_stiffness",
                        context,
                    )?,
                    local_channels: field::<u32, _>(
                        r.local_channels,
                        "cloth.local_channels",
                        context,
                    )?,
                    local_wind: field::<[f32; 3], _>(r.local_wind, "cloth.local_wind", context)?,
                    revision0_reserved_0: Default::default(),
                    revision0_reserved_1: Default::default(),
                    revision0_reserved_2: {
                        let mut bytes = [0u8; 4];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision0_reserved_2.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision0_reserved_2",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<4>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision0_reserved_2",
                            context,
                        )?
                    },
                    revision1_reserved_0: {
                        let mut bytes = [0u8; 4];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision1_reserved_0.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision1_reserved_0",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<4>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision1_reserved_0",
                            context,
                        )?
                    },
                    revision2_reserved_0: {
                        let mut bytes = [0u8; 4];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision2_reserved_0.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision2_reserved_0",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<4>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision2_reserved_0",
                            context,
                        )?
                    },
                    revision0_reserved_3: {
                        let mut bytes = [0u8; 4];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision0_reserved_3.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision0_reserved_3",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<4>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision0_reserved_3",
                            context,
                        )?
                    },
                    revision1_reserved_1: {
                        let mut bytes = [0u8; 4];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision1_reserved_1.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision1_reserved_1",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<4>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision1_reserved_1",
                            context,
                        )?
                    },
                    revision2_reserved_1: {
                        let mut bytes = [0u8; 4];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision2_reserved_1.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision2_reserved_1",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<4>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision2_reserved_1",
                            context,
                        )?
                    },
                    revision3_reserved_0: {
                        let mut bytes = [0u8; 4];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision3_reserved_0.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision3_reserved_0",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<4>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision3_reserved_0",
                            context,
                        )?
                    },
                    revision3_reserved_1: {
                        let mut bytes = [0u8; 4];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision3_reserved_1.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision3_reserved_1",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<4>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision3_reserved_1",
                            context,
                        )?
                    },
                    revision3_reserved_2: {
                        let mut bytes = [0u8; 8];
                        for (index, byte) in bytes.iter_mut().enumerate() {
                            *byte = u8::try_from(
                                values
                                    .get(&format!("m3.cloth.revision3_reserved_2.{index}"))
                                    .copied()
                                    .unwrap_or(0),
                            )
                            .map_err(|_| {
                                Error::new(
                                    "cloth.revision3_reserved_2",
                                    "native byte exceeds width",
                                )
                            })?;
                        }
                        field::<nr::FixedBytes<8>, _>(
                            nr::FixedBytes(bytes),
                            "cloth.revision3_reserved_2",
                            context,
                        )?
                    },
                    legacy_active: if <$v as nr::ClothPhysicsDialect>::VERSION == 3 {
                        Assign::assign(r.active)
                    } else {
                        Assign::assign(nr::AnimationValue::<u32>::default())
                    },
                    active: if <$v as nr::ClothPhysicsDialect>::VERSION == 4 {
                        Assign::assign(r.active)
                    } else {
                        Assign::assign(nr::AnimationValue::<u32>::default())
                    },
                };
                rows.push(record);
            }
            Ok(super::append(doc, &rows)?.cast())
        }};
    }
    match version {
        0 => encode!(nr::V0),
        1 => encode!(nr::V1),
        2 => encode!(nr::V2),
        3 => encode!(nr::V3),
        4 => encode!(nr::V4),
        _ => Err(Error::new(
            "cloth.version",
            "unsupported native cloth revision",
        )),
    }
}
