//! Hierarchy output selects a vocabulary preserving the authored declaration presence.
mod capabilities;
mod payloads;
use crate::model::conversion::wem::actors::common::Field;
use crate::model::conversion::wem::hierarchy::{
    base, poses,
    setup::{RigField, SkinField},
};
use crate::model::conversion::wem::{metadata, text};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::SectionId;
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{
    FeatureId, MaterialSlotId, MeshId, Model, ModelId, NodeId, TextureId,
};
use crate::model::wem::{self, records, RawDocument, Ref, Target};
use payloads::WritePayload;
use std::collections::BTreeMap;

pub(in crate::model::conversion::wem) struct Context<'a> {
    pub nodes: &'a BTreeMap<NodeId, u32>,
    pub meshes: &'a BTreeMap<MeshId, u32>,
    pub models: &'a BTreeMap<ModelId, u32>,
    pub features: &'a BTreeMap<FeatureId, u32>,
    pub textures: &'a BTreeMap<TextureId, u32>,
    pub slots: &'a BTreeMap<MaterialSlotId, u32>,
    pub sections: &'a BTreeMap<SectionId, u32>,
}
fn index<T: Ord>(map: &BTreeMap<T, u32>, key: &T) -> Result<u32, Error> {
    map.get(key)
        .copied()
        .ok_or_else(|| Error::new("wem.node.reference", "object has no destination identity"))
}
fn optional_ref<T, V: Field<Ref<T>>>(value: Ref<T>) -> Result<V, Error>
where
    T: Target,
{
    V::write(Some(value)).or_else(|error| {
        if value.entries == 0 {
            V::write(None)
        } else {
            Err(error)
        }
    })
}
fn rows<'view, 'data: 'view, D>(
    document: &mut RawDocument<'view>,
    model: &Model<'data, Wem>,
    context: &Context<'_>,
) -> Result<Ref<records::targets::Node>, Error>
where
    D: records::NodeDialect,
    D::Payload: WritePayload,
    D::PoseMatrices: Field<Ref<records::targets::Matrix44>>,
    D::Skin: SkinField,
    D::Rig: RigField,
    D::Profiles: Field<u32>,
    records::Node<D>: wem::Write,
{
    let mut output = Vec::with_capacity(model.nodes().len());
    let conventions = &model.controls().poses;
    let pose_ids: Vec<_> = conventions.iter().map(|(id, _)| id).collect();
    let mut ordered: Vec<_> = model.nodes().collect();
    ordered.sort_by_key(|(id, _)| context.nodes.get(id).copied());
    for (ordinal, (id, _)) in ordered.iter().enumerate() {
        if context.nodes.get(id).copied() != u32::try_from(ordinal).ok() {
            return Err(Error::new(
                "wem.node.order",
                "hierarchy identities must map to a complete contiguous destination table",
            ));
        }
    }
    for (id, node) in ordered {
        let (components, matrices) =
            poses::write_values(document, &node.controls.setup.poses, conventions)?;
        let feature = model.node_features(id).next().map(|(_, feature)| feature);
        let payload =
            D::Payload::write_payload(document, node.controls.bone.as_ref(), feature, context)?;
        let availability = node.controls.availability.bits();
        output.push(records::Node::<D> {
            name: text::write_bytes(document, node.name.as_bytes())?,
            parent: node
                .parent
                .map(|id| index(context.nodes, &id))
                .transpose()?
                .unwrap_or(u32::MAX),
            kind: base::kind(node.controls.declared_role),
            flags: base::flags(node)?,
            pivot: node.transform.pivot,
            local: records::Transform {
                translation: node.transform.translation,
                rotation: node.transform.rotation.components(),
                scale: node.transform.scale,
            },
            uniform_scale_only: node.controls.uniform_scale_only.into(),
            poses: components,
            pose_matrices: optional_ref::<_, D::PoseMatrices>(matrices)?,
            payload,
            native: metadata::write(document, &node.controls.metadata)?,
            skin: D::Skin::write_setup(
                document,
                node.controls.setup.skin.as_ref(),
                context.nodes,
                &pose_ids,
            )?,
            rig: D::Rig::write_setup(node.controls.setup.rig.as_ref(), context.nodes)?,
            profiles: D::Profiles::write(Some(availability)).or_else(|error| {
                if availability == 0 {
                    D::Profiles::write(None)
                } else {
                    Err(error)
                }
            })?,
        });
    }
    poses::put(document, &output)
}

pub(in crate::model::conversion::wem) fn supports(model: &Model<'_, Wem>) -> bool {
    macro_rules! candidates {($($dialect:ident),+)=>{false $(|| capabilities::supports::<records::$dialect>(model))+};}
    candidates!(
        V19, V18, V17, V16, V15, V14, V13, V12, V11, V10, V9, V8, V7, V6, V5, V4, V3, V2, V1
    )
}

pub(in crate::model::conversion::wem) fn write<'view, 'data: 'view>(
    document: &mut RawDocument<'view>,
    model: &Model<'data, Wem>,
    context: &Context<'_>,
) -> Result<Ref<records::targets::Node>, Error> {
    macro_rules! candidates {($($dialect:ident),+)=>{$(if capabilities::supports::<records::$dialect>(model){return rows::<records::$dialect>(document,model,context);})+};}
    candidates!(
        V19, V18, V17, V16, V15, V14, V13, V12, V11, V10, V9, V8, V7, V6, V5, V4, V3, V2, V1
    );
    Err(Error::new(
        "wem.nodes.capabilities",
        "no shared node vocabulary preserves the authored setup and actor parameter declarations",
    ))
}

#[cfg(test)]
mod tests;
