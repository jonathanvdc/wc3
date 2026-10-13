//! WEM hierarchy construction, typed native payloads, and collision graphs.
use super::*;
mod attachments;
mod cameras;
mod events;
mod lights;
mod particles;
mod ribbons;

pub(super) struct NodePatch {
    pub(super) particle: Option<usize>,
    pub(super) texts: Vec<(&'static str, String)>,
    pub(super) index: usize,
    pub(super) payload: wr::NodePayload<wr::V19>,
    pub(super) kind: wr::NodeKind,
    pub(super) metadata: Vec<(&'static str, i64)>,
}

pub(super) fn import_scene<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
) -> Result<Vec<NodePatch>, Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let mut patches = Vec::new();
    attachments::import_attachments(source, model, &mut patches)?;
    lights::import_lights(source, model, &mut patches)?;
    events::import_events(source, model, &mut patches)?;
    cameras::import_cameras(source, model, &mut patches)?;
    ribbons::import_ribbons(source, model, &mut patches)?;
    particles::import_particles(source, model, &mut patches)?;
    Ok(patches)
}

pub(super) fn patch_scene<V: ModelLayout + ParticleConversion>(
    document: wem::Document<'static>,
    patches: Vec<NodePatch>,
    core: &Core<'_, V>,
) -> Result<wem::Document<'static>, Error> {
    let collision_vertices: Vec<_> = core.collision_vertices.iter().collect();
    let collision_triangles: Vec<_> = core.collision_indices.iter().collect();
    let core_collision = (!collision_vertices.is_empty()).then_some((
        collision_vertices.as_slice(),
        collision_triangles.as_slice(),
    ));
    if patches.is_empty() && core_collision.is_none() {
        return Ok(document);
    }
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    let mut model = document
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut nodes: Vec<_> = document
        .raw()
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut doc = document.into_raw();
    for patch in patches {
        let node = &mut nodes[patch.index];
        node.kind = patch.kind;
        node.payload = if let Some(index) = patch.particle {
            wr::NodePayload::M2ParticleEmitter(Box::new(V::import_particle(
                &core.particles[index],
                &mut doc,
            )?))
        } else {
            patch.payload
        };
        node.native = bag(&mut doc, &patch.metadata)?;
        add_texts(&mut doc, &mut node.native, &patch.texts)?;
    }
    let id = doc
        .section_id(model.nodes.nodes.index)
        .ok_or_else(|| Error::new("wem.nodes", "missing node section"))?;
    doc.replace(id, &nodes)?;
    if let Some((vertices, triangles)) = core_collision {
        let vertices: Vec<_> = vertices
            .iter()
            .map(|&value| wr::Vector3 { value })
            .collect();
        let triangles: Vec<_> = triangles
            .iter()
            .map(|&value| wr::U32 {
                value: u32::from(value),
            })
            .collect();
        let shape = wr::PhysicsShape::<wr::V2> {
            kind: wr::PhysicsShapeKind::TRIANGLE_MESH,
            transform: from_fn(|row| from_fn(|col| if row == col { 1. } else { 0. })),
            vertices: put(&mut doc, &vertices)?.cast(),
            triangles: put(&mut doc, &triangles)?.cast(),
            ..Default::default()
        };
        let shape = put(&mut doc, &[shape])?.cast();
        let body = wr::PhysicsBody::<wr::V3> {
            id: 1,
            node: u32::MAX,
            motion: wr::BodyMotion::STATIC,
            shapes: shape,
            ..Default::default()
        };
        model.physics.bodies = put(&mut doc, &[body])?.cast();
        model.physics.next_id = 2;
        let id = doc
            .section_id(root.models.index)
            .ok_or_else(|| Error::new("wem.model", "missing model section"))?;
        doc.replace(id, &[model])?;
    }
    Ok(wem::Document::from_raw(doc)?)
}

pub(super) fn native_nodes(model: &sm::Model<'_>) -> Result<Vec<wr::Node<wr::V19>>, Error> {
    let Some(native) = &model.native else {
        return Ok(Vec::new());
    };
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let models = native
        .document
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?;
    let record = models
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "selected model absent"))??;
    Ok(native
        .document
        .raw()
        .resolve::<wr::Node<wr::V19>>(record.nodes.nodes.cast())?
        .iter()
        .collect::<Result<_, _>>()?)
}

pub(super) fn common_flags(native: u32) -> u32 {
    [
        (0x1, wr::NodeFlags::DONT_INHERIT_TRANSLATION.0),
        (0x2, wr::NodeFlags::DONT_INHERIT_SCALE.0),
        (0x4, wr::NodeFlags::DONT_INHERIT_ROTATION.0),
        (0x8, wr::NodeFlags::BILLBOARDED.0),
        (0x10, wr::NodeFlags::BILLBOARD_LOCK_X.0),
        (0x20, wr::NodeFlags::BILLBOARD_LOCK_Y.0),
        (0x40, wr::NodeFlags::BILLBOARD_LOCK_Z.0),
        (0x1000000, wr::NodeFlags::MODEL_SPACE.0),
    ]
    .into_iter()
    .filter(|(bit, _)| native & bit != 0)
    .fold(0, |flags, (_, bit)| flags | bit)
}
pub(super) fn native_flags(
    common: u32,
    original: u32,
    context: &mut Context,
    path: &str,
) -> Result<u32, Error> {
    let supported = wr::NodeFlags::DONT_INHERIT_TRANSLATION.0
        | wr::NodeFlags::DONT_INHERIT_ROTATION.0
        | wr::NodeFlags::DONT_INHERIT_SCALE.0
        | wr::NodeFlags::BILLBOARDED.0
        | wr::NodeFlags::BILLBOARD_LOCK_X.0
        | wr::NodeFlags::BILLBOARD_LOCK_Y.0
        | wr::NodeFlags::BILLBOARD_LOCK_Z.0
        | wr::NodeFlags::MODEL_SPACE.0;
    if common & !supported != 0 {
        context.unsupported(
            path,
            "M2 has no equivalent to the selected common node flags",
        )?;
    }
    let mapped = [
        (wr::NodeFlags::DONT_INHERIT_TRANSLATION.0, 0x1),
        (wr::NodeFlags::DONT_INHERIT_SCALE.0, 0x2),
        (wr::NodeFlags::DONT_INHERIT_ROTATION.0, 0x4),
        (wr::NodeFlags::BILLBOARDED.0, 0x8),
        (wr::NodeFlags::BILLBOARD_LOCK_X.0, 0x10),
        (wr::NodeFlags::BILLBOARD_LOCK_Y.0, 0x20),
        (wr::NodeFlags::BILLBOARD_LOCK_Z.0, 0x40),
        (wr::NodeFlags::MODEL_SPACE.0, 0x1000000),
    ]
    .into_iter()
    .filter(|(bit, _)| common & bit != 0)
    .fold(0, |flags, (_, bit)| flags | bit);
    Ok((original & !0x100007f) | mapped)
}
