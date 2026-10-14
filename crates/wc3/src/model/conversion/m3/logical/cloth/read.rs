//! Particle anchors and proxy contributions borrow the encoded numeric arrays.
use super::super::{
    bindings::{self, Bindings},
    materials::{anim, read_error, revision},
    slots::{Slot, Unresolved as _},
    GraphBuilder,
};
use crate::model::conversion::{Context, Error};
use crate::model::m3::{records, Document, Envelope, Ref};
use crate::model::semantic::{
    profiles::m3::{CageInfluence, Cloth, ClothCollider, ClothId, ClothProxy, ClothSystem, System},
    MeshId, NodeId, NodeInfluence, OwnerId, Values,
};
use std::array::from_fn;
use std::collections::BTreeMap;

fn mesh(regions: &[MeshId], index: u32) -> Result<MeshId, Error> {
    regions.get(index as usize).copied().ok_or_else(|| {
        Error::new(
            "m3.cloth.mesh",
            "cloth declaration references an undeclared region",
        )
    })
}
fn anchors<'view, E: Envelope + Send + Sync>(
    document: &'view Document<'_, E>,
    bones: Ref<records::targets::U32>,
    weights: Ref<records::targets::U32>,
    palette: Vec<NodeId>,
) -> Result<Values<'view, [Option<NodeInfluence>; 4]>, Error> {
    let bones = document
        .resolve::<records::U32>(bones.cast())
        .map_err(read_error)?;
    let weights = document
        .resolve::<records::U32>(weights.cast())
        .map_err(read_error)?;
    if bones.len() != weights.len() {
        return Err(Error::new(
            "m3.cloth.anchors",
            "bone and weight rows have different lengths",
        ));
    }
    for index in 0..bones.len() {
        let b = bones
            .get(index)
            .unwrap()
            .map_err(read_error)?
            .value
            .to_le_bytes();
        let w = weights
            .get(index)
            .unwrap()
            .map_err(read_error)?
            .value
            .to_le_bytes();
        for lane in 0..4 {
            if w[lane] != 0 && b[lane] as usize >= palette.len() {
                return Err(Error::new(
                    "m3.cloth.anchors",
                    "nonzero contribution references an undeclared anchor",
                ));
            }
        }
    }
    Ok(Values::view(bones.len(), move |index| {
        let b = bones.get(index).unwrap().unwrap().value.to_le_bytes();
        let w = weights.get(index).unwrap().unwrap().value.to_le_bytes();
        from_fn(|lane| {
            (w[lane] != 0).then(|| NodeInfluence {
                node: palette[b[lane] as usize],
                weight: f32::from(w[lane]) / 255.,
            })
        })
    }))
}
fn proxies<'view, E: Envelope + Send + Sync>(
    document: &'view Document<'_, E>,
    reference: Ref<records::targets::ClothProxy>,
    regions: &[MeshId],
    cages: &BTreeMap<u32, ClothId>,
) -> Result<Vec<ClothProxy<'view>>, Error> {
    let mut output = Vec::new();
    for row in document
        .resolve::<records::ClothProxy>(reference.cast())
        .map_err(read_error)?
        .iter()
    {
        let row = row.map_err(read_error)?;
        let indices = document
            .resolve::<records::U64>(row.proxy_vertices.cast())
            .map_err(read_error)?;
        let weights = document
            .resolve::<records::U32>(row.proxy_weights.cast())
            .map_err(read_error)?;
        if indices.len() != weights.len() {
            return Err(Error::new(
                "m3.cloth.proxy",
                "vertex and weight rows have different lengths",
            ));
        }
        for row in indices.iter() {
            row.map_err(read_error)?;
        }
        for row in weights.iter() {
            row.map_err(read_error)?;
        }
        let influences = Values::view(indices.len(), move |index| {
            let vertices = indices.get(index).unwrap().unwrap().value;
            let weights = weights.get(index).unwrap().unwrap().value.to_le_bytes();
            from_fn(|lane| CageInfluence {
                vertex: ((vertices >> (lane * 16)) & 0xffff) as u32,
                weight: f32::from(weights[lane]) / 255.,
            })
        });
        output.push(ClothProxy {
            mesh: mesh(regions, row.proxy_index)?,
            cloth: *cages.get(&row.cloth_index).ok_or_else(|| {
                Error::new("m3.cloth.proxy", "simulation region has no cloth solver")
            })?,
            influences,
        });
    }
    Ok(output)
}
pub(in crate::model::conversion::m3::logical) fn read<'view, E: Envelope + Send + Sync>(
    document: &'view Document<'_, E>,
    reference: Ref<records::targets::ClothPhysics>,
    regions: &[MeshId],
    nodes: &[NodeId],
    builder: &mut GraphBuilder<'view>,
    registry: &mut Bindings,
    context: &mut Context,
) -> Result<(), Error> {
    if reference.is_empty() {
        return Ok(());
    }
    let mut cloths = Vec::new();
    let mut proxy_refs = Vec::new();
    let mut cages = BTreeMap::new();
    macro_rules! read {
        ($v:ty) => {
            for row in document
                .resolve::<records::ClothPhysics<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                for (field, populated) in [
                    ("revision0_reserved_0", r.revision0_reserved_0.populated()),
                    ("revision0_reserved_1", r.revision0_reserved_1.populated()),
                    ("revision0_reserved_2", r.revision0_reserved_2.populated()),
                    ("revision0_reserved_3", r.revision0_reserved_3.populated()),
                    ("revision1_reserved_0", r.revision1_reserved_0.populated()),
                    ("revision1_reserved_1", r.revision1_reserved_1.populated()),
                    ("revision2_reserved_0", r.revision2_reserved_0.populated()),
                    ("revision2_reserved_1", r.revision2_reserved_1.populated()),
                    ("revision3_reserved_0", r.revision3_reserved_0.populated()),
                    ("revision3_reserved_1", r.revision3_reserved_1.populated()),
                    ("revision3_reserved_2", r.revision3_reserved_2.populated()),
                ] {
                    if populated {
                        context.unsupported(
                            format!("m3.cloth.{field}"),
                            "this populated source field has no established interpretation",
                        )?;
                    }
                }
                let palette = document
                    .resolve::<records::U16>(r.skin_bones.cast())
                    .map_err(read_error)?
                    .iter()
                    .map(|row| {
                        let index = row.map_err(read_error)?.value as usize;
                        nodes.get(index).copied().ok_or_else(|| {
                            Error::new("m3.cloth.anchors", "palette exceeds the skeleton")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let anchors = anchors(document, r.vertex_bones, r.vertex_weights, palette.clone())?;
                let particles = document
                    .resolve::<records::Byte>(r.sim_enabled.cast())
                    .map_err(read_error)?;
                for row in particles.iter() {
                    row.map_err(read_error)?;
                }
                let particles = Values::view(particles.len(), move |index| {
                    particles.get(index).unwrap().unwrap().value
                });
                let colliders = document
                    .resolve::<records::ClothCollider>(r.colliders.cast())
                    .map_err(read_error)?
                    .iter()
                    .map(|row| {
                        let row = row.map_err(read_error)?;
                        Ok(ClothCollider {
                            transform: row.transform,
                            radius: row.radius,
                            height: row.height,
                            bone: if row.bone == 0xffff {
                                None
                            } else {
                                Some(*palette.get(row.bone as usize).ok_or_else(|| {
                                    Error::new(
                                        "m3.cloth.collider",
                                        "collider exceeds the anchor palette",
                                    )
                                })?)
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                let active = Slot::<records::AnimationValue<u32>>::slot(r.active)
                    .or_else(|| Slot::<records::AnimationValue<u32>>::slot(r.legacy_active))
                    .map(anim);
                proxy_refs.push(
                    Slot::<Ref<records::targets::ClothProxy>>::slot(r.proxies).unwrap_or_default(),
                );
                let id = ClothId(cloths.len() as u32);
                if cages.insert(r.cage_region, id).is_some() {
                    return Err(Error::new(
                        "m3.cloth.cage",
                        "multiple solvers declare the same cage region",
                    ));
                }
                cloths.push(Cloth {
                    cage: mesh(regions, r.cage_region)?,
                    anchor_control: r.anchor_control,
                    anchors,
                    sim_enabled: particles,
                    colliders,
                    proxies: Vec::new(),
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
                    shear_stiffness: Slot::<f32>::slot(r.shear_stiffness).unwrap_or_default(),
                    drag_factor: Slot::<f32>::slot(r.drag_factor).unwrap_or_default(),
                    lift_factor: Slot::<f32>::slot(r.lift_factor).unwrap_or_default(),
                    sphere_stiffness: Slot::<f32>::slot(r.sphere_stiffness).unwrap_or_default(),
                    flatten: Slot::<u32>::slot(r.flatten).unwrap_or_default(),
                    active,
                    use_skin_collision: Slot::<u32>::slot(r.use_skin_collision).unwrap_or_default(),
                    skin_offset: Slot::<f32>::slot(r.skin_offset).unwrap_or_default(),
                    skin_exponent: Slot::<f32>::slot(r.skin_exponent).unwrap_or_default(),
                    skin_stiffness: Slot::<f32>::slot(r.skin_stiffness).unwrap_or_default(),
                    local_channels: Slot::<u32>::slot(r.local_channels).unwrap_or_default(),
                    local_wind: Slot::<[f32; 3]>::slot(r.local_wind).unwrap_or_default(),
                });
            }
        };
    }
    match revision(document, reference).map_err(read_error)? {
        0 => read!(records::V0),
        1 => read!(records::V1),
        2 => read!(records::V2),
        3 => read!(records::V3),
        4 => read!(records::V4),
        _ => return Err(Error::new("m3.cloth", "unsupported cloth record revision")),
    }
    for (index, reference) in proxy_refs.into_iter().enumerate() {
        cloths[index].proxies = proxies(document, reference, regions, &cages)?;
    }
    let id = builder.add_system(System::Cloth(ClothSystem::default()))?;
    let mut system = System::Cloth(ClothSystem { cloths });
    bindings::system(&mut system, OwnerId::System(id), registry);
    builder.set_system(id, system)?;
    Ok(())
}
