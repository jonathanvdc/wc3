//! Shared bounds checks for records with an outer size and embedded node.

use crate::Error;

/// Offsets within one validated record.
pub(crate) struct Layout {
    pub(crate) fixed_start: usize,
    pub(crate) track_start: usize,
}

pub(crate) fn layout(bytes: &[u8], tag: [u8; 4], fixed_size: usize) -> Result<Layout, Error> {
    let outer_size = bytes
        .get(..4)
        .ok_or(Error::MalformedRecord { tag, offset: 0 })?;
    if u32::from_le_bytes(outer_size.try_into().expect("four-byte size")) as usize != bytes.len() {
        return Err(Error::MalformedRecord { tag, offset: 0 });
    }
    let node_size_bytes = bytes
        .get(4..8)
        .ok_or(Error::MalformedRecord { tag, offset: 4 })?;
    let node_size =
        u32::from_le_bytes(node_size_bytes.try_into().expect("four-byte size")) as usize;
    if node_size < 96 {
        return Err(Error::MalformedRecord { tag, offset: 4 });
    }
    let fixed_start = 4usize
        .checked_add(node_size)
        .filter(|&end| end <= bytes.len())
        .ok_or(Error::MalformedRecord { tag, offset: 4 })?;
    let track_start = fixed_start
        .checked_add(fixed_size)
        .filter(|&end| end <= bytes.len())
        .ok_or(Error::MalformedRecord {
            tag,
            offset: fixed_start,
        })?;
    Ok(Layout {
        fixed_start,
        track_start,
    })
}

pub(crate) fn records(data: &[u8], tag: [u8; 4], fixed_size: usize) -> Result<Vec<&[u8]>, Error> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let size_bytes = data
            .get(offset..offset.saturating_add(4))
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
        let end = offset
            .checked_add(size)
            .filter(|&end| end <= data.len() && end > offset)
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let record = &data[offset..end];
        layout(record, tag, fixed_size)?;
        result.push(record);
        offset = end;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Record;

    #[test]
    fn checks_outer_and_embedded_node_sizes() {
        let node = crate::Node::new("Emitter", 0).unwrap();
        let mut record = vec![0; 4];
        record.extend_from_slice(&node.encode().unwrap());
        record.extend_from_slice(&[0; 8]);
        let size = record.len() as u32;
        record[..4].copy_from_slice(&size.to_le_bytes());
        assert_eq!(layout(&record, *b"TEST", 8).unwrap().fixed_start, 100);
        record[4..8].copy_from_slice(&1000u32.to_le_bytes());
        assert!(layout(&record, *b"TEST", 8).is_err());
    }
}
