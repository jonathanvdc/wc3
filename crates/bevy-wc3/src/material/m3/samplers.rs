//! Reuse sampler states independently of the standard shader's image roles.
use bevy::prelude::{Handle, Image};
use std::array::from_fn;
use wc3::model::Sampler;

pub(super) const SAMPLER_COUNT: usize = 6;
pub(super) struct SamplerPool {
    entries: [Option<(Sampler, Handle<Image>)>; SAMPLER_COUNT],
}
impl Default for SamplerPool {
    fn default() -> Self {
        Self {
            entries: from_fn(|_| None),
        }
    }
}
impl SamplerPool {
    pub(super) fn intern(&mut self, sampler: Sampler, image: &Handle<Image>) -> Option<u32> {
        if let Some(index) = self.entries.iter().position(|entry| {
            entry
                .as_ref()
                .is_some_and(|(existing, _)| *existing == sampler)
        }) {
            return Some(index as u32);
        }
        let index = self.entries.iter().position(Option::is_none)?;
        self.entries[index] = Some((sampler, image.clone()));
        Some(index as u32)
    }
    pub(super) fn images(self) -> [Option<Handle<Image>>; SAMPLER_COUNT] {
        self.entries.map(|entry| entry.map(|(_, image)| image))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::AddressMode;
    #[test]
    fn all_native_wrap_pairs_fit_and_roles_share_matching_samplers() {
        let mut pool = SamplerPool::default();
        let image = Handle::<Image>::default();
        let mut pairs = Vec::new();
        for u in [AddressMode::Clamp, AddressMode::Repeat] {
            for v in [AddressMode::Clamp, AddressMode::Repeat] {
                let sampler = Sampler { u, v };
                let index = pool.intern(sampler, &image).unwrap();
                assert_eq!(pool.intern(sampler, &image), Some(index));
                pairs.push(index);
            }
        }
        assert_eq!(pairs, [0, 1, 2, 3]);
        assert_eq!(pool.images().iter().flatten().count(), 4);
    }
    #[test]
    fn exhaustion_is_explicit_instead_of_changing_authored_sampling() {
        let mut pool = SamplerPool::default();
        let image = Handle::<Image>::default();
        let modes = [AddressMode::Clamp, AddressMode::Repeat, AddressMode::Mirror];
        let mut inserted = 0;
        for u in modes {
            for v in modes {
                let result = pool.intern(Sampler { u, v }, &image);
                assert_eq!(result.is_some(), inserted < SAMPLER_COUNT);
                inserted += 1;
            }
        }
    }
}
