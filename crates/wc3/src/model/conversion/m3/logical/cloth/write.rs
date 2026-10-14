//! Solver data is encoded from actual mesh, node, and cloth relationships.
use super::super::{
    animation::AnimationIds,
    write::{animated, append},
    Graph,
};
use crate::model::conversion::{Context, Error};
use crate::model::m3::{records, Document, Envelope, Ref};
use crate::model::semantic::{
    profiles::m3::{Cloth, System},
    MeshId, NodeId,
};
use std::collections::{btree_map::Entry, BTreeMap};
fn region(regions: &BTreeMap<MeshId, Vec<u32>>, mesh: MeshId) -> Result<u32, Error> {
    regions
        .get(&mesh)
        .and_then(|rows| rows.first())
        .copied()
        .ok_or_else(|| Error::new("m3.cloth.mesh", "cloth mesh has no emitted native region"))
}
fn byte(weight: f32, context: &mut Context) -> Result<u8, Error> {
    let value = (weight * 255.).round();
    if !(0. ..=255.).contains(&value) {
        return Err(Error::new(
            "m3.cloth.weight",
            "contribution is outside the native unit interval",
        ));
    }
    let encoded = value as u8;
    if f32::from(encoded) / 255. != weight {
        context.approximate(
            "m3.cloth.weight",
            (f32::from(encoded) / 255. - weight).abs(),
            "contribution requires eight-bit quantization",
        )?;
    }
    Ok(encoded)
}
pub(in crate::model::conversion::m3::logical) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Document<'_, E>,
    regions: &BTreeMap<MeshId, Vec<u32>>,
    nodes: &BTreeMap<NodeId, u32>,
    ids: &AnimationIds,
    context: &mut Context,
) -> Result<Ref<records::targets::ClothPhysics>, Error> {
    let mut rows = Vec::new();
    for (_, system) in graph.systems() {
        let System::Cloth(system) = system else {
            continue;
        };
        for cloth in &system.cloths {
            rows.push(encode(
                graph,
                document,
                cloth,
                &system.cloths,
                GeometryReferences { regions, nodes },
                ids,
                context,
            )?);
        }
    }
    append(document, &rows).map(Ref::cast)
}
struct GeometryReferences<'a> {
    regions: &'a BTreeMap<MeshId, Vec<u32>>,
    nodes: &'a BTreeMap<NodeId, u32>,
}
fn encode<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Document<'_, E>,
    cloth: &Cloth<'_>,
    cloths: &[Cloth<'_>],
    geometry: GeometryReferences<'_>,
    ids: &AnimationIds,
    context: &mut Context,
) -> Result<records::ClothPhysics<records::V4>, Error> {
    let GeometryReferences { regions, nodes } = geometry;
    let mut palette = BTreeMap::new();
    let mut palette_nodes = Vec::new();
    for row in cloth.anchors.iter() {
        for influence in row.into_iter().flatten() {
            if let Entry::Vacant(entry) = palette.entry(influence.node) {
                let index = u8::try_from(palette_nodes.len()).map_err(|_| {
                    Error::new(
                        "m3.cloth.anchors",
                        "anchor palette exceeds eight-bit capacity",
                    )
                })?;
                entry.insert(u32::from(index));
                palette_nodes.push(influence.node);
            }
        }
    }
    for collider in &cloth.colliders {
        if let Some(node) = collider.bone {
            if let Entry::Vacant(entry) = palette.entry(node) {
                let index = u32::try_from(palette_nodes.len())
                    .map_err(|_| Error::new("m3.cloth.anchors", "palette capacity exceeded"))?;
                entry.insert(index);
                palette_nodes.push(node);
            }
        }
    }
    let palette_rows = palette_nodes
        .iter()
        .map(|node| {
            Ok(records::U16 {
                value: nodes
                    .get(node)
                    .and_then(|index| u16::try_from(*index).ok())
                    .ok_or_else(|| {
                        Error::new(
                            "m3.cloth.anchors",
                            "anchor requires a 16-bit native bone index",
                        )
                    })?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let mut bone_rows = Vec::new();
    let mut weight_rows = Vec::new();
    for row in cloth.anchors.iter() {
        let mut bones = [0u8; 4];
        let mut weights = [0u8; 4];
        for (lane, influence) in row.into_iter().enumerate() {
            if let Some(influence) = influence {
                bones[lane] = u8::try_from(palette[&influence.node]).map_err(|_| {
                    Error::new(
                        "m3.cloth.anchors",
                        "anchor palette exceeds eight-bit capacity",
                    )
                })?;
                weights[lane] = byte(influence.weight, context)?;
            }
        }
        bone_rows.push(records::U32 {
            value: u32::from_le_bytes(bones),
        });
        weight_rows.push(records::U32 {
            value: u32::from_le_bytes(weights),
        });
    }
    let colliders = cloth
        .colliders
        .iter()
        .map(|collider| records::ClothCollider {
            transform: collider.transform,
            radius: collider.radius,
            height: collider.height,
            bone: collider.bone.map_or(0xffff, |node| palette[&node]),
        })
        .collect::<Vec<_>>();
    let mut proxies = Vec::new();
    for proxy in &cloth.proxies {
        let mut vertices = Vec::new();
        let mut weights = Vec::new();
        for row in proxy.influences.iter() {
            let mut vertex_word = 0u64;
            let mut weight_word = [0u8; 4];
            for (lane, influence) in row.into_iter().enumerate() {
                let index = u16::try_from(influence.vertex).map_err(|_| {
                    Error::new("m3.cloth.proxy", "cage vertex exceeds sixteen-bit capacity")
                })?;
                vertex_word |= u64::from(index) << (lane * 16);
                weight_word[lane] = byte(influence.weight, context)?;
            }
            vertices.push(records::U64 { value: vertex_word });
            weights.push(records::U32 {
                value: u32::from_le_bytes(weight_word),
            });
        }
        proxies.push(records::ClothProxy {
            proxy_index: region(regions, proxy.mesh)?,
            cloth_index: region(regions, cloths[proxy.cloth.index()].cage)?,
            proxy_vertices: append(document, &vertices)?.cast(),
            proxy_weights: append(document, &weights)?.cast(),
        });
    }
    let active = cloth
        .active
        .as_ref()
        .map(|value| animated(value, ids))
        .unwrap_or_else(|| records::AnimationValue {
            initial_value: 1,
            null_value: 1,
            ..Default::default()
        });
    let _ = graph;
    Ok(records::ClothPhysics::<records::V4> {
        cage_region: region(regions, cloth.cage)?,
        anchor_control: cloth.anchor_control,
        skin_bones: append(document, &palette_rows)?.cast(),
        sim_enabled: append(
            document,
            &cloth
                .sim_enabled
                .iter()
                .map(|value| records::Byte { value })
                .collect::<Vec<_>>(),
        )?
        .cast(),
        vertex_bones: append(document, &bone_rows)?.cast(),
        vertex_weights: append(document, &weight_rows)?.cast(),
        colliders: append(document, &colliders)?.cast(),
        proxies: append(document, &proxies)?.cast(),
        density: cloth.density,
        tracking: cloth.tracking,
        stretch_stiffness: cloth.stretch_stiffness,
        horizontal_stiffness: cloth.horizontal_stiffness,
        bending_stiffness: cloth.bending_stiffness,
        damping: cloth.damping,
        friction: cloth.friction,
        gravity: cloth.gravity,
        explosion_scale: cloth.explosion_scale,
        wind_scale: cloth.wind_scale,
        shear_stiffness: cloth.shear_stiffness,
        drag_factor: cloth.drag_factor,
        lift_factor: cloth.lift_factor,
        sphere_stiffness: cloth.sphere_stiffness,
        flatten: cloth.flatten,
        active,
        use_skin_collision: cloth.use_skin_collision,
        skin_offset: cloth.skin_offset,
        skin_exponent: cloth.skin_exponent,
        skin_stiffness: cloth.skin_stiffness,
        local_channels: cloth.local_channels,
        local_wind: cloth.local_wind,
        ..Default::default()
    })
}
