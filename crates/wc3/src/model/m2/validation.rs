//! Cross-record consistency checks separate from bounded binary decoding.
use super::companions::{DynamicSkeleton, Skeleton, SkeletonChunk};
use super::records::{Sequence, Validate};
use super::{Core, Dialect, Model, ModelLayout, ReadError};
use crate::visit_m2_skeleton;
use std::collections::BTreeSet;

impl<D: Dialect> Model<'_, D> {
    /// Checks native cross-record references without requiring supporting files.
    ///
    /// Binary decoding already checks local byte bounds. This validates animation
    /// aliases, skeleton topology, collision indices, and local lookup tables.
    pub fn validate(&self) -> Result<(), ReadError> {
        if self.cores().count() != 1 {
            return Err(ReadError::invalid(0, "multiple MD21 cores"));
        }
        self.core().validate()
    }
}
impl<V: ModelLayout> Core<'_, V> {
    /// Checks locally available record relationships.
    pub fn validate(&self) -> Result<(), ReadError> {
        validate_aliases(&self.sequences)?;
        for (index, _) in self.bones.iter().enumerate() {
            let mut current = index;
            let mut seen = BTreeSet::new();
            loop {
                if !seen.insert(current) {
                    return Err(ReadError::invalid(0, "bone parent cycle"));
                }
                let bone = self
                    .bones
                    .get(current)
                    .ok_or_else(|| ReadError::invalid(0, "bone parent index"))?;
                if bone.parent_bone_id == -1 {
                    break;
                }
                if bone.parent_bone_id < -1 {
                    return Err(ReadError::invalid(0, "bone parent sentinel"));
                }
                current = bone.parent_bone_id as usize;
            }
        }
        if !self.collision_indices.len().is_multiple_of(3) {
            return Err(ReadError::invalid(0, "collision triangle index count"));
        }
        if self
            .collision_indices
            .iter()
            .any(|index| index as usize >= self.collision_vertices.len())
        {
            return Err(ReadError::invalid(0, "collision vertex index"));
        }
        for (lookup, count) in [
            (&self.sequence_lookup, self.sequences.len()),
            (&self.attachment_lookup, self.attachments.len()),
            (&self.camera_lookup, self.cameras.len()),
        ] {
            if lookup
                .iter()
                .any(|index| index != u16::MAX && index as usize >= count)
            {
                return Err(ReadError::invalid(0, "lookup index"));
            }
        }
        macro_rules! validate { ($($field:ident),* $(,)?) => { $(for record in &self.$field { record.validate()?; })* }; }
        validate!(
            sequences,
            bones,
            colors,
            texture_weights,
            texture_transforms,
            attachments,
            events,
            lights,
            cameras,
            ribbons,
            particles
        );
        self.texture_flipbooks.validate()?;
        self.playable_animation_lookup.validate()?;
        self.texture_combiner_combos.validate()?;
        for skin in self.embedded_skins() {
            skin.validate()?;
        }
        Ok(())
    }
}

impl<V: ModelLayout> Skeleton<'_, V> {
    /// Checks the native track cardinalities and sequence records of this skeleton.
    pub fn validate(&self) -> Result<(), ReadError> {
        for chunk in &self.chunks {
            match chunk {
                SkeletonChunk::Bones { bones, .. } => {
                    for bone in bones {
                        bone.validate()?;
                    }
                }
                SkeletonChunk::Attachments { attachments, .. } => {
                    for attachment in attachments {
                        attachment.validate()?;
                    }
                }
                SkeletonChunk::Sequences { sequences, .. } => {
                    validate_aliases(sequences)?;
                    for sequence in sequences {
                        sequence.validate()?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn validate_aliases<V: ModelLayout>(sequences: &[Sequence<V>]) -> Result<(), ReadError> {
    for (index, sequence) in sequences.iter().enumerate() {
        if sequence.flags.alias() {
            let mut current = index;
            let mut seen = BTreeSet::new();
            loop {
                if !seen.insert(current) {
                    return Err(ReadError::invalid(0, "animation alias cycle"));
                }
                let sequence = sequences
                    .get(current)
                    .ok_or_else(|| ReadError::invalid(0, "animation alias index"))?;
                if !sequence.flags.alias() {
                    break;
                }
                current = sequence.alias_next as usize;
            }
        }
    }
    Ok(())
}

impl DynamicSkeleton<'_> {
    /// Validates native skeleton records using their retained layout.
    pub fn validate(&self) -> Result<(), ReadError> {
        visit_m2_skeleton!(self, |file| file.validate())
    }
}
