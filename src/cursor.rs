//! Checked, bounded reads over immutable bytes.

use crate::Error;

/// A copyable position within immutable bytes. Child cursors are bounded slices.
#[derive(Clone, Copy, Debug)]
pub struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
    base: usize,
}

impl<'a> Cursor<'a> {
    /// Starts at the beginning of `bytes`.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            base: 0,
        }
    }

    /// Returns the position relative to this cursor's slice.
    pub fn position(&self) -> usize {
        self.offset
    }

    /// Returns the position relative to the original cursor's bytes.
    pub fn absolute_position(&self) -> usize {
        self.base + self.offset
    }

    /// Borrows all bytes not yet consumed by this cursor.
    pub fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.offset..]
    }

    /// Reads exactly `len` bytes and advances only on success.
    pub fn read_exact(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let start = self.offset;
        let end = start.checked_add(len).ok_or(Error::UnexpectedEnd {
            offset: self.absolute_position(),
            needed: len,
        })?;
        let value = self.bytes.get(start..end).ok_or(Error::UnexpectedEnd {
            offset: self.absolute_position(),
            needed: len,
        })?;
        self.offset = end;
        Ok(value)
    }

    /// Borrows the next `len` bytes without advancing.
    pub fn peek_exact(&self, len: usize) -> Result<&'a [u8], Error> {
        let mut copy = *self;
        copy.read_exact(len)
    }

    /// Reads a little-endian `u32`.
    pub fn read_u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(
            self.read_exact(4)?.try_into().expect("four-byte word"),
        ))
    }

    /// Reads an IEEE 754 float from four little-endian bytes.
    pub fn read_f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    /// Reads three IEEE 754 floats from twelve little-endian bytes.
    pub fn read_vec3(&mut self) -> Result<[f32; 3], Error> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        Ok([x, y, z])
    }

    /// Advances this cursor and returns a cursor confined to those bytes.
    /// Copy the parent first if parsing the child may need to be rolled back.
    pub fn slice(&mut self, len: usize) -> Result<Self, Error> {
        let base = self.absolute_position();
        let bytes = self.read_exact(len)?;
        Ok(Self {
            bytes,
            offset: 0,
            base,
        })
    }

    /// Reads a little-endian size that includes its own four bytes, then
    /// returns a cursor bounded to the remaining record body.
    /// Leaves the parent in place if the size or body is invalid.
    pub fn slice_u32_sized(&mut self) -> Result<Self, Error> {
        let mut next = *self;
        let start = next.absolute_position();
        let length = next.read_u32()? as usize;
        let body_len = length.checked_sub(4).ok_or(Error::InvalidRecordLength {
            offset: start,
            length,
        })?;
        let body = next.slice(body_len)?;
        *self = next;
        Ok(body)
    }

    /// Requires that all bytes in this cursor's slice were consumed.
    pub fn finish(self) -> Result<(), Error> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(Error::TrailingRecordBytes {
                consumed: self.offset,
                total: self.bytes.len(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_are_bounded_and_cursors_are_copyable_checkpoints() {
        let mut cursor = Cursor::new(&[1, 2, 3, 4, 5]);
        assert_eq!(cursor.read_u32().unwrap(), 0x0403_0201);
        let checkpoint = cursor;
        assert_eq!(cursor.read_exact(1).unwrap(), &[5]);
        cursor = checkpoint;
        assert_eq!(cursor.read_exact(1).unwrap(), &[5]);
        cursor.finish().unwrap();

        let mut parent = Cursor::new(&[10, 11, 12, 13]);
        let mut child = parent.slice(2).unwrap();
        assert_eq!(parent.position(), 2);
        assert_eq!(child.read_exact(2).unwrap(), &[10, 11]);
        assert_eq!(
            child.read_exact(1),
            Err(Error::UnexpectedEnd {
                offset: 2,
                needed: 1
            })
        );
        assert_eq!(child.position(), 2);
        assert_eq!(parent.read_exact(2).unwrap(), &[12, 13]);

        let mut outer = Cursor::new(&[20, 21, 22, 23]);
        outer.read_exact(1).unwrap();
        let mut middle = outer.slice(2).unwrap();
        let mut inner = middle.slice(1).unwrap();
        assert_eq!(inner.absolute_position(), 1);
        assert_eq!(
            inner.read_exact(2),
            Err(Error::UnexpectedEnd {
                offset: 1,
                needed: 2
            })
        );
        assert_eq!(inner.position(), 0);
        assert_eq!(
            middle.finish(),
            Err(Error::TrailingRecordBytes {
                consumed: 1,
                total: 2
            })
        );
    }

    #[test]
    fn sized_slice_consumes_one_record_and_preserves_position_on_failure() {
        let bytes = [6, 0, 0, 0, 42, 43, 9];
        let mut cursor = Cursor::new(&bytes);
        let mut body = cursor.slice_u32_sized().unwrap();
        assert_eq!(cursor.position(), 6);
        assert_eq!(body.read_exact(2).unwrap(), &[42, 43]);
        body.finish().unwrap();
        assert_eq!(cursor.read_exact(1).unwrap(), &[9]);

        let mut short = Cursor::new(&[3, 0, 0, 0]);
        assert_eq!(
            short.slice_u32_sized().unwrap_err(),
            Error::InvalidRecordLength {
                offset: 0,
                length: 3
            }
        );
        assert_eq!(short.position(), 0);

        let mut truncated = Cursor::new(&[8, 0, 0, 0, 1]);
        assert_eq!(
            truncated.slice_u32_sized().unwrap_err(),
            Error::UnexpectedEnd {
                offset: 4,
                needed: 4
            }
        );
        assert_eq!(truncated.position(), 0);
    }
}
