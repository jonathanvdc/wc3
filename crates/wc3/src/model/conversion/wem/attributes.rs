//! Unpacked attribute buffers become borrowed, typed logical values.
use crate::model::conversion::Error;
use crate::model::semantic::{AttributeValues, Values};
use crate::model::wem::{records, RawDocument, Record, Ref};
use std::borrow::Cow;
use zerovec::{ule::AsULE, ZeroVec};

fn packed<'view, T: AsULE + Send + Sync + 'view>(
    bytes: &'view [u8],
) -> Result<Values<'view, T>, Error>
where
    T::ULE: Send + Sync,
{
    Ok(Values::from_zero_vec(ZeroVec::parse_bytes(bytes).map_err(
        |error| Error::new("wem.attribute", error.to_string()),
    )?))
}

pub(super) fn read<'view>(
    document: &'view RawDocument<'_>,
    layer: &records::AttrLayer,
    count: usize,
) -> Result<AttributeValues<'view>, Error> {
    let bytes = if let Some(section) = document
        .referenced_section(layer.data)
        .map_err(|e| Error::new("wem.attribute", e.to_string()))?
    {
        if section.tag() != records::Byte::TAG || section.revision() != records::Byte::VERSION {
            return Err(Error::new(
                "wem.attribute",
                "unpacked data has a different byte layout",
            ));
        }
        section
            .bytes()
            .get(..layer.data.entries as usize)
            .ok_or_else(|| Error::new("wem.attribute", "data span exceeds its buffer"))?
    } else {
        &[]
    };
    let width = match layer.type_ {
        records::AttrType::F32
        | records::AttrType::U32
        | records::AttrType::I32
        | records::AttrType::U8x4 => 4,
        records::AttrType::F32x2 => 8,
        records::AttrType::F32x3 => 12,
        records::AttrType::F32x4 | records::AttrType::Quat | records::AttrType::U32x4 => 16,
        records::AttrType::U16 => 2,
        records::AttrType::Bool => 1,
        _ => {
            return Err(Error::new(
                "wem.attribute.type",
                "unrecognized attribute interpretation",
            ));
        }
    };
    if count.checked_mul(width) != Some(bytes.len()) {
        return Err(Error::new(
            "wem.attribute",
            "declared domain size differs from unpacked element cardinality",
        ));
    }
    Ok(match layer.type_ {
        records::AttrType::F32 => AttributeValues::Scalar(packed(bytes)?),
        records::AttrType::F32x2 => AttributeValues::Vector2(packed(bytes)?),
        records::AttrType::F32x3 => AttributeValues::Vector3(packed(bytes)?),
        records::AttrType::F32x4 => AttributeValues::Vector4(packed(bytes)?),
        records::AttrType::Quat => AttributeValues::Quaternion(packed(bytes)?),
        records::AttrType::U8x4 => AttributeValues::Byte4(packed(bytes)?),
        records::AttrType::U16 => AttributeValues::U16(packed(bytes)?),
        records::AttrType::U32 => AttributeValues::Unsigned(packed(bytes)?),
        records::AttrType::I32 => AttributeValues::Signed(packed(bytes)?),
        records::AttrType::U32x4 => AttributeValues::Unsigned4(packed(bytes)?),
        records::AttrType::Bool => {
            AttributeValues::Boolean(Values::view(count, move |index| bytes[index] != 0))
        }
        _ => unreachable!("interpretation checked before constructing typed views"),
    })
}

pub(super) fn write<'view, 'data: 'view>(
    document: &mut RawDocument<'view>,
    values: &AttributeValues<'data>,
) -> Result<(records::AttrType, Ref<records::targets::Byte>), Error> {
    let packed = match values {
        AttributeValues::Scalar(v) => Some((records::AttrType::F32, v.borrowed_canonical_bytes())),
        AttributeValues::Vector2(v) => {
            Some((records::AttrType::F32x2, v.borrowed_canonical_bytes()))
        }
        AttributeValues::Vector3(v) => {
            Some((records::AttrType::F32x3, v.borrowed_canonical_bytes()))
        }
        AttributeValues::Vector4(v) => {
            Some((records::AttrType::F32x4, v.borrowed_canonical_bytes()))
        }
        AttributeValues::Quaternion(v) => {
            Some((records::AttrType::Quat, v.borrowed_canonical_bytes()))
        }
        AttributeValues::Byte4(v) => Some((records::AttrType::U8x4, v.borrowed_canonical_bytes())),
        AttributeValues::U16(v) => Some((records::AttrType::U16, v.borrowed_canonical_bytes())),
        AttributeValues::Unsigned(v) => {
            Some((records::AttrType::U32, v.borrowed_canonical_bytes()))
        }
        AttributeValues::Signed(v) => Some((records::AttrType::I32, v.borrowed_canonical_bytes())),
        AttributeValues::Unsigned4(v) => {
            Some((records::AttrType::U32x4, v.borrowed_canonical_bytes()))
        }
        _ => None,
    };
    if let Some((kind, Some(bytes))) = packed {
        if bytes.is_empty() {
            return Ok((kind, Ref::default()));
        }
        let count = u32::try_from(bytes.len())
            .map_err(|_| Error::new("wem.attribute", "byte count exceeds u32"))?;
        let section = document
            .append_data(
                records::Byte::TAG,
                records::Byte::VERSION,
                count,
                Cow::Borrowed(bytes),
            )
            .map_err(|error| Error::new("wem.attribute", error.to_string()))?;
        return Ok((kind, Ref::from_section(count, section.index())));
    }
    let mut bytes = Vec::new();
    macro_rules! scalar {
        ($values:ident) => {
            for value in $values.iter() {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        };
    }
    macro_rules! vector {
        ($values:ident) => {
            for value in $values.iter() {
                for component in value {
                    bytes.extend_from_slice(&component.to_le_bytes());
                }
            }
        };
    }
    let kind = match values {
        AttributeValues::SurfaceControls(_) => {
            return Err(Error::new(
                "wem.geometry.attributes",
                "surface records have no native attribute encoding",
            ))
        }
        AttributeValues::Scalar(v) => {
            scalar!(v);
            records::AttrType::F32
        }
        AttributeValues::Vector2(v) => {
            vector!(v);
            records::AttrType::F32x2
        }
        AttributeValues::Vector3(v) => {
            vector!(v);
            records::AttrType::F32x3
        }
        AttributeValues::Vector4(v) => {
            vector!(v);
            records::AttrType::F32x4
        }
        AttributeValues::Quaternion(v) => {
            vector!(v);
            records::AttrType::Quat
        }
        AttributeValues::Unsigned4(v) => {
            vector!(v);
            records::AttrType::U32x4
        }
        AttributeValues::Byte4(v) => {
            for value in v.iter() {
                bytes.extend_from_slice(&value);
            }
            records::AttrType::U8x4
        }
        AttributeValues::Boolean(v) => {
            bytes.extend(v.iter().map(u8::from));
            records::AttrType::Bool
        }
        AttributeValues::U16(v) => {
            scalar!(v);
            records::AttrType::U16
        }
        AttributeValues::Unsigned(v) => {
            scalar!(v);
            records::AttrType::U32
        }
        AttributeValues::Signed(v) => {
            scalar!(v);
            records::AttrType::I32
        }
    };
    if bytes.is_empty() {
        return Ok((kind, Default::default()));
    }
    let count = u32::try_from(bytes.len())
        .map_err(|_| Error::new("wem.attribute", "buffer exceeds native byte count"))?;
    let rows = bytes
        .into_iter()
        .map(|value| records::Byte { value })
        .collect::<Vec<_>>();
    let section = document
        .append(&rows)
        .map_err(|e| Error::new("wem.attribute", e.to_string()))?;
    Ok((kind, Ref::from_section(count, section.index())))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_width_values_borrow_unpacked_bytes() {
        let mut document = RawDocument::new();
        let source = AttributeValues::Unsigned(Values::owned(vec![16_777_217, u32::MAX]));
        let (type_, data) = write(&mut document, &source).unwrap();
        let layer = records::AttrLayer {
            type_,
            data,
            ..Default::default()
        };
        let AttributeValues::Unsigned(values) = read(&document, &layer, 2).unwrap() else {
            panic!("wrong numeric domain")
        };
        assert_eq!(values.get(0), Some(16_777_217));
        assert_eq!(values.get(1), Some(u32::MAX));
        assert!(values.is_borrowed());
        assert!(read(&document, &layer, 1).is_err());
    }
    #[test]
    fn quaternion_interpretation_and_boolean_cardinality_remain_typed() {
        let mut document = RawDocument::new();
        let source = AttributeValues::Quaternion(Values::owned(vec![[0., 0., 0., 1.]]));
        let (type_, data) = write(&mut document, &source).unwrap();
        let layer = records::AttrLayer {
            type_,
            data,
            ..Default::default()
        };
        assert!(matches!(
            read(&document, &layer, 1).unwrap(),
            AttributeValues::Quaternion(_)
        ));
        let source = AttributeValues::Boolean(Values::owned(vec![false, true]));
        let (type_, data) = write(&mut document, &source).unwrap();
        let layer = records::AttrLayer {
            type_,
            data,
            ..Default::default()
        };
        let AttributeValues::Boolean(values) = read(&document, &layer, 2).unwrap() else {
            panic!("wrong boolean domain")
        };
        assert_eq!(values.iter().collect::<Vec<_>>(), vec![false, true]);
    }
}
