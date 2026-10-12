//! Native MODL revision dispatch and common metadata inspection.
use super::records::*;
use super::vertices::DynamicVertices;
use super::{Envelope, Model, ReadError, Ref};
use crate::visit_m3_root;
/// A native model root retaining its independently selected section revision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RootRecord {
    /// MODL revision 20, without conversion to another record family.
    V20(ModelRecord<V20>),
    /// MODL revision 21, without conversion to another record family.
    V21(ModelRecord<V21>),
    /// MODL revision 22, without conversion to another record family.
    V22(ModelRecord<V22>),
    /// MODL revision 23, without conversion to another record family.
    V23(ModelRecord<V23>),
    /// MODL revision 24, without conversion to another record family.
    V24(ModelRecord<V24>),
    /// MODL revision 25, without conversion to another record family.
    V25(ModelRecord<V25>),
    /// MODL revision 26, without conversion to another record family.
    V26(ModelRecord<V26>),
    /// MODL revision 27, without conversion to another record family.
    V27(ModelRecord<V27>),
    /// MODL revision 28, without conversion to another record family.
    V28(ModelRecord<V28>),
    /// MODL revision 29, without conversion to another record family.
    V29(ModelRecord<V29>),
    /// MODL revision 30, without conversion to another record family.
    V30(ModelRecord<V30>),
}
impl RootRecord {
    /// Native MODL section revision.
    pub fn revision(&self) -> u32 {
        match self {
            Self::V20(_) => 20,
            Self::V21(_) => 21,
            Self::V22(_) => 22,
            Self::V23(_) => 23,
            Self::V24(_) => 24,
            Self::V25(_) => 25,
            Self::V26(_) => 26,
            Self::V27(_) => 27,
            Self::V28(_) => 28,
            Self::V29(_) => 29,
            Self::V30(_) => 30,
        }
    }
    /// Model filename CHAR reference.
    pub fn name(&self) -> Ref<targets::Character> {
        visit_m3_root!(self, |root| root.name)
    }
    /// Native model flags without rendering interpretation.
    pub fn flags(&self) -> u32 {
        visit_m3_root!(self, |root| root.flags)
    }
    /// Packed vertex-format flags owned by MODL.
    pub fn vertex_flags(&self) -> u32 {
        visit_m3_root!(self, |root| root.vertex_flags)
    }
    /// Reference to packed vertex bytes, retaining native index and flag words.
    pub fn vertices(&self) -> Ref<targets::Byte> {
        visit_m3_root!(self, |root| root.vertices)
    }
    /// Native skeleton reference, independent of the BONE section revision.
    pub fn bones(&self) -> Ref<targets::Bone> {
        visit_m3_root!(self, |root| root.bones)
    }
    /// Sequence table reference, independent of SEQS revision.
    pub fn sequences(&self) -> Ref<targets::Sequence> {
        visit_m3_root!(self, |root| root.sequences)
    }
    /// Single M3A binding hash, or zero in a layout without that field.
    pub fn animation_hash(&self) -> u32 {
        match self {
            Self::V20(_) => 0,
            Self::V21(root) => root.m3a_anim_hash,
            Self::V22(root) => root.m3a_anim_hash,
            Self::V23(root) => root.m3a_anim_hash,
            Self::V24(root) => root.m3a_anim_hash,
            Self::V25(root) => root.m3a_anim_hash,
            Self::V26(root) => root.m3a_anim_hash,
            Self::V27(root) => root.m3a_anim_hash,
            Self::V28(root) => root.m3a_anim_hash,
            Self::V29(root) => root.m3a_anim_hash,
            Self::V30(root) => root.m3a_anim_hash,
        }
    }
    /// Additional M3A hash table reference, absent before its native field was introduced.
    pub fn additional_animation_hashes(&self) -> Option<Ref<targets::U32>> {
        match self {
            Self::V20(_) | Self::V21(_) | Self::V22(_) => None,
            Self::V23(root) => Some(root.m3a_anim_hashes),
            Self::V24(root) => Some(root.m3a_anim_hashes),
            Self::V25(root) => Some(root.m3a_anim_hashes),
            Self::V26(root) => Some(root.m3a_anim_hashes),
            Self::V27(root) => Some(root.m3a_anim_hashes),
            Self::V28(root) => Some(root.m3a_anim_hashes),
            Self::V29(root) => Some(root.m3a_anim_hashes),
            Self::V30(root) => Some(root.m3a_anim_hashes),
        }
    }
}
impl<E: Envelope> Model<'_, E> {
    /// Decodes the root through its exact native revision, without an upgrade.
    pub fn root_record(&self) -> Result<RootRecord, ReadError> {
        let revision = self
            .document()
            .section(self.root_id())
            .ok_or_else(|| ReadError::invalid(0, "model root identity"))?
            .revision();
        Ok(match revision {
            20 => RootRecord::V20(self.root::<ModelRecord<V20>>()?),
            21 => RootRecord::V21(self.root::<ModelRecord<V21>>()?),
            22 => RootRecord::V22(self.root::<ModelRecord<V22>>()?),
            23 => RootRecord::V23(self.root::<ModelRecord<V23>>()?),
            24 => RootRecord::V24(self.root::<ModelRecord<V24>>()?),
            25 => RootRecord::V25(self.root::<ModelRecord<V25>>()?),
            26 => RootRecord::V26(self.root::<ModelRecord<V26>>()?),
            27 => RootRecord::V27(self.root::<ModelRecord<V27>>()?),
            28 => RootRecord::V28(self.root::<ModelRecord<V28>>()?),
            29 => RootRecord::V29(self.root::<ModelRecord<V29>>()?),
            30 => RootRecord::V30(self.root::<ModelRecord<V30>>()?),
            _ => return Err(ReadError::invalid(0, "unsupported MODL revision")),
        })
    }
    /// Enumerates nonzero native M3A hashes in declared order, retaining duplicate declarations.
    pub fn animation_hashes(&self) -> Result<Vec<u32>, ReadError> {
        let root = self.root_record()?;
        let mut hashes = Vec::new();
        let primary = root.animation_hash();
        if primary != 0 {
            hashes.push(primary);
        }
        if let Some(reference) = root.additional_animation_hashes() {
            for value in self.document().resolve::<U32>(reference.cast())?.iter() {
                let value = value?.value;
                if value != 0 {
                    hashes.push(value);
                }
            }
        }
        Ok(hashes)
    }
    /// Opens a borrowed packed vertex view, or a Raw view when flag interpretation is unresolved.
    pub fn vertices(&self) -> Result<DynamicVertices<&[u8]>, ReadError> {
        let root = self.root_record()?;
        let array = self.document().resolve::<Byte>(root.vertices().cast())?;
        // Resolving validates the reference; the section slice itself supplies a stable borrow.
        let bytes = if root.vertices().entries == 0 {
            &[][..]
        } else {
            let section = self
                .document()
                .sections()
                .find(|(id, _)| id.index() == root.vertices().index)
                .map(|(_, section)| section)
                .ok_or_else(|| ReadError::invalid(0, "vertex section identity"))?;
            &section.bytes()[..array.len()]
        };
        DynamicVertices::from_flags(bytes, root.vertex_flags())
    }
}
/// Dispatches an operation over a concrete native MODL revision.
#[macro_export]
macro_rules! visit_m3_root {
    ($root:expr,|$typed:ident|$body:expr) => {
        match $root {
            $crate::model::m3::RootRecord::V20($typed) => $body,
            $crate::model::m3::RootRecord::V21($typed) => $body,
            $crate::model::m3::RootRecord::V22($typed) => $body,
            $crate::model::m3::RootRecord::V23($typed) => $body,
            $crate::model::m3::RootRecord::V24($typed) => $body,
            $crate::model::m3::RootRecord::V25($typed) => $body,
            $crate::model::m3::RootRecord::V26($typed) => $body,
            $crate::model::m3::RootRecord::V27($typed) => $body,
            $crate::model::m3::RootRecord::V28($typed) => $body,
            $crate::model::m3::RootRecord::V29($typed) => $body,
            $crate::model::m3::RootRecord::V30($typed) => $body,
        }
    };
}
