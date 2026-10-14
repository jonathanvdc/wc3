//! Selection inspects typed field support before writing any sections.
use super::WritePayload;
use crate::model::conversion::wem::actors::common::Field;
use crate::model::conversion::wem::hierarchy::setup::{RigField, SkinField};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::Model;
use crate::model::wem::{records, Ref};
pub(super) fn supports<D: records::NodeDialect>(model: &Model<'_, Wem>) -> bool
where
    D::Payload: WritePayload,
    D::PoseMatrices: Field<Ref<records::targets::Matrix44>>,
    D::Skin: SkinField,
    D::Rig: RigField,
    D::Profiles: Field<u32>,
{
    model.nodes().all(|(id, node)| {
        let matrices = node
            .controls
            .setup
            .poses
            .iter()
            .any(|pose| pose.matrix.is_some());
        let availability = node.controls.availability.bits();
        let payloads: Vec<_> = model.node_features(id).collect();
        payloads.len() + usize::from(node.controls.bone.is_some()) <= 1
            && D::Skin::supports(node.controls.setup.skin.as_ref())
            && D::Rig::supports(node.controls.setup.rig.as_ref())
            && (!matrices || D::PoseMatrices::write(Some(Ref::default())).is_ok())
            && (availability == 0 || D::Profiles::write(Some(availability)).is_ok())
            && D::Payload::supports(
                node.controls.bone.as_ref(),
                payloads.first().map(|(_, feature)| *feature),
            )
    })
}
