//! Ordered shader parameters retain their key domain and value interpretation.
use crate::model::{Link, ValidateLinks};

/// An authored shader parameter with independently selected identity, value, and controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParameterEntry<K, V, C = ()> {
    /// Identity in the selected shader parameter namespace.
    pub key: K,
    /// Authored value; opaque words remain distinct from interpreted numbers.
    pub value: V,
    /// Additional declarations governing this parameter.
    pub controls: C,
}

/// Ordered parameters, retaining repeated keys and their authored positions.
pub type OrderedParameters<K, V, C = ()> = Vec<ParameterEntry<K, V, C>>;

/// A shader-tag identifier in the Diablo III namespace.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShaderTag(pub u32);

/// A shader-value identifier in the Diablo IV namespace.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShaderValue(pub i32);

/// Uninterpreted parameter bits, including values that resemble nonfinite floats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OpaqueWord(pub u32);

/// Unresolved encoded parameter data; no scalar or dictionary interpretation is inferred.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpaqueBytes(pub Vec<u8>);

impl<K: ValidateLinks, V: ValidateLinks, C: ValidateLinks> ValidateLinks
    for ParameterEntry<K, V, C>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.key.visit_links(visitor);
        self.value.visit_links(visitor);
        self.controls.visit_links(visitor);
    }
}
impl ValidateLinks for ShaderTag {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for ShaderValue {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for OpaqueWord {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for OpaqueBytes {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
