//! Animation initialization has its own value, independent of a payload baseline.
use super::super::attributes;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::animation::AnyDeclaration;
use crate::model::semantic::{AttributeValues, Quaternion, TextureId};
use crate::model::wem::{records, RawDocument, Ref};
use std::borrow::Cow;
use std::collections::BTreeMap;

pub(super) enum Initial {
    Scalar(Option<f32>),
    Vector2(Option<[f32; 2]>),
    Vector3(Option<[f32; 3]>),
    Vector4(Option<[f32; 4]>),
    Rotation(Option<Quaternion>),
    Boolean(Option<bool>),
    Unsigned16(Option<u16>),
    Unsigned(Option<u32>),
    Signed(Option<i32>),
    Bytes4(Option<[u8; 4]>),
    Unsigned4(Option<[u32; 4]>),
    Texture(Option<Option<TextureId>>),
}
pub(super) fn read(
    document: &RawDocument<'_>,
    source: &records::AnimChannel,
    textures: &[TextureId],
    texture_selection: bool,
) -> Result<Initial, Error> {
    let count = usize::from(source.init_value.entries != 0);
    let layer = records::AttrLayer {
        type_: source.value_type,
        data: source.init_value,
        ..Default::default()
    };
    let values = attributes::read(document, &layer, count)?;
    Ok(match values {
        AttributeValues::Scalar(values) => Initial::Scalar(values.get(0)),
        AttributeValues::Vector2(values) => Initial::Vector2(values.get(0)),
        AttributeValues::Vector3(values) => Initial::Vector3(values.get(0)),
        AttributeValues::Vector4(values) => Initial::Vector4(values.get(0)),
        AttributeValues::Quaternion(values) => Initial::Rotation(values.get(0).map(Quaternion)),
        AttributeValues::Boolean(values) => Initial::Boolean(values.get(0)),
        AttributeValues::U16(values) => Initial::Unsigned16(values.get(0)),
        AttributeValues::Unsigned(values) if texture_selection => Initial::Texture(
            values
                .get(0)
                .map(|index| {
                    if index == u32::MAX {
                        Ok(None)
                    } else {
                        textures
                            .get(index as usize)
                            .copied()
                            .map(Some)
                            .ok_or_else(|| {
                                Error::new(
                                    "wem.animation.initial",
                                    "image is outside the document texture table",
                                )
                            })
                    }
                })
                .transpose()?,
        ),
        AttributeValues::Unsigned(values) => Initial::Unsigned(values.get(0)),
        AttributeValues::Signed(values) => Initial::Signed(values.get(0)),
        AttributeValues::Byte4(values) => Initial::Bytes4(values.get(0)),
        AttributeValues::Unsigned4(values) => Initial::Unsigned4(values.get(0)),
        _ => {
            return Err(Error::new(
                "wem.animation.initial",
                "attribute representation is not an animation value domain",
            ));
        }
    })
}

pub(super) fn write(
    document: &mut RawDocument<'_>,
    declaration: &AnyDeclaration,
    textures: &BTreeMap<TextureId, u32>,
) -> Result<(records::AttrType, Ref<records::targets::Byte>), Error> {
    let mut bytes = Vec::new();
    macro_rules! scalar {
        ($value:expr) => {
            if let Some(value) = $value.initial {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        };
    }
    macro_rules! vector {
        ($value:expr) => {
            if let Some(value) = $value.initial {
                for component in value {
                    bytes.extend_from_slice(&component.to_le_bytes());
                }
            }
        };
    }
    let value_type = match declaration {
        AnyDeclaration::Scalar(_, value) => {
            scalar!(value);
            records::AttrType::F32
        }
        AnyDeclaration::Vector2(_, value) => {
            vector!(value);
            records::AttrType::F32x2
        }
        AnyDeclaration::Vector3(_, value) => {
            vector!(value);
            records::AttrType::F32x3
        }
        AnyDeclaration::Vector4(_, value) => {
            vector!(value);
            records::AttrType::F32x4
        }
        AnyDeclaration::Rotation(_, value) => {
            if let Some(value) = value.initial {
                for component in value.components() {
                    bytes.extend_from_slice(&component.to_le_bytes());
                }
            }
            records::AttrType::Quat
        }
        AnyDeclaration::Boolean(_, value) => {
            if let Some(value) = value.initial {
                bytes.push(u8::from(value));
            }
            records::AttrType::Bool
        }
        AnyDeclaration::Unsigned16(_, value) => {
            scalar!(value);
            records::AttrType::U16
        }
        AnyDeclaration::Unsigned(_, value) => {
            scalar!(value);
            records::AttrType::U32
        }
        AnyDeclaration::Signed(_, value) => {
            scalar!(value);
            records::AttrType::I32
        }
        AnyDeclaration::Bytes4(_, value) => {
            if let Some(value) = value.initial {
                bytes.extend_from_slice(&value);
            }
            records::AttrType::U8x4
        }
        AnyDeclaration::Unsigned4(_, value) => {
            vector!(value);
            records::AttrType::U32x4
        }
        AnyDeclaration::Texture(_, value) => {
            if let Some(value) = value.initial {
                let index = value
                    .map(|id| {
                        textures.get(&id).copied().ok_or_else(|| {
                            Error::new("wem.animation.initial", "image belongs to another document")
                        })
                    })
                    .transpose()?
                    .unwrap_or(u32::MAX);
                bytes.extend_from_slice(&index.to_le_bytes());
            }
            records::AttrType::U32
        }
    };
    if bytes.is_empty() {
        return Ok((value_type, Ref::default()));
    }
    let count = bytes.len() as u32;
    let section = document
        .append_data(*b"U8__", 0, count, Cow::Owned(bytes))
        .map_err(|error| Error::new("wem.animation.initial", error.to_string()))?;
    Ok((value_type, Ref::from_section(count, section.index())))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absence_is_distinct_from_declared_zero_and_integer_values_are_exact() {
        let mut document = RawDocument::new();
        let bytes = u32::MAX.to_le_bytes();
        let section = document
            .append(
                &bytes
                    .iter()
                    .map(|value| records::Byte { value: *value })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let source = records::AnimChannel {
            value_type: records::AttrType::U32,
            init_value: Ref::from_section(4, section.index()),
            ..Default::default()
        };
        assert!(matches!(
            read(&document, &source, &[], false).unwrap(),
            Initial::Unsigned(Some(u32::MAX))
        ));
        let source = records::AnimChannel {
            init_value: Ref::default(),
            ..source
        };
        assert!(matches!(
            read(&document, &source, &[], false).unwrap(),
            Initial::Unsigned(None)
        ));
    }
}
