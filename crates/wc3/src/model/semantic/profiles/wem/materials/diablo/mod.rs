//! WEM producer controls around the shared Diablo III shader vocabulary.
use crate::model::semantic::profiles::diablo::d3;
use crate::model::semantic::{Error, Link, ValidateLinks};
use bitfield::bitfield;

bitfield! { /// Producer-defined VariantFlags options; every bit is retained.
 #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
 pub struct VariantFlags(i32);
 /// Full authored option mask.
 pub bits,_:31,0;
}
impl VariantFlags {
    /// Constructs the complete authored option mask.
    pub const fn from_bits_retain(bits: i32) -> Self {
        Self(bits)
    }
}
/// WEM Diablo declaration with independently authored producer options.
#[derive(Clone, Debug, Default)]
pub struct DiabloMaterial<'a> {
    /// Producer variant options; absence differs from an explicitly empty mask.
    pub variant_flags: Option<VariantFlags>,
    /// Shared Diablo III shader state.
    pub shader: d3::Material<'a>,
}
impl ValidateLinks for DiabloMaterial<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.shader.visit_links(visitor);
    }
}
impl DiabloMaterial<'_> {
    /// Checks the shared shader's authored coefficients and texture-stage windows.
    pub fn validate(&self) -> Result<(), Error> {
        self.shader.validate()
    }
}
