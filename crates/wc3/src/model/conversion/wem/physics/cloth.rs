//! Cloth cages, bindings, preparation, and contact use actual scoped geometry groups.
use super::super::text;
use super::codec::{
    allocate, append, mapped, node, read_error, read_words, revision, section, words,
    write_section, Optional,
};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::{
    Cloth, ClothBakeInto, ClothCageKind, ClothColliderId, ClothId, ClothRecipe, Physics, Sc2Cloth,
};
use crate::model::semantic::profiles::wem::SectionId;
use crate::model::semantic::{MeshId, NodeId};
use crate::model::wem::{records, InlineOption, RawDocument, Ref};
use std::collections::BTreeMap;
pub(super) fn read<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Cloth>,
    target: &mut Physics<'a>,
    colliders: &BTreeMap<u32, ClothColliderId>,
    nodes: &[NodeId],
    meshes: &[MeshId],
    sections: &[Vec<SectionId>],
) -> Result<BTreeMap<u32, ClothId>, Error> {
    let mut ids = BTreeMap::new();
    if reference.entries == 0 {
        return Ok(ids);
    }
    macro_rules! rows {
        ($revision:ty) => {{
            for value in document
                .resolve::<records::Cloth<$revision>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let value = value.map_err(read_error)?;
                let cloth = cloth(document, &value, colliders, nodes, meshes, sections)?;
                let id = target.add_cloth(cloth);
                if ids.insert(value.id, id).is_some() {
                    return Err(Error::new("wem.physics.cloth", "duplicate cloth identity"));
                }
            }
        }};
    }
    match revision(document, reference)? {
        Some(1) => rows!(records::V1),
        Some(2) => rows!(records::V2),
        _ => {
            return Err(Error::new(
                "wem.physics.cloth",
                "unsupported cloth revision",
            ));
        }
    }
    Ok(ids)
}
fn cloth<'a, D: records::ClothDialect>(
    document: &'a RawDocument<'_>,
    value: &records::Cloth<D>,
    colliders: &BTreeMap<u32, ClothColliderId>,
    nodes: &[NodeId],
    meshes: &[MeshId],
    sections: &[Vec<SectionId>],
) -> Result<Cloth<'a>, Error>
where
    D::Recipe: Optional<records::ClothRecipe>,
{
    let bindings = document
        .resolve::<records::ClothBinding>(value.bindings.cast())
        .map_err(read_error)?
        .iter()
        .map(|binding| section(binding.map_err(read_error)?.section, meshes, sections))
        .collect::<Result<Vec<_>, Error>>()?;
    let collider_ids = read_words(document, value.colliders)?
        .iter()
        .map(|id| {
            colliders.get(&id).copied().ok_or_else(|| {
                Error::new(
                    "wem.physics.cloth",
                    "collision primitive belongs to another simulation domain",
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let recipe = Optional::<records::ClothRecipe>::optional(&value.recipe)
        .map(|recipe| {
            let origins = read_words(document, recipe.from)?;
            if !origins.is_empty() && origins.len() != bindings.len() {
                return Err(Error::new(
                    "wem.physics.cloth.recipe",
                    "binding origin count differs from binding count",
                ));
            }
            let original_bindings = origins
                .iter()
                .zip(&bindings)
                .map(|(index, binding)| {
                    let mesh = meshes
                        .iter()
                        .position(|id| *id == binding.mesh)
                        .ok_or_else(|| {
                            Error::new(
                                "wem.physics.cloth.recipe",
                                "binding mesh belongs to another model",
                            )
                        })?;
                    section(
                        records::SectionRef {
                            mesh: u32::try_from(mesh).map_err(read_error)?,
                            section: index,
                        },
                        meshes,
                        sections,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok::<_, Error>(ClothRecipe {
                name: text::read(document, recipe.name)?,
                bones: read_words(document, recipe.bones)?
                    .iter()
                    .map(|id| node(nodes, id))
                    .collect::<Result<_, _>>()?,
                threshold: recipe.threshold,
                cage: match recipe.cage {
                    records::ClothCageKind::Auto => ClothCageKind::Auto,
                    records::ClothCageKind::AsModelled => ClothCageKind::AsModelled,
                    records::ClothCageKind::Grid => ClothCageKind::Grid,
                    records::ClothCageKind::Strip => ClothCageKind::Strip,
                    records::ClothCageKind::Reduced => ClothCageKind::Reduced,
                    records::ClothCageKind::FromFaces => ClothCageKind::FromFaces,
                    records::ClothCageKind::Count => ClothCageKind::Count,
                    records::ClothCageKind::Unknown(code) => ClothCageKind::Unknown(code),
                },
                particles: recipe.particles,
                reach: recipe.reach,
                pins_by_hand: recipe.pins_by_hand.get(),
                bake_into: match recipe.bake_into {
                    records::ClothBakeInto::Bones => ClothBakeInto::Bones,
                    records::ClothBakeInto::FullDetail => ClothBakeInto::FullDetail,
                    records::ClothBakeInto::Count => ClothBakeInto::Count,
                    records::ClothBakeInto::Unknown(code) => ClothBakeInto::Unknown(code),
                },
                binding_origins: original_bindings,
                cage_origin: if recipe.cage_from == u32::MAX {
                    None
                } else {
                    Some(section(
                        records::SectionRef {
                            mesh: value.cage.mesh,
                            section: recipe.cage_from,
                        },
                        meshes,
                        sections,
                    )?)
                },
            })
        })
        .transpose()?;
    Ok(Cloth {
        cage: section(value.cage.clone(), meshes, sections)?,
        bindings,
        colliders: collider_ids,
        active: value.active.get(),
        density: value.density,
        damping: value.damping,
        friction: value.friction,
        stretch_stiffness: value.stretch_stiffness,
        bend_stiffness: value.bend_stiffness,
        gravity_scale: value.gravity_scale,
        wind: value.wind,
        sc2: value.sc2.value.as_ref().map(|value| Sc2Cloth {
            tracking: value.tracking,
            horizontal_stiffness: value.horizontal_stiffness,
            shear_stiffness: value.shear_stiffness,
            explosion_scale: value.explosion_scale,
            wind_scale: value.wind_scale,
            drag_factor: value.drag_factor,
            lift_factor: value.lift_factor,
            sphere_stiffness: value.sphere_stiffness,
            skin_offset: value.skin_offset,
            skin_exponent: value.skin_exponent,
            skin_stiffness: value.skin_stiffness,
            flatten: value.flatten.get(),
            use_skin_collision: value.use_skin_collision.get(),
        }),
        recipe,
    })
}
pub(super) struct WriteContext<'a> {
    pub colliders: &'a BTreeMap<ClothColliderId, u32>,
    pub nodes: &'a BTreeMap<NodeId, u32>,
    pub meshes: &'a BTreeMap<MeshId, u32>,
    pub sections: &'a BTreeMap<SectionId, u32>,
    pub reserved: &'a BTreeMap<ClothId, u32>,
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    source: &Physics<'_>,
    context: &WriteContext<'_>,
    next: &mut u32,
) -> Result<(Ref<records::targets::Cloth>, BTreeMap<ClothId, u32>), Error> {
    let WriteContext {
        colliders,
        nodes,
        meshes,
        sections,
        reserved,
    } = *context;
    let mut ids = BTreeMap::new();
    let mut rows = Vec::new();
    for (id, value) in source.cloths() {
        let native_id = match reserved.get(&id) {
            Some(id) => *id,
            None => allocate(next)?,
        };
        ids.insert(id, native_id);
        let bindings = value
            .bindings
            .iter()
            .copied()
            .map(|section| {
                Ok(records::ClothBinding {
                    section: write_section(section, meshes, sections)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let colliders = value
            .colliders
            .iter()
            .copied()
            .map(|id| mapped(colliders, id, "cloth collision primitive"))
            .collect::<Result<Vec<_>, Error>>()?;
        let recipe = value
            .recipe
            .as_ref()
            .map(|recipe| {
                if !recipe.binding_origins.is_empty()
                    && recipe.binding_origins.len() != value.bindings.len()
                {
                    return Err(Error::new(
                        "wem.physics.cloth.recipe",
                        "binding origin count differs from binding count",
                    ));
                }
                let from = recipe
                    .binding_origins
                    .iter()
                    .zip(&value.bindings)
                    .map(|(origin, binding)| {
                        if origin.mesh != binding.mesh {
                            return Err(Error::new(
                                "wem.physics.cloth.recipe",
                                "origin must belong to its binding mesh",
                            ));
                        }
                        mapped(sections, origin.section, "cloth binding origin")
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let cage_from = recipe
                    .cage_origin
                    .map(|origin| {
                        if origin.mesh != value.cage.mesh {
                            return Err(Error::new(
                                "wem.physics.cloth.recipe",
                                "cage origin must belong to its cage mesh",
                            ));
                        }
                        mapped(sections, origin.section, "cloth cage origin")
                    })
                    .transpose()?
                    .unwrap_or(u32::MAX);
                let bones = recipe
                    .bones
                    .iter()
                    .copied()
                    .map(|id| mapped(nodes, id, "cloth candidate anchor"))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok::<_, Error>(records::ClothRecipe {
                    name: text::write_bytes(document, recipe.name.as_bytes())?,
                    bones: words(document, bones)?,
                    threshold: recipe.threshold,
                    cage: match recipe.cage {
                        ClothCageKind::Auto => records::ClothCageKind::Auto,
                        ClothCageKind::AsModelled => records::ClothCageKind::AsModelled,
                        ClothCageKind::Grid => records::ClothCageKind::Grid,
                        ClothCageKind::Strip => records::ClothCageKind::Strip,
                        ClothCageKind::Reduced => records::ClothCageKind::Reduced,
                        ClothCageKind::FromFaces => records::ClothCageKind::FromFaces,
                        ClothCageKind::Count => records::ClothCageKind::Count,
                        ClothCageKind::Unknown(code) => records::ClothCageKind::Unknown(code),
                    },
                    particles: recipe.particles,
                    reach: recipe.reach,
                    pins_by_hand: recipe.pins_by_hand.into(),
                    bake_into: match recipe.bake_into {
                        ClothBakeInto::Bones => records::ClothBakeInto::Bones,
                        ClothBakeInto::FullDetail => records::ClothBakeInto::FullDetail,
                        ClothBakeInto::Count => records::ClothBakeInto::Count,
                        ClothBakeInto::Unknown(code) => records::ClothBakeInto::Unknown(code),
                    },
                    from: words(document, from)?,
                    cage_from,
                })
            })
            .transpose()?;
        rows.push(records::Cloth::<records::V2> {
            id: native_id,
            cage: write_section(value.cage, meshes, sections)?,
            bindings: append(document, &bindings)?.cast(),
            colliders: words(document, colliders)?,
            active: value.active.into(),
            density: value.density,
            damping: value.damping,
            friction: value.friction,
            stretch_stiffness: value.stretch_stiffness,
            bend_stiffness: value.bend_stiffness,
            gravity_scale: value.gravity_scale,
            wind: value.wind,
            sc2: InlineOption::from(value.sc2.as_ref().map(|value| records::Sc2ClothParams {
                tracking: value.tracking,
                horizontal_stiffness: value.horizontal_stiffness,
                shear_stiffness: value.shear_stiffness,
                explosion_scale: value.explosion_scale,
                wind_scale: value.wind_scale,
                drag_factor: value.drag_factor,
                lift_factor: value.lift_factor,
                sphere_stiffness: value.sphere_stiffness,
                skin_offset: value.skin_offset,
                skin_exponent: value.skin_exponent,
                skin_stiffness: value.skin_stiffness,
                flatten: value.flatten.into(),
                use_skin_collision: value.use_skin_collision.into(),
            })),
            recipe: InlineOption::from(recipe),
        });
    }
    Ok((append(document, &rows)?.cast(), ids))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::semantic::profiles::wem::{Geometry, Section, Wem};
    use crate::model::semantic::{Bounds, GeometryData, Mesh, Model};
    #[test]
    fn cage_and_original_binding_are_scoped_and_editable() {
        let mut geometry = Geometry::new(GeometryData::default()).unwrap();
        let section = geometry
            .add_section(Section {
                availability: Default::default(),
                name: "cage".into(),
                slot: None,
                rigid_node: None,
                selection_group: 0,
                flags: Default::default(),
                bounds: Bounds::default(),
                metadata: Default::default(),
            })
            .unwrap();
        let mut builder = Model::<Wem>::builder("cloth");
        let mesh = builder.add_mesh(Mesh::new(geometry)).unwrap();
        let mut document = RawDocument::new();
        let bindings = append(
            &mut document,
            &[records::ClothBinding {
                section: records::SectionRef {
                    mesh: 0,
                    section: 0,
                },
            }],
        )
        .unwrap();
        let from = words(&mut document, [0]).unwrap();
        let name = text::write_bytes(&mut document, b"preparation").unwrap();
        let source = append(
            &mut document,
            &[records::Cloth::<records::V2> {
                id: 73,
                cage: records::SectionRef {
                    mesh: 0,
                    section: 0,
                },
                bindings: bindings.cast(),
                density: 2.,
                recipe: InlineOption::from(Some(records::ClothRecipe {
                    name,
                    from,
                    cage_from: 0,
                    ..Default::default()
                })),
                ..Default::default()
            }],
        )
        .unwrap();
        let mut physics = Physics::default();
        let ids = read(
            &document,
            source.cast(),
            &mut physics,
            &BTreeMap::new(),
            &[],
            &[mesh],
            &[vec![section]],
        )
        .unwrap();
        let id = ids[&73];
        let mut cloth = physics.cloth(id).unwrap().clone();
        assert_eq!(cloth.cage.mesh, mesh);
        assert_eq!(
            cloth.recipe.as_ref().unwrap().binding_origins[0].section,
            section
        );
        cloth.density = 11.;
        physics.set_cloth(id, cloth).unwrap();
        let mut output = RawDocument::new();
        let (written, _) = write(
            &mut output,
            &physics,
            &WriteContext {
                colliders: &BTreeMap::new(),
                nodes: &BTreeMap::new(),
                meshes: &BTreeMap::from([(mesh, 0)]),
                sections: &BTreeMap::from([(section, 0)]),
                reserved: &BTreeMap::from([(id, 7)]),
            },
            &mut 70_000,
        )
        .unwrap();
        let values = output
            .resolve::<records::Cloth<records::V2>>(written.cast())
            .unwrap();
        let value = values.get(0).unwrap().unwrap();
        assert_eq!(value.id, 7);
        assert_eq!(value.density, 11.);
        let recipe = value.recipe.value.unwrap();
        assert_eq!(output.string(recipe.name).unwrap(), Some("preparation"));
        assert_eq!(read_words(&output, recipe.from).unwrap().get(0), Some(0));
    }
}
