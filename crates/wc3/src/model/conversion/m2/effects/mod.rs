//! Native effect exports and collision graph reconstruction.
use super::*;
mod attachments;
mod cameras;
mod events;
mod lights;
mod particles;
mod ribbons;

struct SceneExport<'view, 'data, V: ModelLayout> {
    model: &'view sm::Model<'data>,
    core: &'view mut Core<'static, V>,
    starts: &'view [u32],
    globals: &'view mut Vec<u32>,
    context: &'view mut Context,
}

pub(super) fn export_scene<V: ModelLayout + ParticleConversion + CameraConversion>(
    model: &sm::Model<'_>,
    core: &mut Core<'static, V>,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
) -> Result<(), Error>
where
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, f32>: ValuesFactory<f32>,
    V::TrackValues<'static, u8>: ValuesFactory<u8>,
    V::TrackValues<'static, u32>: ValuesFactory<u32>,
    V::TrackValues<'static, i16>: ValuesFactory<i16>,
    V::TrackValues<'static, u16>: ValuesFactory<u16>,
    V::SplineValues<'static, f32>: SplineFactory<f32>,
    V::SplineValues<'static, [f32; 3]>: SplineFactory<[f32; 3]>,
    V::TrackValues<'static, [f32; 3]>: ValuesFactory<[f32; 3]>,
{
    let nodes = native_nodes(model)?;
    for (index, node) in nodes.iter().enumerate() {
        let common = &model.nodes[index];
        let metadata = metadata(model.native.as_ref().unwrap().document.raw(), &node.native)?;
        match &node.payload {
            wr::NodePayload::Attachment(payload) => attachments::export(
                &mut SceneExport {
                    model,
                    core,
                    starts,
                    globals,
                    context,
                },
                index,
                node,
                common,
                &metadata,
                payload,
            )?,
            wr::NodePayload::Light(payload) => lights::export(
                &mut SceneExport {
                    model,
                    core,
                    starts,
                    globals,
                    context,
                },
                index,
                node,
                common,
                &metadata,
                payload,
            )?,
            wr::NodePayload::M2ParticleEmitter(payload) => particles::export(
                &mut SceneExport {
                    model,
                    core,
                    starts,
                    globals,
                    context,
                },
                index,
                node,
                common,
                &metadata,
                payload,
            )?,
            wr::NodePayload::Wc3RibbonEmitter(payload) => ribbons::export(
                &mut SceneExport {
                    model,
                    core,
                    starts,
                    globals,
                    context,
                },
                index,
                node,
                common,
                &metadata,
                payload,
            )?,
            wr::NodePayload::Camera(payload) => cameras::export(
                &mut SceneExport {
                    model,
                    core,
                    starts,
                    globals,
                    context,
                },
                index,
                node,
                common,
                &metadata,
                payload,
            )?,
            wr::NodePayload::Event(payload) => events::export(
                &mut SceneExport {
                    model,
                    core,
                    starts,
                    globals,
                    context,
                },
                index,
                node,
                common,
                &metadata,
                payload,
            )?,
            _ => {}
        }
    }
    if let Some(native) = &model.native {
        let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
        let model_record = native
            .document
            .raw()
            .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
            .get(native.model as usize)
            .ok_or_else(|| Error::new("wem.model", "missing model"))??;
        for body in native
            .document
            .raw()
            .resolve::<wr::PhysicsBody<wr::V3>>(model_record.physics.bodies.cast())?
            .iter()
        {
            let body = body?;
            if body.node != u32::MAX {
                continue;
            }
            if body.motion != wr::BodyMotion::STATIC {
                context.unsupported(
                    "physics.body.motion",
                    "M2 collision tables cannot express a dynamic body",
                )?;
                continue;
            }
            for shape in native
                .document
                .raw()
                .resolve::<wr::PhysicsShape<wr::V2>>(body.shapes.cast())?
                .iter()
            {
                let shape = shape?;
                if shape.kind != wr::PhysicsShapeKind::TRIANGLE_MESH {
                    context.unsupported(
                        "physics.shape.kind",
                        "M2 collision output currently requires explicit triangulated geometry",
                    )?;
                    continue;
                }
                let base = core.collision_vertices.len();
                let mut vertices: Vec<_> = core.collision_vertices.iter().collect();
                vertices.extend(
                    native
                        .document
                        .raw()
                        .resolve::<wr::Vector3>(shape.vertices.cast())?
                        .iter()
                        .map(|v| v.map(|v| v.value))
                        .collect::<Result<Vec<_>, _>>()?,
                );
                let mut indices: Vec<_> = core.collision_indices.iter().collect();
                for index in native
                    .document
                    .raw()
                    .resolve::<wr::U32>(shape.triangles.cast())?
                    .iter()
                {
                    let index = index?.value as usize + base;
                    indices.push(u16::try_from(index).map_err(|_| {
                        Error::new("collision.indices", "collision index exceeds u16")
                    })?);
                }
                let mut normals = Vec::new();
                for triangle in indices.as_chunks::<3>().0 {
                    let [a, b, c] = [
                        vertices[usize::from(triangle[0])],
                        vertices[usize::from(triangle[1])],
                        vertices[usize::from(triangle[2])],
                    ];
                    let ab = from_fn::<_, 3, _>(|i| b[i] - a[i]);
                    let ac = from_fn::<_, 3, _>(|i| c[i] - a[i]);
                    let normal = [
                        ab[1] * ac[2] - ab[2] * ac[1],
                        ab[2] * ac[0] - ab[0] * ac[2],
                        ab[0] * ac[1] - ab[1] * ac[0],
                    ];
                    let length = normal.iter().map(|v| v * v).sum::<f32>().sqrt();
                    normals.push(if length > 0. {
                        normal.map(|v| v / length)
                    } else {
                        [0.; 3]
                    });
                }
                core.collision_vertices = ZeroVec::alloc_from_slice(&vertices);
                core.collision_indices = ZeroVec::alloc_from_slice(&indices);
                core.collision_normals = ZeroVec::alloc_from_slice(&normals);
            }
        }
    }
    Ok(())
}

fn actor_has_transform(model: &sm::Model<'_>, index: usize, common: &sm::Node<'_>) -> bool {
    common.translation!=[0.;3]||common.rotation!=[0.,0.,0.,1.]||common.scale!=[1.;3]||model.clips.iter().flat_map(|clip|&clip.tracks).any(|track|matches!(track.target,sm::Target::Translation(owner)|sm::Target::Rotation(owner)|sm::Target::Scale(owner)if owner as usize==index))
}
fn actor_bone(model: &sm::Model<'_>, index: usize, common: &sm::Node<'_>) -> u32 {
    if actor_has_transform(model, index, common) {
        index as u32
    } else {
        common.parent.unwrap_or(0)
    }
}
