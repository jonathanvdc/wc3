//! Checked, bounded reads over immutable bytes.

use crate::Error;

/// A copyable position within a byte slice. Child cursors are bounded slices.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
    base: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            base: 0,
        }
    }

    pub(crate) fn position(&self) -> usize {
        self.offset
    }

    pub(crate) fn absolute_position(&self) -> usize {
        self.base + self.offset
    }

    pub(crate) fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.offset..]
    }

    pub(crate) fn read_exact(&mut self, len: usize) -> Result<&'a [u8], Error> {
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

    pub(crate) fn peek_exact(&self, len: usize) -> Result<&'a [u8], Error> {
        let mut copy = *self;
        copy.read_exact(len)
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(
            self.read_exact(4)?.try_into().expect("four-byte word"),
        ))
    }

    pub(crate) fn read_f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    /// Advances this cursor and returns a cursor confined to those bytes.
    pub(crate) fn slice(&mut self, len: usize) -> Result<Self, Error> {
        let base = self.absolute_position();
        let bytes = self.read_exact(len)?;
        Ok(Self {
            bytes,
            offset: 0,
            base,
        })
    }

    pub(crate) fn finish(self) -> Result<(), Error> {
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
}
