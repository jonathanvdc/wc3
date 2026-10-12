//! Runtime dispatch retaining a complete, statically typed companion document.
use super::super as m2;
use super::super::codec::Encoder;
use super::super::{
    ModelLayout, ReadError, V272PreSplit, WriteError, V256, V257, V260, V263, V264, V265, V272,
    V273, V274,
};
use super::{Skeleton, Skin, SupportsExternalSkins};
use crate::model::common::Ownership;

/// Exact native layout identity, including distinct interpretations of version 272.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutKind {
    /// Native V256 record family.
    V256,
    /// Native V257 record family.
    V257,
    /// Native V260 record family.
    V260,
    /// Native V263 record family.
    V263,
    /// Native V264 record family.
    V264,
    /// Native V265 record family.
    V265,
    /// Native V272 record family.
    V272,
    /// Native V272PreSplit record family.
    V272PreSplit,
    /// Native V273 record family.
    V273,
    /// Native V274 record family.
    V274,
}
/// Companion dispatch supplied by each sealed model-layout marker.
pub trait CompanionLayout: Sized {
    /// Exact record family selected by this marker.
    const KIND: LayoutKind;
    /// Wraps this skeleton while preserving its typed fields.
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data>
    where
        Self: ModelLayout;
    /// Borrows a standalone skin only when its native marker matches.
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>>
    where
        Self: ModelLayout;
    /// Mutably borrows a standalone skin only when its native marker matches.
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>>
    where
        Self: ModelLayout;
    /// Borrows a skeleton only when its native marker matches.
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>>
    where
        Self: ModelLayout;
    /// Mutably borrows a skeleton only when its native marker matches.
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>>
    where
        Self: ModelLayout;
}

/// A standalone skin with its owner's native record layout retained.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum DynamicSkin<'data> {
    /// Standalone profile in the V264 layout.
    V264(Skin<'data, V264>),
    /// Standalone profile in the V265 layout.
    V265(Skin<'data, V265>),
    /// Standalone profile in the V272 layout.
    V272(Skin<'data, V272>),
    /// Standalone profile in the V272PreSplit layout.
    V272PreSplit(Skin<'data, V272PreSplit>),
    /// Standalone profile in the V273 layout.
    V273(Skin<'data, V273>),
    /// Standalone profile in the V274 layout.
    V274(Skin<'data, V274>),
}
/// A shared skeleton with its owner's native record layout retained.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum DynamicSkeleton<'data> {
    /// Skeleton records in the V256 layout.
    V256(Skeleton<'data, V256>),
    /// Skeleton records in the V257 layout.
    V257(Skeleton<'data, V257>),
    /// Skeleton records in the V260 layout.
    V260(Skeleton<'data, V260>),
    /// Skeleton records in the V263 layout.
    V263(Skeleton<'data, V263>),
    /// Skeleton records in the V264 layout.
    V264(Skeleton<'data, V264>),
    /// Skeleton records in the V265 layout.
    V265(Skeleton<'data, V265>),
    /// Skeleton records in the V272 layout.
    V272(Skeleton<'data, V272>),
    /// Skeleton records in the V272PreSplit layout.
    V272PreSplit(Skeleton<'data, V272PreSplit>),
    /// Skeleton records in the V273 layout.
    V273(Skeleton<'data, V273>),
    /// Skeleton records in the V274 layout.
    V274(Skeleton<'data, V274>),
}
impl CompanionLayout for V256 {
    const KIND: LayoutKind = LayoutKind::V256;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V256(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        let _ = file;
        None
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        let _ = file;
        None
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V256(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V256(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V257 {
    const KIND: LayoutKind = LayoutKind::V257;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V257(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        let _ = file;
        None
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        let _ = file;
        None
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V257(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V257(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V260 {
    const KIND: LayoutKind = LayoutKind::V260;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V260(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        let _ = file;
        None
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        let _ = file;
        None
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V260(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V260(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V263 {
    const KIND: LayoutKind = LayoutKind::V263;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V263(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        let _ = file;
        None
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        let _ = file;
        None
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V263(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V263(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V264 {
    const KIND: LayoutKind = LayoutKind::V264;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V264(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        if let DynamicSkin::V264(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        if let DynamicSkin::V264(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V264(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V264(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V265 {
    const KIND: LayoutKind = LayoutKind::V265;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V265(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        if let DynamicSkin::V265(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        if let DynamicSkin::V265(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V265(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V265(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V272 {
    const KIND: LayoutKind = LayoutKind::V272;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V272(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        if let DynamicSkin::V272(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        if let DynamicSkin::V272(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V272(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V272(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V272PreSplit {
    const KIND: LayoutKind = LayoutKind::V272PreSplit;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V272PreSplit(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        if let DynamicSkin::V272PreSplit(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        if let DynamicSkin::V272PreSplit(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V272PreSplit(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V272PreSplit(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V273 {
    const KIND: LayoutKind = LayoutKind::V273;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V273(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        if let DynamicSkin::V273(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        if let DynamicSkin::V273(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V273(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V273(file) = file {
            Some(file)
        } else {
            None
        }
    }
}
impl CompanionLayout for V274 {
    const KIND: LayoutKind = LayoutKind::V274;
    fn into_dynamic_skeleton<'data>(file: Skeleton<'data, Self>) -> DynamicSkeleton<'data> {
        DynamicSkeleton::V274(file)
    }
    fn skin_ref<'view, 'data>(file: &'view DynamicSkin<'data>) -> Option<&'view Skin<'data, Self>> {
        if let DynamicSkin::V274(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skin_mut<'view, 'data>(
        file: &'view mut DynamicSkin<'data>,
    ) -> Option<&'view mut Skin<'data, Self>> {
        if let DynamicSkin::V274(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_ref<'view, 'data>(
        file: &'view DynamicSkeleton<'data>,
    ) -> Option<&'view Skeleton<'data, Self>> {
        if let DynamicSkeleton::V274(file) = file {
            Some(file)
        } else {
            None
        }
    }
    fn skeleton_mut<'view, 'data>(
        file: &'view mut DynamicSkeleton<'data>,
    ) -> Option<&'view mut Skeleton<'data, Self>> {
        if let DynamicSkeleton::V274(file) = file {
            Some(file)
        } else {
            None
        }
    }
}

/// Dispatches a companion while retaining its concrete native layout.
#[macro_export]
macro_rules! visit_m2_skin {
    ($file:expr, |$binding:ident| $body:expr) => {
        match $file {
            $crate::model::m2::companions::DynamicSkin::V264($binding) => $body,
            $crate::model::m2::companions::DynamicSkin::V265($binding) => $body,
            $crate::model::m2::companions::DynamicSkin::V272($binding) => $body,
            $crate::model::m2::companions::DynamicSkin::V272PreSplit($binding) => $body,
            $crate::model::m2::companions::DynamicSkin::V273($binding) => $body,
            $crate::model::m2::companions::DynamicSkin::V274($binding) => $body,
        }
    };
}

/// Dispatches a companion while retaining its concrete native layout.
#[macro_export]
macro_rules! visit_m2_skeleton {
    ($file:expr, |$binding:ident| $body:expr) => {
        match $file {
            $crate::model::m2::companions::DynamicSkeleton::V256($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V257($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V260($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V263($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V264($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V265($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V272($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V272PreSplit($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V273($binding) => $body,
            $crate::model::m2::companions::DynamicSkeleton::V274($binding) => $body,
        }
    };
}

impl<'data> DynamicSkin<'data> {
    /// Wraps a statically selected standalone profile.
    pub fn from_typed<V: SupportsExternalSkins>(file: Skin<'data, V>) -> Self {
        V::into_dynamic_skin(file)
    }
    /// Borrows the native profile when the requested layout matches.
    pub fn as_typed<V: ModelLayout>(&self) -> Option<&Skin<'data, V>> {
        V::skin_ref(self)
    }
    /// Mutably borrows the native profile when the requested layout matches.
    pub fn as_typed_mut<V: ModelLayout>(&mut self) -> Option<&mut Skin<'data, V>> {
        V::skin_mut(self)
    }
    /// Returns the exact marker and profile selected by the owner.
    pub fn layout(&self) -> LayoutKind {
        visit_m2_skin!(self, |file| skin_layout(file))
    }
    /// Rebuilds a profile using its own native fields and header width.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        visit_m2_skin!(self, |file| file.write())
    }
}
fn skin_layout<V: ModelLayout>(_: &Skin<'_, V>) -> LayoutKind {
    V::KIND
}
impl<'data> DynamicSkeleton<'data> {
    /// Wraps a statically selected skeleton.
    pub fn from_typed<V: ModelLayout>(file: Skeleton<'data, V>) -> Self {
        V::into_dynamic_skeleton(file)
    }
    /// Borrows the native skeleton when the requested layout matches.
    pub fn as_typed<V: ModelLayout>(&self) -> Option<&Skeleton<'data, V>> {
        V::skeleton_ref(self)
    }
    /// Mutably borrows the native skeleton when the requested layout matches.
    pub fn as_typed_mut<V: ModelLayout>(&mut self) -> Option<&mut Skeleton<'data, V>> {
        V::skeleton_mut(self)
    }
    /// Returns the exact marker and profile selected by the owner.
    pub fn layout(&self) -> LayoutKind {
        visit_m2_skeleton!(self, |file| skeleton_layout(file))
    }
    /// Context flags supplied by the owning model.
    pub fn flags(&self) -> u32 {
        visit_m2_skeleton!(self, |file| file.flags)
    }
    /// Rebuilds the skeleton using its native record family.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        visit_m2_skeleton!(self, |file| file.write())
    }
}
fn skeleton_layout<V: ModelLayout>(_: &Skeleton<'_, V>) -> LayoutKind {
    V::KIND
}
impl LayoutKind {
    /// Numeric identity stored in a model's core header.
    pub fn version(self) -> u32 {
        match self {
            Self::V256 => 256,
            Self::V257 => 257,
            Self::V260 => 260,
            Self::V263 => 263,
            Self::V264 => 264,
            Self::V265 => 265,
            Self::V272 => 272,
            Self::V272PreSplit => 272,
            Self::V273 => 273,
            Self::V274 => 274,
        }
    }
    /// Decodes a standalone profile in the owner's explicitly selected layout.
    pub fn read_skin(self, data: &[u8]) -> Result<DynamicSkin<'_>, ReadError> {
        match self {
            Self::V264 => Ok(DynamicSkin::V264(Skin::<V264>::read(data)?)),
            Self::V265 => Ok(DynamicSkin::V265(Skin::<V265>::read(data)?)),
            Self::V272 => Ok(DynamicSkin::V272(Skin::<V272>::read(data)?)),
            Self::V272PreSplit => Ok(DynamicSkin::V272PreSplit(Skin::<V272PreSplit>::read(data)?)),
            Self::V273 => Ok(DynamicSkin::V273(Skin::<V273>::read(data)?)),
            Self::V274 => Ok(DynamicSkin::V274(Skin::<V274>::read(data)?)),
            _ => Err(ReadError::invalid(
                0,
                "standalone skins unavailable in owner layout",
            )),
        }
    }
    /// Decodes a skeleton with inherited sequence-storage context.
    pub fn read_skeleton<'data>(
        self,
        data: &'data [u8],
        flags: u32,
        external: &[bool],
    ) -> Result<DynamicSkeleton<'data>, ReadError> {
        match self {
            Self::V256 => Ok(DynamicSkeleton::V256(
                Skeleton::<V256>::read_with_sequences(data, flags, external)?,
            )),
            Self::V257 => Ok(DynamicSkeleton::V257(
                Skeleton::<V257>::read_with_sequences(data, flags, external)?,
            )),
            Self::V260 => Ok(DynamicSkeleton::V260(
                Skeleton::<V260>::read_with_sequences(data, flags, external)?,
            )),
            Self::V263 => Ok(DynamicSkeleton::V263(
                Skeleton::<V263>::read_with_sequences(data, flags, external)?,
            )),
            Self::V264 => Ok(DynamicSkeleton::V264(
                Skeleton::<V264>::read_with_sequences(data, flags, external)?,
            )),
            Self::V265 => Ok(DynamicSkeleton::V265(
                Skeleton::<V265>::read_with_sequences(data, flags, external)?,
            )),
            Self::V272 => Ok(DynamicSkeleton::V272(
                Skeleton::<V272>::read_with_sequences(data, flags, external)?,
            )),
            Self::V272PreSplit => Ok(DynamicSkeleton::V272PreSplit(
                Skeleton::<V272PreSplit>::read_with_sequences(data, flags, external)?,
            )),
            Self::V273 => Ok(DynamicSkeleton::V273(
                Skeleton::<V273>::read_with_sequences(data, flags, external)?,
            )),
            Self::V274 => Ok(DynamicSkeleton::V274(
                Skeleton::<V274>::read_with_sequences(data, flags, external)?,
            )),
        }
    }
}

impl m2::Write for DynamicSkin<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.write()?)
    }
}
impl m2::Write for DynamicSkeleton<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.write()?)
    }
}
