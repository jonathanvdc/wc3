//! Profile-specific scene effects with editable WEM payloads.
mod attachments;
mod cameras;
pub(in crate::model::conversion::m3) mod cloth;
pub(in crate::model::conversion::m3) mod collision;
mod emitter_fields;
mod lights;
mod links;
pub(in crate::model::conversion::m3) mod metadata;
mod particle;
pub(in crate::model::conversion::m3) mod physics;
mod projector;
mod ribbon;
use super::{native, nr, read_error, revision, Context, Error};
use crate::model::wem::{self, records as wr, semantic as sem};
use metadata::{animated, restore};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
const IDENTITY: [[f32; 4]; 4] = [
    [1., 0., 0., 0.],
    [0., 1., 0., 0.],
    [0., 0., 1., 0.],
    [0., 0., 0., 1.],
];
fn install(
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut Vec<wr::Node<wr::V19>>,
    bone: u32,
    payload: wr::NodePayload<wr::V19>,
    kind: wr::NodeKind,
    values: BTreeMap<String, i64>,
) -> Result<(), Error> {
    let owner = nodes
        .get(bone as usize)
        .ok_or_else(|| Error::new("m3.effect.bone", "effect references a missing bone"))?;
    let name = doc.string(owner.name)?.unwrap_or_default().to_owned();
    let mut values = values;
    values.insert("m3.effect.bone".into(), i64::from(bone));
    let name = metadata::text(doc, &format!("{name} effect {}", nodes.len()))?;
    nodes.push(wr::Node::<wr::V19> {
        name,
        parent: bone,
        kind,
        payload,
        local: wr::Transform {
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
            scale: [1.; 3],
        },
        profiles: 1 << wr::ProfileId::SC2.0,
        native: metadata::extend(doc, &wr::NativeBag::default(), &values)?,
        ..Default::default()
    });
    Ok(())
}
pub(super) fn attach<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
) -> Result<wem::Document<'static>, Error> {
    let mut doc = document.into_raw();
    let reference = wem::Ref::new(
        doc.root_reference().entries,
        doc.root_reference().index,
        doc.root_reference().flags,
    );
    let root = doc
        .resolve::<wr::DocumentRecord<wr::V1>>(reference)?
        .get(0)
        .ok_or_else(|| Error::new("wem.root", "missing root"))??;
    let mut model = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.models", "missing model"))??;
    let mut nodes = doc
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let (
        forces,
        warps,
        light_ref,
        camera_ref,
        attachment_ref,
        particle_ref,
        ribbon_ref,
        copy_ref,
        projection_ref,
    ) = native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| (
        r.forces,
        r.warps,
        r.lights,
        r.cameras,
        r.attachment_points,
        r.particle_emitters,
        r.ribbon_emitters,
        r.particle_emitter_copies,
        r.projections
    ));
    let source_doc = source.document();
    macro_rules! forces {
        ($v:ty) => {
            for row in source_doc
                .resolve::<nr::Force<$v>>(forces.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let mut values = BTreeMap::new();
                for (key, value) in [("m3.force.flags", i64::from(row.flags))] {
                    values.insert(key.to_owned(), value);
                }
                animated(&mut values, "force.strength", row.strength);
                animated(&mut values, "force.width", row.width);
                animated(&mut values, "force.height", row.height);
                animated(&mut values, "force.length", row.length);
                if row.force_type > 255 || row.force_shape > 255 {
                    return Err(Error::new(
                        "m3.force.kind",
                        "force kind or shape exceeds WEM enum width",
                    ));
                }
                let payload = wr::ForceFieldPayload::<wr::V19> {
                    kind: wr::ForceKind(row.force_type as u8),
                    volume: wr::ForceVolume(row.force_shape as u8),
                    falloff: wem::Bool::from(row.flags & 1 != 0),
                    height_gradient: wem::Bool::from(row.flags & 2 != 0),
                    unbounded: wem::Bool::from(row.flags & 4 != 0),
                    affects_particles: wem::Bool::from(false),
                    affects_bodies: wem::Bool::from(false),
                    channels: row.local_channels,
                    scope: row.unknown,
                    strength: row.strength.initial_value,
                    width: row.width.initial_value,
                    height: row.height.initial_value,
                    length: row.length.initial_value,
                    transform: IDENTITY,
                };
                install(
                    &mut doc,
                    &mut nodes,
                    row.bone_index,
                    wr::NodePayload::ForceField(Box::new(payload)),
                    wr::NodeKind::FORCE_FIELD,
                    values,
                )?;
            }
        };
    }
    if forces.entries != 0 {
        match revision(source_doc, forces).map_err(read_error)? {
            0 => forces!(nr::V0),
            1 => forces!(nr::V1),
            2 => forces!(nr::V2),
            _ => return Err(Error::new("m3.forces", "unsupported force revision")),
        }
    }
    for row in source_doc
        .resolve::<nr::Warp>(warps.cast())
        .map_err(read_error)?
        .iter()
    {
        let row = row.map_err(read_error)?;
        let mut values = BTreeMap::new();
        for (key, value) in [
            ("warp.radius", row.radius),
            ("warp.height", row.height),
            ("warp.strength", row.strength),
            ("warp.angular", row.angular),
            ("warp.axial", row.axial),
            ("warp.radial", row.radial),
        ] {
            animated(&mut values, key, value);
        }
        let payload = wr::VertexWarpPayload {
            type_: row.warp_type,
            reserved: row.unknown,
            radius: row.radius.initial_value,
            height: row.height.initial_value,
            strength: row.strength.initial_value,
            angular: row.angular.initial_value,
            axial: row.axial.initial_value,
            radial: row.radial.initial_value,
        };
        install(
            &mut doc,
            &mut nodes,
            row.bone_index,
            wr::NodePayload::VertexWarp(Box::new(payload)),
            wr::NodeKind::VERTEX_WARP,
            values,
        )?;
    }
    projector::attach(source_doc, projection_ref, &mut doc, &mut nodes)?;
    lights::attach(source_doc, light_ref, &mut doc, &mut nodes)?;
    cameras::attach(source_doc, camera_ref, &mut doc, &mut nodes)?;
    particle::attach(source_doc, particle_ref, copy_ref, &mut doc, &mut nodes)?;
    ribbon::attach(source_doc, ribbon_ref, &mut doc, &mut nodes)?;
    attachments::attach(source_doc, attachment_ref, &mut doc, &mut nodes)?;
    links::attach(&doc, &mut nodes)?;
    if !nodes.is_empty() {
        let id = doc
            .section_id(model.nodes.nodes.index)
            .ok_or_else(|| Error::new("wem.nodes", "missing node section"))?;
        doc.replace(id, &nodes)?;
        model.nodes.nodes = wem::Ref::new(nodes.len() as u32, id.index(), model.nodes.nodes.flags);
        let id = doc
            .section_id(root.models.index)
            .ok_or_else(|| Error::new("wem.models", "missing model section"))?;
        doc.replace(id, &[model])?;
    }
    Ok(wem::Document::from_raw(doc)?)
}
#[derive(Default)]
pub(super) struct Effects {
    pub(super) projections: Ref<nr::targets::Projector>,
    pub(super) particle_copies: Ref<nr::targets::ParticleEmitterCopy>,
    pub(super) particles: Ref<nr::targets::ParticleEmitter>,
    pub(super) ribbons: Ref<nr::targets::RibbonEmitter>,
    pub(super) physics_joints: Ref<nr::targets::PhysicsJoint>,
    pub(super) rigid_bodies: Ref<nr::targets::RigidBody>,
    pub(super) attachments: Ref<nr::targets::AttachmentPoint>,
    pub(super) cameras: Ref<nr::targets::Camera>,
    pub(super) lights: Ref<nr::targets::Light>,
    pub(super) forces: Ref<nr::targets::Force>,
    pub(super) warps: Ref<nr::targets::Warp>,
}
pub(super) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    context: &mut super::Context,
) -> Result<Effects, Error> {
    let Some(native) = &model.native else {
        return Ok(Effects::default());
    };
    let source = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model_row = source
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.models", "missing model"))??;
    let nodes = source
        .resolve::<wr::Node<wr::V19>>(model_row.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let map = super::hierarchy::mapping(model)?;
    let modes = super::animation::modes(model, context)?;
    let mut camera_rows = Vec::new();
    let mut light_rows = Vec::new();
    let mut projection_rows = Vec::new();
    let mut particle_rows = Vec::new();
    let mut ribbon_rows = Vec::new();
    let mut forces = Vec::new();
    let mut warps = Vec::new();
    for (wem_index, node) in nodes.into_iter().enumerate() {
        let index = map[wem_index] as usize;
        let mut values = metadata::read(source, &node.native)?;
        super::animation::reconcile_controls(&mut values, &modes);
        values.insert("m3.effect.wem_index".into(), wem_index as i64);
        match node.payload {
            wr::NodePayload::Helper(_) if values.contains_key("m3.projector.source_version") => {
                projection_rows.push((values, index as u32))
            }
            wr::NodePayload::Sc2ParticleEmitter(payload) => {
                particle_rows.push((*payload, values, index as u32))
            }
            wr::NodePayload::Sc2RibbonEmitter(payload) => {
                ribbon_rows.push((*payload, values, index as u32))
            }
            wr::NodePayload::ForceField(payload) => {
                if payload.transform != IDENTITY {
                    return Err(Error::new(
                        format!("nodes[{index}].force.transform"),
                        "M3 force transform requires a separate hierarchy node",
                    ));
                }
                if payload.affects_particles.get() || payload.affects_bodies.get() {
                    return Err(Error::new(
                        format!("nodes[{index}].force.affects"),
                        "M3 target-class flags have no verified source mapping",
                    ));
                }
                let known = u32::from(payload.falloff.get())
                    | u32::from(payload.height_gradient.get()) << 1
                    | u32::from(payload.unbounded.get()) << 2;
                forces.push(nr::Force::<nr::V2> {
                    bone_index: index as u32,
                    force_type: u32::from(payload.kind.0),
                    force_shape: u32::from(payload.volume.0),
                    flags: (values.get("m3.force.flags").copied().unwrap_or(0) as u32 & !7) | known,
                    unknown: payload.scope,
                    local_channels: payload.channels,
                    strength: restore(&values, "force.strength", payload.strength),
                    width: restore(&values, "force.width", payload.width),
                    height: restore(&values, "force.height", payload.height),
                    length: restore(&values, "force.length", payload.length),
                });
            }
            wr::NodePayload::VertexWarp(payload) => warps.push(nr::Warp {
                bone_index: index as u32,
                warp_type: payload.type_,
                unknown: payload.reserved,
                radius: restore(&values, "warp.radius", payload.radius),
                height: restore(&values, "warp.height", payload.height),
                strength: restore(&values, "warp.strength", payload.strength),
                angular: restore(&values, "warp.angular", payload.angular),
                axial: restore(&values, "warp.axial", payload.axial),
                radial: restore(&values, "warp.radial", payload.radial),
            }),
            wr::NodePayload::Camera(payload) => camera_rows.push(cameras::export(
                index,
                metadata::get_text(source, &node.native, "m3.camera.name")?
                    .unwrap_or(source.string(node.name)?.unwrap_or_default()),
                *payload,
                &values,
                doc,
            )?),
            wr::NodePayload::Light(payload) => {
                light_rows.push(lights::export(index, *payload, &values)?)
            }
            _ => {}
        }
    }
    links::export(&mut particle_rows, &ribbon_rows)?;
    let (particles, particle_copies) = particle::export(&particle_rows, source, doc, context)?;
    Ok(Effects {
        projections: projector::export(&projection_rows, doc)?,
        particles,
        particle_copies,
        ribbons: ribbon::export(&ribbon_rows, source, doc, context, &map)?,
        physics_joints: physics::export_joints(model, doc, &map, context)?,
        rigid_bodies: physics::export(model, doc, &map, context)?,
        attachments: attachments::export(model, doc, &map)?,
        cameras: super::append(doc, &camera_rows)?.cast(),
        lights: super::append(doc, &light_rows)?.cast(),
        forces: super::append(doc, &forces)?.cast(),
        warps: super::append(doc, &warps)?.cast(),
    })
}
