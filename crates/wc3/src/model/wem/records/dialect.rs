//! Static section revisions and shared capabilities for selected fields.
use crate::model::wem::{Validate, VisitReferences};
use std::fmt::Debug;
mod sealed {
    pub trait Sealed {}
}
/// A supported section revision marker; selections are fixed by this library.
pub trait Revision: sealed::Sealed + Clone + Copy + Debug + Default + PartialEq {}
/// Capabilities shared by fields selected through a revision dialect.
pub trait NativeField: Clone + Debug + Default + PartialEq + Validate + VisitReferences {}
impl<T: Clone + Debug + Default + PartialEq + Validate + VisitReferences> NativeField for T {}
/// Independent section revision 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V0;
impl sealed::Sealed for V0 {}
impl Revision for V0 {}
/// Independent section revision 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V1;
impl sealed::Sealed for V1 {}
impl Revision for V1 {}
/// Independent section revision 2.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V2;
impl sealed::Sealed for V2 {}
impl Revision for V2 {}
/// Independent section revision 3.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V3;
impl sealed::Sealed for V3 {}
impl Revision for V3 {}
/// Independent section revision 4.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V4;
impl sealed::Sealed for V4 {}
impl Revision for V4 {}
/// Independent section revision 5.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V5;
impl sealed::Sealed for V5 {}
impl Revision for V5 {}
/// Independent section revision 6.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V6;
impl sealed::Sealed for V6 {}
impl Revision for V6 {}
/// Independent section revision 7.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V7;
impl sealed::Sealed for V7 {}
impl Revision for V7 {}
/// Independent section revision 8.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V8;
impl sealed::Sealed for V8 {}
impl Revision for V8 {}
/// Independent section revision 9.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V9;
impl sealed::Sealed for V9 {}
impl Revision for V9 {}
/// Independent section revision 10.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V10;
impl sealed::Sealed for V10 {}
impl Revision for V10 {}
/// Independent section revision 11.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V11;
impl sealed::Sealed for V11 {}
impl Revision for V11 {}
/// Independent section revision 12.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V12;
impl sealed::Sealed for V12 {}
impl Revision for V12 {}
/// Independent section revision 13.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V13;
impl sealed::Sealed for V13 {}
impl Revision for V13 {}
/// Independent section revision 14.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V14;
impl sealed::Sealed for V14 {}
impl Revision for V14 {}
/// Independent section revision 15.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V15;
impl sealed::Sealed for V15 {}
impl Revision for V15 {}
/// Independent section revision 16.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V16;
impl sealed::Sealed for V16 {}
impl Revision for V16 {}
/// Independent section revision 17.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V17;
impl sealed::Sealed for V17 {}
impl Revision for V17 {}
/// Independent section revision 18.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V18;
impl sealed::Sealed for V18 {}
impl Revision for V18 {}
/// Independent section revision 19.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V19;
impl sealed::Sealed for V19 {}
impl Revision for V19 {}
