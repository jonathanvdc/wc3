//! External M3A documents bound by their native MODL hashes.
//!
//! Files retain their own envelopes and section identities. Bundle output does
//! not merge animation records or rewrite sequence-controller graphs.
use super::records::{Bone, Character, V0, V1};
use super::{Document, DynamicDocument, Envelope, Md33, Md34, ReadError, WriteError};
use std::collections::{BTreeMap, BTreeSet};

/// A native M3A document using the same envelope and section graph as a model.
pub type AnimationFile<'data> = DynamicDocument<'data>;
impl DynamicDocument<'_> {
    /// MODL single binding hash, or zero when absent from its native revision.
    pub fn binding_hash(&self) -> Result<u32, ReadError> {
        match self {
            Self::Md33(model) => Ok(model.root_record()?.animation_hash()),
            Self::Md34(model) => Ok(model.root_record()?.animation_hash()),
        }
    }
    /// Additional native animation hashes, excluding this file's registered identity.
    fn animation_dependencies(&self, identity: u32) -> Result<Vec<u32>, ReadError> {
        let hashes = match self {
            Self::Md33(model) => model.animation_hashes()?,
            Self::Md34(model) => model.animation_hashes()?,
        };
        Ok(hashes
            .into_iter()
            .filter(|hash| *hash != identity)
            .collect())
    }
    fn skeleton(&self) -> Result<Vec<(u16, Vec<u8>)>, ReadError> {
        match self {
            Self::Md33(model) => skeleton(model),
            Self::Md34(model) => skeleton(model),
        }
    }
    fn check_identity(&self, identity: u32) -> Result<(), ReadError> {
        if identity == 0 {
            return Err(ReadError::invalid(0, "zero animation hash"));
        }
        let native = self.binding_hash()?;
        if native != 0 && native != identity {
            return Err(ReadError::invalid(0, "animation binding hash mismatch"));
        }
        Ok(())
    }
}
impl<'data> From<Document<'data, Md33>> for DynamicDocument<'data> {
    fn from(model: Document<'data, Md33>) -> Self {
        Self::Md33(model)
    }
}
impl<'data> From<Document<'data, Md34>> for DynamicDocument<'data> {
    fn from(model: Document<'data, Md34>) -> Self {
        Self::Md34(model)
    }
}

/// Caller-owned association between a base model and native external animations.
#[derive(Clone, Debug)]
pub struct Bundle<'data, E: Envelope = Md34> {
    /// Base model; edits retain its native envelope and independently versioned records.
    pub model: Document<'data, E>,
    animations: BTreeMap<u32, AnimationFile<'data>>,
}
/// In-memory native outputs keyed by animation hashes rather than guessed filenames.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EncodedBundle {
    /// Encoded base model.
    pub model: Vec<u8>,
    /// Encoded external documents under their verified or caller-supplied binding hashes.
    pub animations: BTreeMap<u32, Vec<u8>>,
}
impl<'data, E: Envelope> Bundle<'data, E> {
    /// Starts a partial bundle without requesting any external files.
    pub fn from_model(model: Document<'data, E>) -> Self {
        Self {
            model,
            animations: BTreeMap::new(),
        }
    }
    /// Registers a native animation; a nonzero authored binding must match the supplied identity.
    ///
    /// A zero authored hash carries no binding evidence, so this explicit
    /// registration supplies the association without inventing a hash algorithm.
    pub fn insert_animation(
        &mut self,
        hash: u32,
        file: AnimationFile<'data>,
    ) -> Result<Option<AnimationFile<'data>>, ReadError> {
        file.check_identity(hash)?;
        Ok(self.animations.insert(hash, file))
    }
    /// Borrows a registered native animation document.
    pub fn animation(&self, hash: u32) -> Option<&AnimationFile<'data>> {
        self.animations.get(&hash)
    }
    /// Edits an external document; complete validation checks its binding again.
    pub fn animation_mut(&mut self, hash: u32) -> Option<&mut AnimationFile<'data>> {
        self.animations.get_mut(&hash)
    }
    /// Iterates registered documents in deterministic hash order.
    pub fn animations(&self) -> impl Iterator<Item = (u32, &AnimationFile<'data>)> {
        self.animations.iter().map(|(hash, file)| (*hash, file))
    }
    /// Declared external animation hashes, deduplicated in deterministic order.
    pub fn dependencies(&self) -> Result<Vec<u32>, ReadError> {
        let mut pending: BTreeSet<_> = self.model.animation_hashes()?.into_iter().collect();
        pending.extend(self.animations.keys().copied());
        let mut visited = BTreeSet::new();
        while let Some(identity) = pending.pop_first() {
            if !visited.insert(identity) {
                continue;
            }
            if let Some(file) = self.animations.get(&identity) {
                for next in file.animation_dependencies(identity)? {
                    if !visited.contains(&next) {
                        pending.insert(next);
                    }
                }
            }
        }
        Ok(visited.into_iter().collect())
    }
    /// Resolves the complete hash closure through caller-owned lookup policy.
    ///
    /// The resolver supplies native bytes; their envelope is detected, their
    /// section revisions are retained, and known authored hashes are verified.
    pub fn read(
        bytes: &'data [u8],
        mut resolve: impl FnMut(u32) -> Result<Option<&'data [u8]>, ReadError>,
    ) -> Result<Self, ReadError> {
        let mut bundle = Self::from_model(Document::read(bytes)?);
        loop {
            let pending: Vec<_> = bundle
                .dependencies()?
                .into_iter()
                .filter(|hash| !bundle.animations.contains_key(hash))
                .collect();
            if pending.is_empty() {
                break;
            }
            for hash in pending {
                if bundle.animations.contains_key(&hash) {
                    continue;
                }
                let bytes = resolve(hash)?
                    .ok_or_else(|| ReadError::invalid(0, "missing external animation"))?;
                bundle.insert_animation(hash, AnimationFile::read(bytes)?)?;
            }
        }
        bundle.validate_complete()?;
        Ok(bundle)
    }
    /// Checks all declared bindings and comparable skeleton hierarchies without merging records.
    pub fn validate_complete(&self) -> Result<(), ReadError> {
        self.model.container().validate()?;
        let base = skeleton(&self.model)?;
        let required = self.dependencies()?;
        for hash in &required {
            if !self.animations.contains_key(hash) {
                return Err(ReadError::invalid(0, "missing external animation"));
            }
        }
        for (hash, file) in &self.animations {
            file.check_identity(*hash)?;
            let bones = file.skeleton()?;
            if !base.is_empty() && !bones.is_empty() && bones != base {
                return Err(ReadError::invalid(
                    0,
                    "external animation skeleton mismatch",
                ));
            }
        }
        Ok(())
    }
    /// Encodes a complete bundle while preserving each file's native section graph.
    pub fn write(&self) -> Result<EncodedBundle, WriteError> {
        self.validate_complete()
            .map_err(|_| WriteError::invalid("animation bundle validation"))?;
        let mut animations = BTreeMap::new();
        for (hash, file) in &self.animations {
            animations.insert(*hash, file.write()?);
        }
        Ok(EncodedBundle {
            model: self.model.write()?,
            animations,
        })
    }
    /// Detaches every source buffer without changing file or section identities.
    pub fn into_owned(self) -> Bundle<'static, E> {
        Bundle {
            model: self.model.into_owned(),
            animations: self
                .animations
                .into_iter()
                .map(|(hash, file)| (hash, file.into_owned()))
                .collect(),
        }
    }
}
fn skeleton<E: Envelope>(model: &Document<'_, E>) -> Result<Vec<(u16, Vec<u8>)>, ReadError> {
    let reference = model.root_record()?.bones();
    if reference.entries == 0 {
        return Ok(Vec::new());
    }
    let id = model
        .container()
        .section_id(reference.index)
        .ok_or_else(|| ReadError::invalid(0, "bone section identity"))?;
    let revision = model
        .container()
        .section(id)
        .ok_or_else(|| ReadError::invalid(0, "bone section identity"))?
        .revision();
    let mut bones = Vec::new();
    macro_rules! read {
        ($record:ty) => {
            for bone in model
                .container()
                .resolve::<$record>(reference.cast())?
                .iter()
            {
                let bone = bone?;
                let name = model.container().resolve::<Character>(bone.name.cast())?;
                bones.push((bone.parent_index, name.as_bytes().to_vec()));
            }
        };
    }
    match revision {
        0 => read!(Bone<V0>),
        1 => read!(Bone<V1>),
        _ => return Err(ReadError::invalid(0, "unsupported bone revision")),
    }
    Ok(bones)
}
