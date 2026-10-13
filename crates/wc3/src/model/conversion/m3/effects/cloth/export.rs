//! Active WEM cages, capsule colliders, proxy associations, and controls rebuilt as PHCL.
use super::*;
pub(in crate::model::conversion::m3) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<Ref<nr::targets::ClothPhysics>, Error> {
    let Some(source) = &model.native else {
        return Ok(Ref::default());
    };
    let raw = source.document.raw();
    let root = source.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let graph = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(source.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let cloths = raw
        .resolve::<wr::Cloth<wr::V2>>(graph.physics.cloths.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    if cloths.is_empty() {
        return Ok(Ref::default());
    }
    if source.requires_geometry_remap() {
        context.unsupported("cloth.cage_projection","projected corner vertices or draw sections require explicit cloth cage/proxy remapping")?;
        return Ok(Ref::default());
    }
    let nodes = raw
        .resolve::<wr::Node<wr::V19>>(graph.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let colliders = raw
        .resolve::<wr::ClothCollider<wr::V2>>(graph.physics.colliders.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::new();
    for cloth in cloths {
        if cloth.cage.section != 0 {
            context.unsupported(
                "cloth.cage.section",
                "M3 cages must select a complete exported mesh region",
            )?;
        }
        let mesh = model
            .meshes
            .get(cloth.cage.mesh as usize)
            .ok_or_else(|| Error::new("cloth.cage", "missing mesh"))?;
        if mesh.lod != 0 {
            return Err(Error::new(
                "cloth.cage",
                "M3 cloth cage requires the base mesh division",
            ));
        }
        let mut values = BTreeMap::new();
        let mut texts = BTreeMap::new();
        for node in &nodes {
            let controls = metadata::read(raw, &node.native)?;
            if controls.get("m3.cloth.id").copied() == Some(i64::from(cloth.id)) {
                values = controls;
                texts = text_fields(raw, &node.native)?;
                break;
            }
        }
        let get = |key: &str| values.get(key).copied().unwrap_or(0);
        let array = |key: &str| texts.get(key).map(String::as_str).unwrap_or_default();
        let skin = if let Some(recipe) = &cloth.recipe.value {
            raw.resolve::<wr::U32>(recipe.bones.cast())?
                .iter()
                .map(|row| {
                    let bone = row?.value;
                    let mapped = *bone_map
                        .get(bone as usize)
                        .ok_or_else(|| Error::new("cloth.skin_bones", "missing anchor node"))?;
                    Ok(nr::U16 {
                        value: u16::try_from(mapped).map_err(|_| {
                            Error::new("cloth.skin_bones", "anchor bone exceeds sixteen bits")
                        })?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?
        } else {
            Vec::new()
        };
        let sim = parse::<u8>(array("m3.cloth.sim_enabled"), "cloth.sim_enabled")?
            .into_iter()
            .map(|value| nr::Byte { value })
            .collect::<Vec<_>>();
        let packed_bones = parse::<u32>(array("m3.cloth.vertex_bones"), "cloth.vertex_bones")?
            .into_iter()
            .map(|value| nr::U32 { value })
            .collect::<Vec<_>>();
        let packed_weights =
            parse::<u32>(array("m3.cloth.vertex_weights"), "cloth.vertex_weights")?
                .into_iter()
                .map(|value| nr::U32 { value })
                .collect::<Vec<_>>();
        for (name, len) in [
            ("sim_enabled", sim.len()),
            ("vertex_bones", packed_bones.len()),
            ("vertex_weights", packed_weights.len()),
        ] {
            if len != 0 && len != mesh.positions.len() {
                return Err(Error::new(
                    format!("cloth.{name}"),
                    "particle arrays are stale after cage topology edits",
                ));
            }
        }
        let collider_ids = raw
            .resolve::<wr::U32>(cloth.colliders.cast())?
            .iter()
            .map(|row| row.map(|row| row.value))
            .collect::<Result<Vec<_>, _>>()?;
        let mut capsules = Vec::new();
        for id in collider_ids {
            let collider = colliders
                .iter()
                .find(|row| row.id == id)
                .ok_or_else(|| Error::new("cloth.collider", "missing collider identity"))?;
            if collider.kind != wr::ClothColliderKind::Capsule || collider.body != 0 {
                context.unsupported(
                    "cloth.collider.kind",
                    "M3 cloth requires independent capsule colliders",
                )?;
                continue;
            }
            let bone = if collider.node == u32::MAX {
                0xffff
            } else {
                *bone_map
                    .get(collider.node as usize)
                    .ok_or_else(|| Error::new("cloth.collider.node", "missing attachment node"))?
            };
            capsules.push(nr::ClothCollider {
                transform: collider.transform,
                radius: collider.radius,
                height: collider.length,
                bone,
            });
        }
        let mut proxies = Vec::new();
        for (index, binding) in raw
            .resolve::<wr::ClothBinding>(cloth.bindings.cast())?
            .iter()
            .enumerate()
        {
            let binding = binding?;
            if binding.section.section != 0 {
                context.unsupported(
                    "cloth.binding.section",
                    "M3 proxy must select a complete region",
                )?;
            }
            let proxy = model
                .meshes
                .get(binding.section.mesh as usize)
                .ok_or_else(|| Error::new("cloth.proxy", "missing output mesh"))?;
            if proxy.lod != 0 {
                return Err(Error::new(
                    "cloth.proxy",
                    "M3 cloth proxies require the base mesh division",
                ));
            }
            let vertices_key = format!("m3.cloth.proxy.{index}.vertices");
            let weights_key = format!("m3.cloth.proxy.{index}.weights");
            let vertices = parse::<u64>(array(&vertices_key), &vertices_key)?
                .into_iter()
                .map(|value| nr::U64 { value })
                .collect::<Vec<_>>();
            let weights = parse::<u32>(array(&weights_key), &weights_key)?
                .into_iter()
                .map(|value| nr::U32 { value })
                .collect::<Vec<_>>();
            if vertices.len() != weights.len()
                || (!vertices.is_empty() && vertices.len() != proxy.positions.len())
            {
                return Err(Error::new(
                    "cloth.proxy.vertices",
                    "proxy interpolation arrays are stale after topology edits",
                ));
            }
            proxies.push(nr::ClothProxy {
                proxy_index: binding.section.mesh,
                cloth_index: values
                    .get(&format!("m3.cloth.proxy.{index}.cage"))
                    .copied()
                    .unwrap_or(i64::from(cloth.cage.mesh)) as u32,
                proxy_vertices: append(doc, &vertices)?.cast(),
                proxy_weights: append(doc, &weights)?.cast(),
            });
        }
        let sc2 = cloth.sc2.value.clone().unwrap_or_default();
        let preserve = |key: &str, flag: bool| -> u32 {
            let value = get(key) as u32;
            if (value != 0) == flag {
                value
            } else {
                u32::from(flag)
            }
        };
        let active = nr::AnimationValue {
            interpolation: nr::Interpolation::from_raw(get("m3.cloth.active.interpolation") as u16),
            flags: nr::AnimationFlags(get("m3.cloth.active.flags") as u16),
            animation_id: get("m3.cloth.active.animation_id") as u32,
            reserved: get("m3.cloth.active.reserved") as i32,
            initial_value: preserve("m3.cloth.active.initial", cloth.active.get()),
            null_value: get("m3.cloth.active.null") as u32,
        };
        rows.push(SourceCloth {
            values: values.clone(),
            record: nr::ClothPhysics::<nr::V4> {
                cage_region: cloth.cage.mesh,
                skin_bone_count: values
                    .get("m3.cloth.skin_bone_count")
                    .and_then(|v| u32::try_from(*v).ok())
                    .unwrap_or(skin.len() as u32),
                skin_bones: append(doc, &skin)?.cast(),
                sim_enabled: append(doc, &sim)?.cast(),
                vertex_bones: append(doc, &packed_bones)?.cast(),
                vertex_weights: append(doc, &packed_weights)?.cast(),
                colliders: append(doc, &capsules)?.cast(),
                proxies: append(doc, &proxies)?.cast(),
                density: cloth.density,
                tracking: sc2.tracking,
                stretch_stiffness: cloth.stretch_stiffness,
                horizontal_stiffness: sc2.horizontal_stiffness,
                bending_stiffness: cloth.bend_stiffness,
                damping: cloth.damping,
                friction: cloth.friction,
                gravity: cloth.gravity_scale,
                explosion_scale: sc2.explosion_scale,
                wind_scale: sc2.wind_scale,
                shear_stiffness: sc2.shear_stiffness,
                drag_factor: sc2.drag_factor,
                lift_factor: sc2.lift_factor,
                sphere_stiffness: sc2.sphere_stiffness,
                flatten: preserve("m3.cloth.flatten", sc2.flatten.get()),
                active,
                use_skin_collision: preserve(
                    "m3.cloth.use_skin_collision",
                    sc2.use_skin_collision.get(),
                ),
                skin_offset: sc2.skin_offset,
                skin_exponent: sc2.skin_exponent,
                skin_stiffness: sc2.skin_stiffness,
                local_channels: get("m3.cloth.local_channels") as u32,
                local_wind: cloth.wind,
                ..Default::default()
            },
        });
    }
    encode::append(doc, &rows, context)
}
