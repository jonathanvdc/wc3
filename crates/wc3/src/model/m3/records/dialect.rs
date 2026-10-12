//! Revision markers select individual section layouts independently of file envelopes.
use crate::model::m3::{self, Validate, VisitReferences};
use std::fmt::Debug;

mod sealed {
    pub trait Revision {}
}

/// Supported numeric section-revision identity, independent of its record family.
///
/// Family dialect traits select which of these markers have an established
/// field layout for that family. Callers cannot define additional markers.
pub trait Revision:
    sealed::Revision + Copy + Clone + Debug + Default + PartialEq + Eq + 'static
{
}

/// Common value capabilities required by selected native fields.
pub trait NativeField: Copy + Debug + Default + PartialEq + Validate + VisitReferences {}
impl<T: Copy + Debug + Default + PartialEq + Validate + VisitReferences> NativeField for T {}
/// Independent section revision 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V0;
/// Independent section revision 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V1;
/// Independent section revision 2.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V2;
/// Independent section revision 3.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V3;
/// Independent section revision 4.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V4;
/// Independent section revision 5.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V5;
/// Independent section revision 6.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V6;
/// Independent section revision 7.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V7;
/// Independent section revision 8.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V8;
/// Independent section revision 9.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V9;
/// Independent section revision 10.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V10;
/// Independent section revision 11.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V11;
/// Independent section revision 12.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V12;
/// Independent section revision 13.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V13;
/// Independent section revision 14.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V14;
/// Independent section revision 15.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V15;
/// Independent section revision 16.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V16;
/// Independent section revision 17.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V17;
/// Independent section revision 18.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V18;
/// Independent section revision 19.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V19;
/// Independent section revision 20.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V20;
/// Independent section revision 21.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V21;
/// Independent section revision 22.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V22;
/// Independent section revision 23.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V23;
/// Independent section revision 24.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V24;
/// Independent section revision 25.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V25;
/// Independent section revision 26.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V26;
/// Independent section revision 27.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V27;
/// Independent section revision 28.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V28;
/// Independent section revision 29.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V29;
/// Independent section revision 30.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V30;

/// Fixed inline bytes whose contents have no assigned field interpretation.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct FixedBytes<const N: usize>(
    /// Bytes in their encoded order, without alignment or normalization.
    pub [u8; N],
);
impl<const N: usize> Default for FixedBytes<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl sealed::Revision for V0 {}
impl Revision for V0 {}
impl sealed::Revision for V1 {}
impl Revision for V1 {}
impl sealed::Revision for V2 {}
impl Revision for V2 {}
impl sealed::Revision for V3 {}
impl Revision for V3 {}
impl sealed::Revision for V4 {}
impl Revision for V4 {}
impl sealed::Revision for V5 {}
impl Revision for V5 {}
impl sealed::Revision for V6 {}
impl Revision for V6 {}
impl sealed::Revision for V7 {}
impl Revision for V7 {}
impl sealed::Revision for V8 {}
impl Revision for V8 {}
impl sealed::Revision for V9 {}
impl Revision for V9 {}
impl sealed::Revision for V10 {}
impl Revision for V10 {}
impl sealed::Revision for V11 {}
impl Revision for V11 {}
impl sealed::Revision for V12 {}
impl Revision for V12 {}
impl sealed::Revision for V13 {}
impl Revision for V13 {}
impl sealed::Revision for V14 {}
impl Revision for V14 {}
impl sealed::Revision for V15 {}
impl Revision for V15 {}
impl sealed::Revision for V16 {}
impl Revision for V16 {}
impl sealed::Revision for V17 {}
impl Revision for V17 {}
impl sealed::Revision for V18 {}
impl Revision for V18 {}
impl sealed::Revision for V19 {}
impl Revision for V19 {}
impl sealed::Revision for V20 {}
impl Revision for V20 {}
impl sealed::Revision for V21 {}
impl Revision for V21 {}
impl sealed::Revision for V22 {}
impl Revision for V22 {}
impl sealed::Revision for V23 {}
impl Revision for V23 {}
impl sealed::Revision for V24 {}
impl Revision for V24 {}
impl sealed::Revision for V25 {}
impl Revision for V25 {}
impl sealed::Revision for V26 {}
impl Revision for V26 {}
impl sealed::Revision for V27 {}
impl Revision for V27 {}
impl sealed::Revision for V28 {}
impl Revision for V28 {}
impl sealed::Revision for V29 {}
impl Revision for V29 {}
impl sealed::Revision for V30 {}
impl Revision for V30 {}
