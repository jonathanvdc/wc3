//! Logical M3 data driven values.
use super::super::parameters::{OpaqueBytes, OrderedParameters, ParameterEntry};
use crate::model::values::programmed::{ShaderFresnelFlags, ShaderType};
use crate::model::{Link, TextureId, ValidateLinks, ZeroVec};
use std::borrow::Cow;

/// Editable M3 DataDriven values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataDriven<'a> {
    /// Supplies U32_ CRC32 identifiers for shader fragments in their linking order.
    pub fragment_hashes: Vec<u32>,
    /// Supplies additional U32_ shader hashes available from revision 2.
    pub extra_hashes: Vec<u32>,
    /// Supplies the CHAR-encoded dictionary of shader properties.
    pub parameters: Parameters<'a>,
    /// Links to SCHR texture filenames selected by texture properties in the dictionary.
    pub textures: Vec<TextureId>,
    /// Unknown-purpose scalar reported as 1.0, 1.5, or 2.0 in the reference samples.
    pub unknown108: f32,
    /// Unknown-purpose scalar reported as 1.0 in the reference samples.
    pub unknown112: f32,
    /// The meaning of the unknown116 storage has not been established; its encoded bits are preserved.
    pub unknown116: f32,
    /// Stores the shader-permutation name’s CRC32; zero requests calculation during loading.
    pub effect_name_hash: u32,
    /// The meaning of the unknown124 storage has not been established; its encoded bits are preserved.
    pub unknown124: u32,
    /// Uninterpreted storage reported as zero in the reference samples.
    pub padding128: u32,
    /// The meaning of the unknown132 storage has not been established; its encoded bits are preserved.
    pub unknown132: i32,
    /// Contains packed bits whose individual meanings are not established.
    pub unknown136: u32,
    /// The meaning of the unknown140 storage has not been established; its encoded bits are preserved.
    pub unknown140: u32,
    /// The meaning of the unknown144 storage has not been established; its encoded bits are preserved.
    pub unknown144: u32,
    /// The meaning of the unknown148 storage has not been established; its encoded bits are preserved.
    pub unknown148: u8,
    /// Caches alpha and Fresnel settings derived from the property dictionary by the loader.
    pub alpha_fresnel_flags: ShaderFresnelFlags,
    /// Selects the shader-family prefix used to form the permutation name.
    pub shader_type: ShaderType,
    /// The meaning of the unknown151 storage has not been established; its encoded bits are preserved.
    pub unknown151: u8,
    /// Stores another permutation identifier; all bits set indicates no second permutation (revision 3 onward).
    pub effect_name_hash2: u32,
    /// Stores a third permutation identifier; all bits set indicates no third permutation (revision 3 onward).
    pub effect_name_hash3: u32,
}

/// A named shader key retains both its authored spelling and source hash.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NamedShaderKey<'a> {
    /// Source name hash, retained independently of the spelling.
    pub hash: u32,
    /// Authored shader property or group name.
    pub name: Cow<'a, str>,
}
/// An ordered named property with unresolved encoded value bytes.
pub type DataProperty<'a> = ParameterEntry<NamedShaderKey<'a>, OpaqueBytes>;
/// An ordered named group of shader properties.
pub type DataGroup<'a> =
    ParameterEntry<NamedShaderKey<'a>, OrderedParameters<NamedShaderKey<'a>, OpaqueBytes>>;

/// Editable M3 DataProperties values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataProperties<'a> {
    /// Stores the source material’s grouping metadata inline.
    pub groups: Vec<DataGroup<'a>>,
}

/// Shader-parameter data whose interpretation depends on the shader family.
#[derive(Clone, Debug, PartialEq)]
pub enum Parameters<'a> {
    /// Decoded named groups supplied by an authoring format.
    Named(DataProperties<'a>),
    /// Unresolved parameter language bytes; no dictionary schema is claimed.
    Unresolved(ZeroVec<'a, u8>),
}
impl Default for Parameters<'_> {
    fn default() -> Self {
        Self::Named(DataProperties::default())
    }
}

impl ValidateLinks for NamedShaderKey<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
