use super::*;
use std::array::from_fn;
pub(super) fn text(
    doc: &mut RawDocument<'static>,
    value: &str,
) -> Result<Ref<r::targets::Character>, WriteError> {
    if value.is_empty() {
        return Ok(Ref::default());
    }
    let values: Vec<_> = value.bytes().map(|value| r::Character { value }).collect();
    Ok(put(doc, &values)?.cast())
}

pub(super) fn put<T: Record + Write>(
    doc: &mut RawDocument<'static>,
    values: &[T],
) -> Result<Ref<T>, WriteError> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let id = doc.append(values)?;
    Ok(Ref::from_section(
        u32::try_from(values.len()).map_err(|_| WriteError::invalid("array count"))?,
        id.index(),
    ))
}

pub(super) fn bytes(
    doc: &mut RawDocument<'static>,
    values: &[u8],
) -> Result<Ref<r::targets::Byte>, WriteError> {
    let rows: Vec<_> = values.iter().map(|&value| r::Byte { value }).collect();
    Ok(put(doc, &rows)?.cast())
}

pub(super) fn words(
    doc: &mut RawDocument<'static>,
    values: &[u32],
) -> Result<Ref<r::targets::U32>, WriteError> {
    let rows: Vec<_> = values.iter().map(|&value| r::U32 { value }).collect();
    Ok(put(doc, &rows)?.cast())
}

pub(super) fn floats(
    doc: &mut RawDocument<'static>,
    values: &[f32],
) -> Result<Ref<r::targets::Real>, WriteError> {
    let rows: Vec<_> = values.iter().map(|&value| r::Real { value }).collect();
    Ok(put(doc, &rows)?.cast())
}

pub(super) fn attribute<const N: usize>(
    doc: &mut RawDocument<'static>,
    name: &str,
    values: &[[f32; N]],
    kind: r::AttrType,
) -> Result<Option<r::AttrLayer>, WriteError> {
    if values.is_empty() {
        return Ok(None);
    }
    let encoded: Vec<_> = values
        .iter()
        .flat_map(|v| v.iter().flat_map(|c| c.to_le_bytes()))
        .collect();
    Ok(Some(r::AttrLayer {
        name: text(doc, name)?,
        domain: r::Domain::Vertex,
        type_: kind,
        storage: r::AttributeEncoding::Float32,
        data: bytes(doc, &encoded)?,
    }))
}

pub(super) fn metadata(
    doc: &mut RawDocument<'static>,
    values: &[(&str, i64, String)],
) -> Result<r::NativeBag, WriteError> {
    let mut rows = Vec::new();
    for (name, value, body) in values {
        rows.push(r::NativeEntry::<r::V2> {
            name: text(doc, name)?,
            value: *value,
            text: text(doc, body)?,
        });
    }
    Ok(r::NativeBag {
        entries: put(doc, &rows)?.cast(),
    })
}

pub(super) fn rows<T>(doc: &RawDocument<'_>, reference: Ref<T>) -> Result<Vec<T>, ReadError>
where
    T: Record + RecordLayout,
    for<'a> T: Read<'a>,
{
    doc.resolve(reference)?.iter().collect()
}

pub(super) fn text_view<'a>(
    doc: &'a RawDocument<'_>,
    reference: Ref<r::targets::Character>,
) -> Result<Cow<'a, str>, ReadError> {
    Ok(Cow::Borrowed(doc.string(reference)?.unwrap_or("")))
}

pub(super) fn rawbytes<'a>(
    doc: &'a RawDocument<'_>,
    reference: Ref<r::targets::Byte>,
) -> Result<&'a [u8], ReadError> {
    if reference.entries == 0 {
        return Ok(&[]);
    }
    let section = doc
        .referenced_section(reference)?
        .ok_or_else(|| ReadError::invalid(0, "byte reference"))?;
    section
        .bytes()
        .get(..reference.entries as usize)
        .ok_or_else(|| ReadError::invalid(0, "byte span"))
}

pub(super) fn metadata_read(
    doc: &RawDocument<'_>,
    bag: r::NativeBag,
) -> Result<BTreeMap<String, (i64, String)>, ReadError> {
    let mut result = BTreeMap::new();
    if bag.entries.entries == 0 {
        return Ok(result);
    }
    let section = doc
        .referenced_section(bag.entries)?
        .ok_or_else(|| ReadError::invalid(0, "metadata reference"))?;
    match section.revision() {
        1 => {
            for row in rows::<r::NativeEntry<r::V1>>(doc, bag.entries.cast())? {
                result.insert(
                    doc.string(row.name)?.unwrap_or("").into(),
                    (row.value, String::new()),
                );
            }
        }
        2 => {
            for row in rows::<r::NativeEntry<r::V2>>(doc, bag.entries.cast())? {
                result.insert(
                    doc.string(row.name)?.unwrap_or("").into(),
                    (row.value, doc.string(row.text)?.unwrap_or("").into()),
                );
            }
        }
        _ => return Err(ReadError::unsupported(0, "native metadata revision")),
    }
    Ok(result)
}

pub(super) fn float_values<const N: usize>(
    doc: &RawDocument<'_>,
    layer: &r::AttrLayer,
) -> Result<Vec<[f32; N]>, ReadError> {
    // ATTR storage describes an export preference; data always holds unpacked values.
    let expected = match N {
        1 => r::AttrType::F32,
        2 => r::AttrType::F32x2,
        3 => r::AttrType::F32x3,
        4 => r::AttrType::F32x4,
        _ => return Err(ReadError::unsupported(0, "attribute component count")),
    };
    if layer.type_ != expected && !(N == 4 && layer.type_ == r::AttrType::Quat) {
        return Err(ReadError::unsupported(0, "attribute value type"));
    }
    let bytes = rawbytes(doc, layer.data)?;
    let stride = N
        .checked_mul(4)
        .ok_or_else(|| ReadError::invalid(0, "attribute width"))?;
    if !bytes.len().is_multiple_of(stride) {
        return Err(ReadError::invalid(0, "attribute byte count"));
    }
    Ok(bytes
        .chunks_exact(stride)
        .map(|row| from_fn(|i| f32::from_le_bytes(row[i * 4..i * 4 + 4].try_into().unwrap())))
        .collect())
}

pub(super) fn decode_matrix(hex: &str) -> Result<[[f32; 4]; 4], ReadError> {
    if hex.len() != 128 {
        return Err(ReadError::invalid(0, "inverse bind metadata"));
    }
    let mut bytes = [0; 64];
    for (i, pair) in hex.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let a = (pair[0] as char)
            .to_digit(16)
            .ok_or_else(|| ReadError::invalid(0, "inverse bind metadata"))?;
        let b = (pair[1] as char)
            .to_digit(16)
            .ok_or_else(|| ReadError::invalid(0, "inverse bind metadata"))?;
        bytes[i] = (a * 16 + b) as u8;
    }
    Ok(from_fn(|r| {
        from_fn(|c| {
            let at = (r * 4 + c) * 4;
            f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
        })
    }))
}

#[cfg(test)]
#[path = "codec_tests.rs"]
mod tests;
