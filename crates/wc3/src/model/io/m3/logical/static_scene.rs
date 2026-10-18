//! Named attachments, billboard behavior, and animated spatial bounds.
use super::{
    animation::AnimationIds,
    bindings::{self, Bindings},
    hierarchy::Skeleton,
    materials::{anim, read_error, revision, string, Slot},
    write::{animated, append},
    Graph, GraphBuilder,
};
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::{profiles::m3 as surface, NodeId};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
fn node(index: u32, nodes: &[NodeId]) -> Result<NodeId, Error> {
    nodes
        .get(index as usize)
        .copied()
        .ok_or_else(|| Error::new("m3.scene.anchor", "anchor exceeds skeleton"))
}
fn index(node: NodeId, bones: &BTreeMap<NodeId, u32>) -> Result<u32, Error> {
    bones
        .get(&node)
        .copied()
        .ok_or_else(|| Error::new("m3.scene.anchor", "anchor has no native bone"))
}
fn small(index: u32) -> Result<u16, Error> {
    u16::try_from(index)
        .map_err(|_| Error::new("m3.scene.anchor", "native anchor capacity exceeded"))
}
pub(super) fn read<'a, E: Envelope>(
    source: &'a native::Document<'_, E>,
    builder: &mut GraphBuilder<'a>,
    skeleton: &Skeleton<'a>,
    registry: &mut Bindings,
) -> Result<(), Error> {
    let root = source.root_record().map_err(read_error)?;
    let (attachments, billboards, shadow, volumes) = native::visit_m3_root!(root, |r| (
        r.attachment_points,
        r.billboard_behaviors,
        r.shadow_boxes,
        Slot::<Ref<nr::targets::ViewVolume>>::value(&r.view_volumes)
    ));
    let document = source.container();
    macro_rules! attachments {
        ($revision:ty) => {
            for row in document
                .resolve::<nr::AttachmentPoint<$revision>>(attachments.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                builder.add_feature(
                    node(r.bone_index, &skeleton.nodes)?,
                    surface::Feature::Attachment(
                        (surface::Attachment {
                            identity: string(document, r.name)?,
                            visibility: (),
                            control: r.unknown,
                        })
                        .into(),
                    ),
                )?;
            }
        };
    }
    if !attachments.is_empty() {
        match revision(document, attachments).map_err(read_error)? {
            0 => attachments!(nr::V0),
            1 => attachments!(nr::V1),
            _ => return Err(Error::new("m3.attachments", "unknown attachment schema")),
        }
    }
    for row in document
        .resolve::<nr::BillboardBehavior>(billboards.cast())
        .map_err(read_error)?
        .iter()
    {
        let r = row.map_err(read_error)?;
        let mut dependents = Vec::new();
        for row in document
            .resolve::<nr::U16>(r.dependents.cast())
            .map_err(read_error)?
            .iter()
        {
            dependents.push(node(
                u32::from(row.map_err(read_error)?.value),
                &skeleton.nodes,
            )?);
        }
        builder.add_feature(
            node(u32::from(r.bone_index), &skeleton.nodes)?,
            surface::Feature::Billboard(
                (surface::BillboardBehavior {
                    dependents,
                    billboard_type: surface::values::BillboardKind::from_raw(
                        r.billboard_type.raw(),
                    ),
                    camera_look_at: r.camera_look_at,
                    up: r.up,
                    forward: r.forward,
                })
                .into(),
            ),
        )?;
    }
    for row in document
        .resolve::<nr::ShadowBox>(shadow.cast())
        .map_err(read_error)?
        .iter()
    {
        let r = row.map_err(read_error)?;
        let owner = builder.reserve_feature(node(u32::from(r.bone_index), &skeleton.nodes)?);
        let mut payload = surface::ShadowBox {
            reserved: r.padding,
            length: anim(r.length, builder.values_mut()),
            width: anim(r.width, builder.values_mut()),
            height: anim(r.height, builder.values_mut()),
        };
        bindings::field(&mut payload.length, registry);
        bindings::field(&mut payload.width, registry);
        bindings::field(&mut payload.height, registry);
        builder.set_feature(owner, surface::Feature::ShadowBox((payload).into()))?;
    }
    for row in document
        .resolve::<nr::ViewVolume>(volumes.cast())
        .map_err(read_error)?
        .iter()
    {
        let r = row.map_err(read_error)?;
        let owner = builder.reserve_feature(node(r.node_index, &skeleton.nodes)?);
        let mut payload = surface::ViewVolume {
            size: anim(r.size, builder.values_mut()),
        };
        bindings::field(&mut payload.size, registry);
        builder.set_feature(owner, surface::Feature::ViewVolume((payload).into()))?;
    }
    Ok(())
}
#[derive(Default)]
pub(super) struct Scene {
    pub attachments: Ref<nr::targets::AttachmentPoint>,
    pub billboards: Ref<nr::targets::BillboardBehavior>,
    pub shadow: Ref<nr::targets::ShadowBox>,
    pub volumes: Ref<nr::targets::ViewVolume>,
}
pub(super) fn write<E: Envelope>(
    model: &Graph<'_>,
    document: &mut native::Container<'_, E>,
    bones: &BTreeMap<NodeId, u32>,
    ids: &AnimationIds,
) -> Result<Scene, Error> {
    let mut attachments = Vec::new();
    let mut billboards = Vec::new();
    let mut shadow = Vec::new();
    let mut volumes = Vec::new();
    for (_, feature) in model.features() {
        match &feature.payload {
            surface::Feature::Attachment(r) => attachments.push(nr::AttachmentPoint::<nr::V1> {
                unknown: r.control,
                name: super::write::string(document, &r.identity)?,
                bone_index: index(feature.node, bones)?,
            }),
            surface::Feature::Billboard(r) => {
                let dependents = r
                    .dependents
                    .iter()
                    .map(|node| {
                        Ok(nr::U16 {
                            value: small(index(*node, bones)?)?,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                billboards.push(nr::BillboardBehavior {
                    dependents: append(document, &dependents)?.cast(),
                    bone_index: small(index(feature.node, bones)?)?,
                    billboard_type: nr::BillboardKind::from_raw(r.billboard_type.raw()),
                    camera_look_at: r.camera_look_at,
                    up: r.up,
                    forward: r.forward,
                });
            }
            surface::Feature::ShadowBox(r) => shadow.push(nr::ShadowBox {
                bone_index: small(index(feature.node, bones)?)?,
                padding: r.reserved,
                length: animated(&r.length, ids),
                width: animated(&r.width, ids),
                height: animated(&r.height, ids),
            }),
            surface::Feature::ViewVolume(r) => volumes.push(nr::ViewVolume {
                node_index: index(feature.node, bones)?,
                size: animated(&r.size, ids),
            }),
            _ => {}
        }
    }
    Ok(Scene {
        attachments: append(document, &attachments)?.cast(),
        billboards: append(document, &billboards)?.cast(),
        shadow: append(document, &shadow)?.cast(),
        volumes: append(document, &volumes)?.cast(),
    })
}
