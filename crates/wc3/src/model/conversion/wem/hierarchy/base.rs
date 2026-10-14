//! Hierarchy flags and authored transforms map to one logical owner.
use super::super::{metadata, text};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{DeclaredRole, NodeControls, SupplementalNodeFlags};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{Billboard, Inheritance, Node, NodeId, Quaternion, Transform};
use crate::model::wem::{records, RawDocument};

pub(in crate::model::conversion::wem) fn read<'view, D: records::NodeDialect>(
    document: &'view RawDocument<'_>,
    source: &records::Node<D>,
    nodes: &[NodeId],
    mut controls: NodeControls<'view>,
) -> Result<Node<'view, Wem>, Error> {
    controls.declared_role = match source.kind {
        records::NodeKind::Helper => DeclaredRole::Helper,
        records::NodeKind::Bone => DeclaredRole::Bone,
        records::NodeKind::Attachment => DeclaredRole::Attachment,
        records::NodeKind::Light => DeclaredRole::Light,
        records::NodeKind::Camera => DeclaredRole::Camera,
        records::NodeKind::ParticleEmitter => DeclaredRole::Particle,
        records::NodeKind::RibbonEmitter => DeclaredRole::Ribbon,
        records::NodeKind::Event => DeclaredRole::Event,
        records::NodeKind::CollisionShape => DeclaredRole::Collision,
        records::NodeKind::Wc3ParticleEmitter1 => DeclaredRole::WarcraftParticle1,
        records::NodeKind::Wc3ParticleEmitter2 => DeclaredRole::WarcraftParticle2,
        records::NodeKind::Wc3RibbonEmitter => DeclaredRole::WarcraftRibbon,
        records::NodeKind::Sc2ParticleEmitter => DeclaredRole::StarCraftParticle,
        records::NodeKind::Sc2RibbonEmitter => DeclaredRole::StarCraftRibbon,
        records::NodeKind::Wc3CornEmitter => DeclaredRole::WarcraftPopcorn,
        records::NodeKind::M2ParticleEmitter => DeclaredRole::WarcraftOnlineParticle,
        records::NodeKind::ForceField => DeclaredRole::Force,
        records::NodeKind::VertexWarp => DeclaredRole::VertexWarp,
        records::NodeKind::Wc3FaceFx => DeclaredRole::WarcraftFaceAnimation,
        records::NodeKind::Unknown(code) => DeclaredRole::Unknown(code),
        records::NodeKind::Count => DeclaredRole::Unknown(source.kind.raw()),
    };
    controls.flags = SupplementalNodeFlags::from_bits_retain(source.flags.bits());
    controls.uniform_scale_only = source.uniform_scale_only.get();
    controls.metadata = metadata::read(document, &source.native)?;
    let flags = source.flags;
    let locked = flags.billboard_lock_x() || flags.billboard_lock_y() || flags.billboard_lock_z();
    Ok(Node {
        name: text::read(document, source.name)?,
        parent: if source.parent == u32::MAX {
            None
        } else {
            Some(*nodes.get(source.parent as usize).ok_or_else(|| {
                Error::new(
                    "wem.node.parent",
                    "parent is absent from the actual hierarchy",
                )
            })?)
        },
        transform: Transform {
            translation: source.local.translation,
            rotation: Quaternion(source.local.rotation),
            scale: source.local.scale,
            pivot: source.pivot,
        },
        inheritance: Inheritance {
            translation: !flags.dont_inherit_translation(),
            rotation: !flags.dont_inherit_rotation(),
            scale: !flags.dont_inherit_scale(),
        },
        billboard: if locked {
            Billboard::Locked {
                enabled: flags.billboarded(),
                x: flags.billboard_lock_x(),
                y: flags.billboard_lock_y(),
                z: flags.billboard_lock_z(),
            }
        } else if flags.billboarded() {
            Billboard::Spherical
        } else {
            Billboard::None
        },
        visible: !flags.hidden(),
        camera_anchored: false,
        controls,
    })
}

pub(in crate::model::conversion::wem) fn kind(role: DeclaredRole) -> records::NodeKind {
    match role {
        DeclaredRole::Helper => records::NodeKind::Helper,
        DeclaredRole::Bone => records::NodeKind::Bone,
        DeclaredRole::Attachment => records::NodeKind::Attachment,
        DeclaredRole::Light => records::NodeKind::Light,
        DeclaredRole::Camera => records::NodeKind::Camera,
        DeclaredRole::Particle => records::NodeKind::ParticleEmitter,
        DeclaredRole::Ribbon => records::NodeKind::RibbonEmitter,
        DeclaredRole::Event => records::NodeKind::Event,
        DeclaredRole::Collision => records::NodeKind::CollisionShape,
        DeclaredRole::WarcraftParticle1 => records::NodeKind::Wc3ParticleEmitter1,
        DeclaredRole::WarcraftParticle2 => records::NodeKind::Wc3ParticleEmitter2,
        DeclaredRole::WarcraftRibbon => records::NodeKind::Wc3RibbonEmitter,
        DeclaredRole::StarCraftParticle => records::NodeKind::Sc2ParticleEmitter,
        DeclaredRole::StarCraftRibbon => records::NodeKind::Sc2RibbonEmitter,
        DeclaredRole::WarcraftPopcorn => records::NodeKind::Wc3CornEmitter,
        DeclaredRole::WarcraftOnlineParticle => records::NodeKind::M2ParticleEmitter,
        DeclaredRole::Force => records::NodeKind::ForceField,
        DeclaredRole::VertexWarp => records::NodeKind::VertexWarp,
        DeclaredRole::WarcraftFaceAnimation => records::NodeKind::Wc3FaceFx,
        DeclaredRole::Unknown(code) => records::NodeKind::from_raw(code),
    }
}
pub(in crate::model::conversion::wem) fn flags(
    node: &Node<'_, Wem>,
) -> Result<records::NodeFlags, Error> {
    if node.camera_anchored {
        return Err(Error::new(
            "wem.node.camera_anchored",
            "the authoring hierarchy has no camera-origin anchoring declaration",
        ));
    }
    let mut flags = records::NodeFlags::from_bits_retain(node.controls.flags.bits());
    flags.set_hidden(!node.visible);
    flags.set_dont_inherit_translation(!node.inheritance.translation);
    flags.set_dont_inherit_rotation(!node.inheritance.rotation);
    flags.set_dont_inherit_scale(!node.inheritance.scale);
    match node.billboard {
        Billboard::None => {}
        Billboard::Spherical => flags.set_billboarded(true),
        Billboard::Locked { enabled, x, y, z } => {
            flags.set_billboarded(enabled);
            flags.set_billboard_lock_x(x);
            flags.set_billboard_lock_y(y);
            flags.set_billboard_lock_z(z);
        }
    }
    Ok(flags)
}
