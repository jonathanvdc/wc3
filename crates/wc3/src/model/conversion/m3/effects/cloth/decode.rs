//! All PHCL dialects normalized by their statically selected optional fields.
use super::*;
pub(super) fn rows<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::ClothPhysics>,
    context: &mut Context,
) -> Result<Vec<SourceCloth>, Error> {
    if reference.entries == 0 {
        return Ok(Vec::new());
    }
    macro_rules! decode{($v:ty)=>{{


    let mut result = Vec::new();
    for row in source
        .resolve::<nr::ClothPhysics<$v>>(reference.cast())
        .map_err(read_error)?
        .iter()
    {
        let row = row.map_err(read_error)?;
        let mut values = BTreeMap::new();
        values.insert(
            "m3.cloth.version".into(),
            <$v as nr::ClothPhysicsDialect>::VERSION as i64,
        );
        let reserved: Ref<nr::targets::Any> = row.revision0_reserved_0.slot().unwrap_or_default();
        if reserved.entries != 0 {
            context.unsupported("cloth.revision0_reserved_0","unresolved cloth auxiliary reference needs an explicit native payload interpretation")?;
        }
        let reserved: Ref<nr::targets::Any> = row.revision0_reserved_1.slot().unwrap_or_default();
        if reserved.entries != 0 {
            context.unsupported("cloth.revision0_reserved_1","unresolved cloth auxiliary reference needs an explicit native payload interpretation")?;
        }
        let bytes: Option<nr::FixedBytes<4>> = row.revision0_reserved_2.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision0_reserved_2.{index}"),
                    i64::from(value),
                );
            }
        }
        let bytes: Option<nr::FixedBytes<4>> = row.revision1_reserved_0.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision1_reserved_0.{index}"),
                    i64::from(value),
                );
            }
        }
        let bytes: Option<nr::FixedBytes<4>> = row.revision2_reserved_0.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision2_reserved_0.{index}"),
                    i64::from(value),
                );
            }
        }
        let bytes: Option<nr::FixedBytes<4>> = row.revision0_reserved_3.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision0_reserved_3.{index}"),
                    i64::from(value),
                );
            }
        }
        let bytes: Option<nr::FixedBytes<4>> = row.revision1_reserved_1.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision1_reserved_1.{index}"),
                    i64::from(value),
                );
            }
        }
        let bytes: Option<nr::FixedBytes<4>> = row.revision2_reserved_1.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision2_reserved_1.{index}"),
                    i64::from(value),
                );
            }
        }
        let bytes: Option<nr::FixedBytes<4>> = row.revision3_reserved_0.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision3_reserved_0.{index}"),
                    i64::from(value),
                );
            }
        }
        let bytes: Option<nr::FixedBytes<4>> = row.revision3_reserved_1.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision3_reserved_1.{index}"),
                    i64::from(value),
                );
            }
        }
        let bytes: Option<nr::FixedBytes<8>> = row.revision3_reserved_2.slot();
        if let Some(bytes) = bytes {
            for (index, value) in bytes.0.into_iter().enumerate() {
                values.insert(
                    format!("m3.cloth.revision3_reserved_2.{index}"),
                    i64::from(value),
                );
            }
        }
        let active: Option<nr::AnimationValue<u32>> = row.active.slot();
        let legacy: Option<nr::AnimationValue<u32>> = row.legacy_active.slot();
        let active = active.or(legacy).unwrap_or(nr::AnimationValue {
            initial_value: 1,
            ..Default::default()
        });
        metadata::header(&mut values, "cloth.active", &active);
        values.insert(
            "m3.cloth.active.initial".into(),
            i64::from(active.initial_value),
        );
        values.insert("m3.cloth.active.null".into(), i64::from(active.null_value));
        let record = nr::ClothPhysics::<nr::V4> {
            cage_region: row.cage_region,
            skin_bone_count: row.skin_bone_count,
            skin_bones: row.skin_bones,
            sim_enabled: row.sim_enabled,
            vertex_bones: row.vertex_bones,
            vertex_weights: row.vertex_weights,
            colliders: row.colliders,
            density: row.density,
            tracking: row.tracking,
            stretch_stiffness: row.stretch_stiffness,
            horizontal_stiffness: row.horizontal_stiffness,
            bending_stiffness: row.bending_stiffness,
            damping: row.damping,
            friction: row.friction,
            gravity: row.gravity,
            explosion_scale: row.explosion_scale,
            wind_scale: row.wind_scale,
            proxies: {
                let value: Option<Ref<nr::targets::ClothProxy>> = row.proxies.slot();
                value.unwrap_or_default()
            },
            shear_stiffness: {
                let value: Option<f32> = row.shear_stiffness.slot();
                value.unwrap_or_default()
            },
            drag_factor: {
                let value: Option<f32> = row.drag_factor.slot();
                value.unwrap_or_default()
            },
            lift_factor: {
                let value: Option<f32> = row.lift_factor.slot();
                value.unwrap_or_default()
            },
            sphere_stiffness: {
                let value: Option<f32> = row.sphere_stiffness.slot();
                value.unwrap_or_default()
            },
            flatten: {
                let value: Option<u32> = row.flatten.slot();
                value.unwrap_or_default()
            },
            use_skin_collision: {
                let value: Option<u32> = row.use_skin_collision.slot();
                value.unwrap_or_default()
            },
            skin_offset: {
                let value: Option<f32> = row.skin_offset.slot();
                value.unwrap_or_default()
            },
            skin_exponent: {
                let value: Option<f32> = row.skin_exponent.slot();
                value.unwrap_or_default()
            },
            skin_stiffness: {
                let value: Option<f32> = row.skin_stiffness.slot();
                value.unwrap_or_default()
            },
            local_channels: {
                let value: Option<u32> = row.local_channels.slot();
                value.unwrap_or_default()
            },
            local_wind: {
                let value: Option<[f32; 3]> = row.local_wind.slot();
                value.unwrap_or_default()
            },
            active,
            ..Default::default()
        };
        result.push(SourceCloth { record, values });
    }
    Ok(result)


}};}
    match revision(source, reference).map_err(read_error)? {
        0 => decode!(nr::V0),
        1 => decode!(nr::V1),
        2 => decode!(nr::V2),
        3 => decode!(nr::V3),
        4 => decode!(nr::V4),
        _ => Err(Error::new("cloth.version", "unsupported PHCL revision")),
    }
}
