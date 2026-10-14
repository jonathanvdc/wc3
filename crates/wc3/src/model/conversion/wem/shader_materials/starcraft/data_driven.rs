//! StarCraft shader inputs resolve text independently of source index sections.
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::materials::starcraft::DataDriven;
use crate::model::semantic::profiles::wem::materials::starcraft::{
    DataDrivenGroup, DataDrivenProperties, DataDrivenProperty,
};
use crate::model::wem::records::{M3AlphaFresnelFlags, M3DataDriven, M3MaterialShaderType};
use crate::model::wem::records::{M3DataDrivenGroup, M3DataDrivenProperties, M3DataDrivenProperty};

use crate::model::conversion::wem::text;

use crate::model::semantic::profiles::m3::values::{ShaderFresnelFlags, ShaderType};

use crate::model::wem::{InlineList, RawDocument};
pub(super) fn read_data_driven<'view>(
    document: &'view RawDocument<'_>,
    value: &M3DataDriven,
) -> Result<DataDriven<'view>, Error> {
    Ok(DataDriven {
        material_name: text::read(document, value.material_name)?,
        fragment_hashes: value.fragment_hashes.0.clone(),
        extra_hashes: value.extra_hashes.0.clone(),
        property_blob: value.property_blob.0.clone(),
        texture_paths: value
            .texture_paths
            .0
            .iter()
            .map(|value| text::read(document, *value))
            .collect::<Result<Vec<_>, Error>>()?,
        unknown108: value.unknown108,
        unknown112: value.unknown112,
        unknown116: value.unknown116,
        effect_name_hash: value.effect_name_hash,
        unknown124: value.unknown124,
        padding128: value.padding128,
        unknown132: value.unknown132,
        unknown136: value.unknown136,
        unknown140: value.unknown140,
        unknown144: value.unknown144,
        unknown148: value.unknown148,
        alpha_fresnel_flags: ShaderFresnelFlags::from(value.alpha_fresnel_flags.bits()),
        shader_type: ShaderType::from_raw(value.shader_type.raw()),
        unknown151: value.unknown151,
        effect_name_hash2: value.effect_name_hash2,
        effect_name_hash3: value.effect_name_hash3,
    })
}
pub(super) fn write_data_driven(
    document: &mut RawDocument<'_>,
    value: &DataDriven<'_>,
) -> Result<M3DataDriven, Error> {
    Ok(M3DataDriven {
        material_name: text::write_bytes(document, value.material_name.as_bytes())?,
        fragment_hashes: InlineList(value.fragment_hashes.clone()),
        extra_hashes: InlineList(value.extra_hashes.clone()),
        property_blob: InlineList(value.property_blob.clone()),
        texture_paths: InlineList(
            value
                .texture_paths
                .iter()
                .map(|value| text::write_bytes(document, value.as_bytes()))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
        unknown108: value.unknown108,
        unknown112: value.unknown112,
        unknown116: value.unknown116,
        effect_name_hash: value.effect_name_hash,
        unknown124: value.unknown124,
        padding128: value.padding128,
        unknown132: value.unknown132,
        unknown136: value.unknown136,
        unknown140: value.unknown140,
        unknown144: value.unknown144,
        unknown148: value.unknown148,
        alpha_fresnel_flags: M3AlphaFresnelFlags::from_bits_retain(
            value.alpha_fresnel_flags.bits(),
        ),
        shader_type: M3MaterialShaderType::from_raw(value.shader_type.raw()),
        unknown151: value.unknown151,
        effect_name_hash2: value.effect_name_hash2,
        effect_name_hash3: value.effect_name_hash3,
    })
}
pub(super) fn read_data_driven_property<'view>(
    document: &'view RawDocument<'_>,
    value: &M3DataDrivenProperty,
) -> Result<DataDrivenProperty<'view>, Error> {
    Ok(DataDrivenProperty {
        name_hash: value.name_hash,
        name: text::read(document, value.name)?,
        data: value.data.0.clone(),
    })
}
pub(super) fn write_data_driven_property(
    document: &mut RawDocument<'_>,
    value: &DataDrivenProperty<'_>,
) -> Result<M3DataDrivenProperty, Error> {
    Ok(M3DataDrivenProperty {
        name_hash: value.name_hash,
        name: text::write_bytes(document, value.name.as_bytes())?,
        data: InlineList(value.data.clone()),
    })
}
pub(super) fn read_data_driven_group<'view>(
    document: &'view RawDocument<'_>,
    value: &M3DataDrivenGroup,
) -> Result<DataDrivenGroup<'view>, Error> {
    Ok(DataDrivenGroup {
        name_hash: value.name_hash,
        name: text::read(document, value.name)?,
        properties: value
            .properties
            .0
            .iter()
            .map(|value| read_data_driven_property(document, value))
            .collect::<Result<Vec<_>, Error>>()?,
    })
}
pub(super) fn write_data_driven_group(
    document: &mut RawDocument<'_>,
    value: &DataDrivenGroup<'_>,
) -> Result<M3DataDrivenGroup, Error> {
    Ok(M3DataDrivenGroup {
        name_hash: value.name_hash,
        name: text::write_bytes(document, value.name.as_bytes())?,
        properties: InlineList(
            value
                .properties
                .iter()
                .map(|value| write_data_driven_property(document, value))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
    })
}
pub(super) fn read_data_driven_properties<'view>(
    document: &'view RawDocument<'_>,
    value: &M3DataDrivenProperties,
) -> Result<DataDrivenProperties<'view>, Error> {
    Ok(DataDrivenProperties {
        groups: value
            .groups
            .0
            .iter()
            .map(|value| read_data_driven_group(document, value))
            .collect::<Result<Vec<_>, Error>>()?,
    })
}
pub(super) fn write_data_driven_properties(
    document: &mut RawDocument<'_>,
    value: &DataDrivenProperties<'_>,
) -> Result<M3DataDrivenProperties, Error> {
    Ok(M3DataDrivenProperties {
        groups: InlineList(
            value
                .groups
                .iter()
                .map(|value| write_data_driven_group(document, value))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
    })
}
