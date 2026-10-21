//! Reusable canonical payload words and queue ownership, separate from VM temporaries.
use super::{snapshot, Arena, Element, Handle, Layout};
use crate::model::scene::popcorn::Value;
use glam::{Mat4, Quat, Vec3};
use std::mem::size_of;
type Result<T> = std::result::Result<T, &'static str>;
pub(crate) struct Storage {
    pub arena: Arena<Element>,
    particles: Vec<Option<Value>>,
    stride: usize,
    words: usize,
}
impl Storage {
    pub fn bytes(particles: usize, queued: usize, stride: usize) -> Option<usize> {
        let queued = if stride == 0 { 0 } else { queued };
        Arena::<Element>::storage_bytes(queued, stride)?.checked_add(
            particles
                .checked_mul(stride)?
                .checked_mul(size_of::<Option<Value>>())?,
        )
    }
    pub fn new(particles: usize, queued: usize, stride: usize) -> Result<Self> {
        let queued = if stride == 0 { 0 } else { queued };
        Ok(Self {
            arena: Arena::new(queued, stride)?,
            particles: vec![
                None;
                particles
                    .checked_mul(stride)
                    .ok_or("particle payload field count overflow")?
            ],
            stride,
            words: 0,
        })
    }
    pub fn clear(&mut self) {
        self.arena.clear();
        self.particles.fill(None);
        self.words = 0;
    }
    pub fn reset_work(&mut self) {
        self.words = 0;
    }
    pub fn charge(&mut self, words: usize, limit: usize) -> bool {
        let Some(next) = self.words.checked_add(words).filter(|next| *next <= limit) else {
            return false;
        };
        self.words = next;
        true
    }
    pub fn read(&self, particle: usize, field: usize) -> Result<Option<Value>> {
        if field >= self.stride {
            return Err("payload extraction root field is outside compiled storage");
        }
        Ok(self.particles[particle * self.stride + field])
    }
    pub fn snapshot_words(length: usize) -> usize {
        length * size_of::<Option<Element>>().div_ceil(4)
    }
    pub fn birth_words(&self, handle: Option<Handle>, root_length: usize) -> Result<usize> {
        if self.stride == 0 {
            return Ok(0);
        }
        let copied = root_length * size_of::<Option<Value>>().div_ceil(4);
        let sampled = if let Some(handle) = handle {
            self.arena
                .read(handle)?
                .iter()
                .flatten()
                .map(|element| element.first.kind().components())
                .sum()
        } else {
            0
        };
        Ok(copied + sampled)
    }
    pub fn initialize(
        &mut self,
        particle: usize,
        root_length: usize,
        handle: Option<Handle>,
        mapping: &[usize],
        fraction: f32,
        ordinal: u32,
    ) -> Result<()> {
        let root_length = root_length.min(self.stride);
        self.particles[particle * self.stride..particle * self.stride + root_length].fill(None);
        if let Some(handle) = handle {
            for (source, value) in self.arena.read(handle)?.iter().enumerate() {
                let Some(value) = value else {
                    continue;
                };
                let field = *mapping
                    .get(source)
                    .ok_or("queued payload mapping is incomplete")?;
                if field >= self.stride {
                    return Err("queued payload mapping exceeds compiled storage");
                }
                self.particles[particle * self.stride + field] =
                    Some(value.sample(fraction, ordinal)?);
            }
        }
        Ok(())
    }
    pub fn frame(&self, particle: usize, layout: &Layout) -> Result<Option<Mat4>> {
        if self.stride == 0 {
            return Ok(None);
        }
        let position = layout
            .position
            .and_then(|field| self.particles[particle * self.stride + field])
            .map(|value| {
                let [x, y, z, _] = value.words().map(f32::from_bits);
                Vec3::new(x, y, z)
            });
        let orientation = layout
            .orientation
            .and_then(|field| self.particles[particle * self.stride + field])
            .map(|value| Quat::from_array(value.words().map(f32::from_bits)));
        snapshot(position, orientation)
    }
}
