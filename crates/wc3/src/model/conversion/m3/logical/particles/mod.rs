//! Independent emitters and their explicitly inherited instances.
use super::emitter_links::Links;
use crate::model::semantic::profiles::m3::properties::{Field, FieldPath};
pub(super) mod context;
mod encode;
mod export;
mod read;
pub(super) mod writer;
pub(super) use export::write;

use super::{
    bindings::Bindings,
    materials::{read_error, revision},
    GraphBuilder,
};
use crate::model::conversion::{Context, Error, IssueKind};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{profiles::m3 as surface, FeatureId};
use context::Reader;
use native::{Envelope, Ref};

pub(super) fn import<'view, E: Envelope>(
    source: &'view native::Document<'_, E>,
    reference: Ref<nr::targets::ParticleEmitter>,
    copies: Ref<nr::targets::ParticleEmitterCopy>,
    builder: &mut GraphBuilder<'view>,
    links: &Links<'_, 'view>,
    bindings: &mut Bindings,
    context: &mut Context,
) -> Result<Vec<FeatureId>, Error> {
    let skeleton = links.skeleton;
    let materials = links.materials;
    let meshes = links.meshes;
    let ribbons = links.ribbons;
    let projectors = links.projectors;
    if reference.is_empty() {
        if !copies.is_empty() {
            context.unsupported(
                "m3.particle_copies",
                "copy table has no independent source emitter",
            )?;
        }
        return Ok(Vec::new());
    }
    macro_rules! import {
        ($version:ty) => {{
            let rows = source
                .resolve::<nr::ParticleEmitter<$version>>(reference.cast())
                .map_err(read_error)?;
            let mut particles = Vec::with_capacity(rows.len());
            for row in rows.iter() {
                let row = row.map_err(read_error)?;
                let node = *skeleton.nodes.get(row.bone_index as usize).ok_or_else(|| {
                    Error::new("m3.particle.node", "emitter anchor exceeds skeleton")
                })?;
                particles.push(builder.reserve_feature(node));
            }
            let copy_rows = source
                .resolve::<nr::ParticleEmitterCopy>(copies.cast())
                .map_err(read_error)?;
            let mut used = vec![false; copy_rows.len()];
            for (index, row) in rows.iter().enumerate() {
                let row = row.map_err(read_error)?;
                let copy_indices = row.copy_indices;
                let mut reader = Reader {
                    source,
                    owner: particles[index],
                    particles: &particles,
                    ribbons,
                    projectors,
                    materials,
                    meshes,
                    bindings,
                    context,
                };
                let payload = read::decode(row, &mut reader)?;
                builder.set_feature(particles[index], surface::Feature::Particle((payload).into()))?;
                for entry in source
                    .resolve::<nr::U32>(copy_indices.cast())
                    .map_err(read_error)?
                    .iter()
                {
                    let copy_index = entry.map_err(read_error)?.value as usize;
                    let copy = copy_rows
                        .get(copy_index)
                        .ok_or_else(|| Error::new("m3.particle.copy", "copy index exceeds table"))?
                        .map_err(read_error)?;
                    if used[copy_index] {
                        reader.context.issue(
                            format!("m3.particle_copies[{copy_index}]"),
                            IssueKind::Normalized,
                            "shared copy storage is represented by a separate inherited instance for each source emitter",
                        );
                    }
                    used[copy_index] = true;
                    let node = *skeleton
                        .nodes
                        .get(copy.bone_index as usize)
                        .ok_or_else(|| {
                            Error::new("m3.particle.copy.node", "copy anchor exceeds skeleton")
                        })?;
                    let owner = builder.reserve_feature(node);
                    reader.owner = owner;
                    let payload = surface::ParticleCopy {
                        source: particles[index],
                        emission_rate: reader
                            .property(&FieldPath::root(Field::ParticleCopy).child(Field::EmissionRate),
copy.emission_rate),
                        squirt_amount: reader
                            .property(&FieldPath::root(Field::ParticleCopy).child(Field::SquirtAmount),
copy.squirt_amount),
                    };
                    builder.set_feature(owner, surface::Feature::ParticleCopy((payload).into()))?;
                }
            }
            for (index, used) in used.iter().enumerate() {
                if !used {
                    context.unsupported(
                        format!("m3.particle_copies[{index}]"),
                        "unreferenced copy has no authored source emitter",
                    )?;
                }
            }
            particles
        }};
    }
    Ok(match revision(source, reference).map_err(read_error)? {
        10 => import!(nr::V10),
        11 => import!(nr::V11),
        12 => import!(nr::V12),
        13 => import!(nr::V13),
        14 => import!(nr::V14),
        15 => import!(nr::V15),
        16 => import!(nr::V16),
        17 => import!(nr::V17),
        18 => import!(nr::V18),
        19 => import!(nr::V19),
        20 => import!(nr::V20),
        21 => import!(nr::V21),
        22 => import!(nr::V22),
        23 => import!(nr::V23),
        24 => import!(nr::V24),
        _ => {
            return Err(Error::new(
                "m3.particles",
                "unsupported particle record revision",
            ));
        }
    })
}
