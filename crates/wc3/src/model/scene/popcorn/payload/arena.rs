//! Fixed-capacity immutable payload snapshots shared by queued fanout births.
use std::mem::size_of;

type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Handle {
    slot: u32,
    generation: u32,
}
#[derive(Clone, Copy, Default)]
struct Lease {
    generation: u32,
    references: usize,
    length: usize,
}
/// A payload snapshot is immutable until its last queued owner releases it.
/// Every field and lease is allocated before execution; no birth owns a Vec or Arc.
pub(crate) struct Arena<T> {
    fields: Vec<Option<T>>,
    leases: Vec<Lease>,
    free: Vec<usize>,
    stride: usize,
}
impl<T: Copy> Arena<T> {
    pub fn storage_bytes(capacity: usize, stride: usize) -> Option<usize> {
        capacity
            .checked_mul(stride)?
            .checked_mul(size_of::<Option<T>>())?
            .checked_add(capacity.checked_mul(size_of::<Lease>() + size_of::<usize>())?)
    }
    pub fn new(capacity: usize, stride: usize) -> Result<Self> {
        if capacity > u32::MAX as usize {
            return Err("payload snapshot capacity exceeds handle space");
        }
        let fields = capacity
            .checked_mul(stride)
            .ok_or("payload snapshot field count overflow")?;
        Ok(Self {
            fields: vec![None; fields],
            leases: vec![Lease::default(); capacity],
            free: (0..capacity).rev().collect(),
            stride,
        })
    }
    /// Reserve one creator lease; retain each queued owner before releasing the creator.
    pub fn snapshot(&mut self, fields: &[Option<T>]) -> Result<Handle> {
        if fields.len() > self.stride {
            return Err("payload snapshot exceeds compiled field stride");
        }
        let slot = *self
            .free
            .last()
            .ok_or("payload snapshot capacity exhausted")?;
        let lease = &mut self.leases[slot];
        let generation = lease
            .generation
            .checked_add(1)
            .ok_or("payload snapshot incarnation exhausted")?;
        self.free.pop();
        *lease = Lease {
            generation,
            references: 1,
            length: fields.len(),
        };
        let start = slot * self.stride;
        self.fields[start..start + fields.len()].copy_from_slice(fields);
        Ok(Handle {
            slot: slot as u32,
            generation,
        })
    }
    fn lease(&self, handle: Handle) -> Result<&Lease> {
        self.leases
            .get(handle.slot as usize)
            .filter(|lease| lease.generation == handle.generation && lease.references != 0)
            .ok_or("payload snapshot handle is stale or belongs to another incarnation")
    }
    pub fn read(&self, handle: Handle) -> Result<&[Option<T>]> {
        let lease = self.lease(handle)?;
        let start = handle.slot as usize * self.stride;
        Ok(&self.fields[start..start + lease.length])
    }
    pub fn retain(&mut self, handle: Handle) -> Result<()> {
        let references = self
            .lease(handle)?
            .references
            .checked_add(1)
            .ok_or("payload snapshot ownership overflow")?;
        self.leases[handle.slot as usize].references = references;
        Ok(())
    }
    pub fn release(&mut self, handle: Handle) -> Result<()> {
        self.lease(handle)?;
        let slot = handle.slot as usize;
        let lease = &mut self.leases[slot];
        lease.references -= 1;
        if lease.references == 0 {
            self.fields[slot * self.stride..slot * self.stride + lease.length].fill(None);
            lease.length = 0;
            self.free.push(slot);
        }
        Ok(())
    }
    /// Discard queued ownership on restart or after a terminal execution failure.
    pub fn clear(&mut self) {
        for lease in &mut self.leases {
            lease.references = 0;
            lease.length = 0;
        }
        self.fields.fill(None);
        self.free.clear();
        self.free.extend((0..self.leases.len()).rev());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fanout_owns_one_snapshot_until_last_reference_and_reuse_invalidates_old_handles() {
        let mut arena = Arena::new(1, 3).unwrap();
        let original = arena.snapshot(&[Some(4), None, Some(9)]).unwrap();
        arena.retain(original).unwrap();
        arena.retain(original).unwrap();
        arena.release(original).unwrap(); // Creator hands ownership to two queued births.
        assert_eq!(arena.read(original).unwrap(), &[Some(4), None, Some(9)]);
        arena.release(original).unwrap();
        assert!(arena.snapshot(&[]).is_err());
        arena.release(original).unwrap();
        assert!(arena.read(original).is_err());
        assert!(arena.release(original).is_err());
        let replacement = arena.snapshot(&[Some(8)]).unwrap();
        assert_ne!(replacement, original);
        assert_eq!(arena.read(replacement).unwrap(), &[Some(8)]);
        assert!(arena.retain(original).is_err());
    }
    #[test]
    fn empty_payloads_have_ownership_and_oversize_failures_do_not_consume_capacity() {
        let mut arena = Arena::<u32>::new(1, 0).unwrap();
        assert!(arena.snapshot(&[Some(0)]).is_err());
        let empty = arena.snapshot(&[]).unwrap();
        assert!(arena.read(empty).unwrap().is_empty());
        assert!(arena.snapshot(&[]).is_err());
        arena.clear();
        assert!(arena.read(empty).is_err());
        assert_ne!(arena.snapshot(&[]).unwrap(), empty);
        assert!(Arena::<u32>::storage_bytes(usize::MAX, usize::MAX).is_none());
    }
}
