//! Canonical emitter tables retain independent definitions and explicit instance overrides.
use super::super::{
    animation::AnimationIds,
    write::{animated, append},
    Graph,
};
use super::{encode, writer::Writer};
use crate::model::conversion::{Context, Error};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{profiles::m3 as surface, FeatureId, MaterialId, MeshId, NodeId};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
pub(in crate::model::conversion::m3::logical) struct Tables {
    pub particles: Ref<nr::targets::ParticleEmitter>,
    pub copies: Ref<nr::targets::ParticleEmitterCopy>,
    pub ribbons: Ref<nr::targets::RibbonEmitter>,
}
pub(in crate::model::conversion::m3::logical) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut native::Document<'_, E>,
    bones: &BTreeMap<NodeId, u32>,
    materials: &BTreeMap<MaterialId, u32>,
    meshes: &BTreeMap<MeshId, Vec<u32>>,
    ids: &AnimationIds,
    context: &mut Context,
) -> Result<Tables, Error> {
    fn indices<'a>(
        graph: &Graph<'a>,
        select: impl Fn(&surface::Feature<'a>) -> bool,
    ) -> BTreeMap<FeatureId, u32> {
        graph
            .features()
            .filter(|(_, feature)| select(&feature.payload))
            .enumerate()
            .map(|(index, (id, _))| (id, index as u32))
            .collect()
    }
    let particles = indices(graph, |p| matches!(p, surface::Feature::Particle(_)));
    let ribbons = indices(graph, |p| matches!(p, surface::Feature::Ribbon(_)));
    let projectors = indices(graph, |p| matches!(p, surface::Feature::Projector(_)));
    let mut writer = Writer {
        document,
        ids,
        context,
        materials,
        meshes,
        particles: &particles,
        ribbons: &ribbons,
        projectors: &projectors,
    };
    let mut copy_rows = Vec::new();
    let mut copy_indices = vec![Vec::new(); particles.len()];
    for (_, feature) in graph.features() {
        let surface::Feature::ParticleCopy(copy) = &feature.payload else {
            continue;
        };
        let source = *particles.get(&copy.source).ok_or_else(|| {
            Error::new(
                "m3.particle_copy.source",
                "copy source is not an independent emitter",
            )
        })? as usize;
        let index = u32::try_from(copy_rows.len())
            .map_err(|_| Error::new("m3.particle_copy", "copy count exceeds native capacity"))?;
        copy_indices[source].push(nr::U32 { value: index });
        copy_rows.push(nr::ParticleEmitterCopy {
            bone_index: *bones.get(&feature.node).ok_or_else(|| {
                Error::new(
                    "m3.particle_copy.node",
                    "copy anchor is absent from native skeleton",
                )
            })?,
            emission_rate: animated(&copy.emission_rate, ids),
            squirt_amount: animated(&copy.squirt_amount, ids),
        });
    }
    let copies = append(writer.document, &copy_rows)?.cast();
    let legacy=graph.features().any(|(_,feature)|matches!(&feature.payload,surface::Feature::Particle(p) if p.noise_smoothness != 0.0));
    macro_rules! encode_particles {
        ($version:ty) => {{
            let mut rows = Vec::new();
            for (id, feature) in graph.features() {
                let surface::Feature::Particle(particle) = &feature.payload else {
                    continue;
                };
                let bone = *bones.get(&feature.node).ok_or_else(|| {
                    Error::new("m3.particle.node", "emitter anchor is absent from skeleton")
                })?;
                let copies =
                    append(writer.document, &copy_indices[particles[&id] as usize])?.cast();
                rows.push(encode::encode::<E, $version>(
                    particle,
                    &mut writer,
                    bone,
                    copies,
                )?);
            }
            append(writer.document, &rows)?.cast()
        }};
    }
    let particles = if legacy {
        encode_particles!(nr::V11)
    } else {
        encode_particles!(nr::V24)
    };
    let legacy_ribbon = graph.features().any(|(_,feature)|matches!(&feature.payload,surface::Feature::Ribbon(ribbon) if ribbon.deprecated_unknown != 0));
    macro_rules! encode_ribbons {
        ($version:ty) => {{
            let mut rows = Vec::new();
            for (_, feature) in graph.features() {
                let surface::Feature::Ribbon(ribbon) = &feature.payload else {
                    continue;
                };
                let bone = *bones.get(&feature.node).ok_or_else(|| {
                    Error::new("m3.ribbon.node", "ribbon anchor is absent from skeleton")
                })?;
                rows.push(super::super::ribbons::encode::encode::<E, $version>(
                    ribbon,
                    &mut writer,
                    bone,
                    bones,
                )?);
            }
            append(writer.document, &rows)?.cast()
        }};
    }
    let ribbons = if legacy_ribbon {
        encode_ribbons!(nr::V6)
    } else {
        encode_ribbons!(nr::V9)
    };
    Ok(Tables {
        particles,
        copies,
        ribbons,
    })
}
