//! Cross-record consistency checks separate from bounded binary decoding.
use super::companions::{DynamicSkeleton, Skeleton, SkeletonChunk};
use super::records::{Bone, Sequence, Validate};
use super::{Core, Dialect, Document, ModelLayout, ReadError, ZeroVec};
use crate::visit_m2_skeleton;

impl<D: Dialect> Document<'_, D> {
    /// Checks native cross-record references without requiring supporting files.
    ///
    /// Binary decoding already checks local byte bounds. This validates animation
    /// aliases, skeleton topology, collision indices, and local lookup tables.
    /// Nonempty SKID animation descriptors must first resolve sequence storage;
    /// dependency enumeration remains available before that context is loaded.
    pub fn validate(&self) -> Result<(), ReadError> {
        if self.requires_sequence_context() {
            return Err(ReadError::invalid(
                0,
                "unresolved skeleton sequence storage",
            ));
        }
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
        validate_bones(&self.bones)?;
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
            (&self.key_bone_lookup, self.bones.len()),
            (&self.attachment_lookup, self.attachments.len()),
            (&self.camera_lookup, self.cameras.len()),
        ] {
            validate_lookup(lookup, count)?;
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
    /// Checks each local table's hierarchy, aliases, lookup indices, and track cardinalities.
    ///
    /// Binary decoding checks byte bounds and retains unresolved animation leaves.
    /// Parent skeletons supply missing tables; bone parents and sequence aliases
    /// address their own local tables. Cross-table references and external
    /// animation streams are not checked here.
    pub fn validate(&self) -> Result<(), ReadError> {
        for chunk in &self.chunks {
            match chunk {
                SkeletonChunk::Bones { bones, lookup } => {
                    validate_bones(bones)?;
                    validate_lookup(lookup, bones.len())?;
                    for bone in bones {
                        bone.validate()?;
                    }
                }
                SkeletonChunk::Attachments {
                    attachments,
                    lookup,
                } => {
                    validate_lookup(lookup, attachments.len())?;
                    for attachment in attachments {
                        attachment.validate()?;
                    }
                }
                SkeletonChunk::Sequences {
                    sequences, lookup, ..
                } => {
                    validate_aliases(sequences)?;
                    validate_lookup(lookup, sequences.len())?;
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

fn validate_lookup(lookup: &ZeroVec<'_, u16>, count: usize) -> Result<(), ReadError> {
    if lookup
        .iter()
        .any(|index| index != u16::MAX && index as usize >= count)
    {
        return Err(ReadError::invalid(0, "lookup index"));
    }
    Ok(())
}

fn validate_bones<V: ModelLayout>(bones: &[Bone<'_, V>]) -> Result<(), ReadError> {
    validate_chains(
        bones.len(),
        |index| match bones[index].parent_bone_id {
            -1 => Ok(None),
            parent if parent < -1 => Err(ReadError::invalid(0, "bone parent sentinel")),
            parent => Ok(Some(parent as usize)),
        },
        "bone parent index",
        "bone parent cycle",
    )
}

fn validate_aliases<V: ModelLayout>(sequences: &[Sequence<V>]) -> Result<(), ReadError> {
    if !sequences.iter().any(|sequence| sequence.flags.alias()) {
        return Ok(());
    }
    validate_chains(
        sequences.len(),
        |index| {
            let sequence = &sequences[index];
            Ok(sequence
                .flags
                .alias()
                .then_some(sequence.alias_next as usize))
        },
        "animation alias index",
        "animation alias cycle",
    )
}

// Each walk has a distinct nonzero marker. Reaching its own marker is a cycle;
// reaching an earlier walk reuses a chain already checked through its endpoint.
fn validate_chains(
    count: usize,
    next: impl Fn(usize) -> Result<Option<usize>, ReadError>,
    invalid_index: &'static str,
    cycle: &'static str,
) -> Result<(), ReadError> {
    let mut visited = vec![0; count];
    for start in 0..count {
        if visited[start] != 0 {
            continue;
        }
        let marker = start + 1;
        let mut current = Some(start);
        while let Some(index) = current {
            let state = visited
                .get_mut(index)
                .ok_or_else(|| ReadError::invalid(0, invalid_index))?;
            if *state == marker {
                return Err(ReadError::invalid(0, cycle));
            }
            if *state != 0 {
                break;
            }
            *state = marker;
            current = next(index)?;
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
