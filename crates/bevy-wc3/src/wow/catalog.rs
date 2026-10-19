use bevy::math::Vec3;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use wc3::model::MaterialStep;
use wc3::model::{
    DrawId, ImageResource, InstanceId, Model, NodeFeature, NodeId, NodeProperties, SocketIdentity,
    Texture,
};

/// Stable native matching hints; semantic identities remain scoped to each model.
#[derive(Clone, Debug)]
pub struct BoneIdentity {
    /// Node in the inspected semantic model.
    pub node: NodeId,
    /// Native external name checksum; zero means absent.
    pub name_crc: u32,
    /// Native key-bone category; negative means absent.
    pub key_bone: i32,
    /// Authored name, useful for explicit application mappings.
    pub name: String,
    /// Native model-space bone pivot, retained for ambiguous or missing checksum matching.
    pub pivot: Vec3,
}
/// One draw classified by its native skin-section selection group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeosetDraw {
    /// Geometry instance, preserving its authored LOD membership.
    pub instance: InstanceId,
    /// Logical draw; all passes and LODs sharing a geoset are selected together.
    pub draw: DrawId,
    /// Native geoset identifier, without inferred category arithmetic.
    pub geoset: u16,
}
/// Build-independent semantic facts used by a character-data adapter.
#[derive(Clone, Debug, Default)]
pub struct CharacterCatalog {
    /// M2 nodes with their native bone matching hints.
    pub bones: Vec<BoneIdentity>,
    /// Native skin-section IDs from instances whose appearance uses M2 model shaders.
    pub geosets: Vec<GeosetDraw>,
    /// Native replacement texture category and its semantic texture identities.
    pub replacement_textures: BTreeMap<u32, Vec<Texture>>,
    /// Native attachment role and its carrying semantic node.
    pub attachments: BTreeMap<u32, Vec<NodeId>>,
}
impl CharacterCatalog {
    /// Inspects the default appearance; does not modify the semantic graph.
    pub fn inspect(model: &Model<'_>) -> Self {
        let bones = model
            .nodes()
            .iter()
            .filter_map(|(id, node)| match &node.properties {
                Some(NodeProperties::M2(controls)) => Some(BoneIdentity {
                    node: id,
                    name_crc: controls.name_crc,
                    key_bone: controls.key_bone,
                    name: node.name.to_string(),
                    pivot: node.transform.pivot,
                }),
                _ => None,
            })
            .collect();
        let mut geosets = Vec::new();
        for (instance_id, instance) in model.instances() {
            for draw in &instance.draws {
                if let Ok(material) = model.material_for_draw(instance_id, draw.id(), None) {
                    if let Some(material) = model.material(material) {
                        if material
                            .surface
                            .steps
                            .iter()
                            .any(|pass| matches!(pass, MaterialStep::M2(_)))
                        {
                            if let Ok(geoset) = u16::try_from(instance.selection_group) {
                                geosets.push(GeosetDraw {
                                    instance: instance_id,
                                    draw: draw.id(),
                                    geoset,
                                });
                            }
                        }
                    }
                }
            }
        }
        let mut replacement_textures: BTreeMap<u32, Vec<Texture>> = BTreeMap::new();
        for texture in model.sampled_images() {
            let id = texture;
            if let ImageResource::Replaceable { id: category, .. } = model
                .image_resource(texture.resource)
                .expect("validated image resource")
            {
                replacement_textures.entry(*category).or_default().push(id);
            }
        }
        let mut attachments: BTreeMap<u32, Vec<NodeId>> = BTreeMap::new();
        for (_, feature) in model.features() {
            let point = match &feature.payload {
                NodeFeature::Attachment(value) => value.socket(),
                _ => None,
            };
            if let Some(point) = point {
                if let SocketIdentity::Role(role) = point.identity {
                    attachments.entry(role).or_default().push(feature.node);
                }
            }
        }
        Self {
            bones,
            geosets,
            replacement_textures,
            attachments,
        }
    }
    /// Matches requested child joints using nonzero name CRC first, then key-bone.
    ///
    /// A CRC that has no parent match can use its key category. Ambiguous or absent
    /// identities require matching CRC and model-space pivot within 0.00001 model
    /// units. Generated names and joint ordinals are never matching keys.
    /// Duplicate requested child nodes and parent aliases are rejected.
    pub fn match_joints(
        &self,
        child: &Self,
        required: &[NodeId],
    ) -> Result<Vec<(NodeId, NodeId)>, JointMatchError> {
        let mut seen = BTreeSet::new();
        let mut used = BTreeSet::new();
        let mut result = Vec::with_capacity(required.len());
        for node in required {
            if !seen.insert(*node) {
                return Err(JointMatchError::DuplicateChild(*node));
            }
            let bone = child
                .bones
                .iter()
                .find(|bone| bone.node == *node)
                .ok_or(JointMatchError::MissingChild(*node))?;
            let crc: Vec<_> = self
                .bones
                .iter()
                .filter(|parent| bone.name_crc != 0 && parent.name_crc == bone.name_crc)
                .collect();
            let matches: Vec<_> = if crc.is_empty() {
                self.bones
                    .iter()
                    .filter(|parent| bone.key_bone >= 0 && parent.key_bone == bone.key_bone)
                    .collect()
            } else {
                crc
            };
            let matches = if matches.len() == 1 {
                matches
            } else {
                let candidates: Vec<_> = if matches.is_empty() {
                    self.bones.iter().collect()
                } else {
                    matches
                };
                candidates
                    .into_iter()
                    .filter(|parent| {
                        parent.name_crc == bone.name_crc
                            && parent.pivot.abs_diff_eq(bone.pivot, 1e-5)
                    })
                    .collect()
            };
            let parent = match matches.as_slice() {
                [] => return Err(JointMatchError::Unmatched(*node)),
                [parent] => parent.node,
                _ => return Err(JointMatchError::Ambiguous(*node)),
            };
            if !used.insert(parent) {
                return Err(JointMatchError::AliasedParent(parent));
            }
            result.push((*node, parent));
        }
        Ok(result)
    }
}
/// Automatic matching fails explicitly rather than deforming armor incorrectly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointMatchError {
    /// Requested child node is absent from the M2 bone catalog.
    MissingChild(NodeId),
    /// Child node was requested more than once.
    DuplicateChild(NodeId),
    /// Neither native identity has a matching parent bone.
    Unmatched(NodeId),
    /// A native identity names several parent bones.
    Ambiguous(NodeId),
    /// Several child joints map to the same parent; use an explicit mapping if intentional.
    AliasedParent(NodeId),
}
impl Display for JointMatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "armor joint matching failed: {self:?}")
    }
}
impl Error for JointMatchError {}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;
