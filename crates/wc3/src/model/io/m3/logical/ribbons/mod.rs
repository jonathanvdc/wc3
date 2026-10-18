//! Ribbon definitions and their typed spline attachments.
use super::emitter_links::Links;
pub(super) mod encode;
mod read;
mod spline;
use super::{
    bindings::Bindings,
    materials::{read_error, revision},
    particles::context::Reader,
    GraphBuilder,
};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::{profiles::m3 as surface, FeatureId};
use native::{Envelope, Ref};
pub(super) fn import<'view, E: Envelope>(
    source: &'view native::Container<'_, E>,
    reference: Ref<nr::targets::RibbonEmitter>,
    builder: &mut GraphBuilder<'view>,
    links: &Links<'_, 'view>,
    bindings: &mut Bindings,
    context: &mut Context,
) -> Result<Vec<FeatureId>, Error> {
    let skeleton = links.skeleton;
    let materials = links.materials;
    let meshes = links.meshes;
    if reference.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    macro_rules! import {
        ($version:ty) => {
            for row in source
                .resolve::<nr::RibbonEmitter<$version>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let node = *skeleton.nodes.get(row.bone_index as usize).ok_or_else(|| {
                    Error::new("m3.ribbon.node", "ribbon anchor exceeds skeleton")
                })?;
                let owner = builder.reserve_feature(node);
                let mut reader = Reader {
                    builder: &mut *builder,
                    source,
                    particles: &[],
                    ribbons: &[],
                    projectors: &[],
                    materials,
                    meshes,
                    bindings,
                    context,
                };
                let payload = read::decode(row, &mut reader, &skeleton.nodes)?;
                builder.set_feature(owner, surface::Feature::Ribbon((payload).into()))?;
                result.push(owner);
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        4 => import!(nr::V4),
        5 => import!(nr::V5),
        6 => import!(nr::V6),
        7 => import!(nr::V7),
        8 => import!(nr::V8),
        9 => import!(nr::V9),

        _ => {
            return Err(Error::new(
                "m3.ribbon",
                "unsupported ribbon record revision",
            ));
        }
    }
    Ok(result)
}
