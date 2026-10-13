//! Connected model documents and coordinated external-animation encoding.
use super::chunks::Chunk;
use super::codec::{Array, ArrayRef, Decoder, Encoder, Value};
use super::companions::{
    AnimationFile, AnimationStream, BoneFile, CompanionLayout, DynamicSkeleton, DynamicSkin,
    GeometryFile, LayoutKind, PhysicsFile, Skeleton, SkeletonChunk, Skin, SkinProfile,
    SupportsExternalSkins,
};
use super::records::{
    ArrayVisitor, Bone, Sequence, TimelineStorage, Track, ValuesLeaf, ValuesStorage, VisitArrays,
};
use super::records::{Interpolation, ModelFlags, SequenceFlags};
use super::resources::{Dependency, DependencyKind, FileKey};
use super::{Core, Dialect, DynamicModel, Model, ModelLayout, ReadError, WriteError};
use crate::model::common::{binary::Chunks, Ownership};
use crate::model::common::{Borrowed, IntoZeroVec};
use std::collections::{BTreeMap, BTreeSet};
use zerovec::ZeroVec;

/// A supporting document retaining its independent file representation.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum File<'data> {
    /// A standalone skin profile.
    Skin(Box<DynamicSkin<'data>>),
    /// A modern chunked geometry profile referenced in place of a classic skin.
    Geometry(GeometryFile<'data>),
    /// Raw or chunked external animation data.
    Animation(AnimationFile<'data>),
    /// A shared skeleton with explicit parent references.
    Skeleton(DynamicSkeleton<'data>),
    /// Sparse bone override matrices.
    Bone(BoneFile<'data>),
    /// Standalone physics data.
    Physics(PhysicsFile<'data>),
    /// A recursively referenced geometry or particle model.
    Model(Box<DynamicModel<'data>>),
}
/// A native bone table together with the identity of its owning document.
pub type BoneTable<'view, 'data, V> = (&'view FileKey, &'view [Bone<'data, V>]);

/// A native geometry profile, whether embedded or stored in a supporting file.
#[derive(Clone, Copy, Debug)]
pub enum ProfileRef<'view, 'data, V: ModelLayout> {
    /// An original embedded profile in a pre-Wrath core.
    Embedded(&'view SkinProfile<'data, V>),
    /// A standalone SKIN profile.
    Skin(&'view Skin<'data, V>),
    /// A modern chunked geometry document.
    Geometry(&'view GeometryFile<'data>),
}
/// Encoded supporting documents identified by their existing logical identities.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EncodedBundle {
    /// Encoded root model.
    pub model: Vec<u8>,
    /// Supporting outputs; writing or packaging them remains caller-controlled.
    pub files: BTreeMap<FileKey, Vec<u8>>,
}
/// Native documents connected through model references.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Bundle<'data, D: Dialect> {
    /// Root model document.
    pub model: Model<'data, D>,
    /// Logical identity used to derive sibling filenames.
    pub key: FileKey,
    files: BTreeMap<FileKey, File<'data>>,
}
/// A resolved track retaining encoded values and spline tangents.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedTrack<'view, T: Value>
where
    [T; 3]: Value,
{
    /// Track interpolation code without rendering interpretation.
    pub interpolation: Interpolation,
    /// Global sequence index, or 0xffff for an ordinary sequence.
    pub global_sequence: u16,
    /// Key times in the native clock (legacy timelines are not rebased).
    pub timestamps: ZeroVec<'view, u32>,
    /// Native values or spline triplets.
    pub values: ResolvedValues<'view, T>,
}
/// A resolved sequence's values in their native wire representation.
#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedValues<'view, T: Value>
where
    [T; 3]: Value,
{
    /// Ordinary key values.
    Values(ZeroVec<'view, T>),
    /// Value, incoming tangent, and outgoing tangent for each key.
    Splines(ZeroVec<'view, [T; 3]>),
}
impl<'data, D: Dialect> Bundle<'data, D> {
    /// Starts a partial bundle without loading any supporting files.
    pub fn from_model(model: Model<'data, D>) -> Self {
        Self {
            model,
            key: FileKey::Root,
            files: BTreeMap::new(),
        }
    }
    /// Sets the root identity used for sibling path references.
    pub fn with_key(mut self, key: FileKey) -> Self {
        self.key = key;
        self
    }
    /// Registers or replaces a supporting document under its referenced identity.
    pub fn insert(&mut self, key: FileKey, file: File<'data>) -> Option<File<'data>> {
        self.files.insert(key, file)
    }
    /// Finds a native supporting document.
    pub fn file(&self, key: &FileKey) -> Option<&File<'data>> {
        self.files.get(key)
    }
    /// Finds a native supporting document for editing.
    pub fn file_mut(&mut self, key: &FileKey) -> Option<&mut File<'data>> {
        self.files.get_mut(key)
    }
    /// Iterates supporting documents in deterministic identity order.
    pub fn files(&self) -> impl Iterator<Item = (&FileKey, &File<'data>)> {
        self.files.iter()
    }
    /// Enumerates declared dependencies without accessing their sources.
    pub fn dependencies(&self) -> Vec<Dependency> {
        let mut result = self.model.dependencies(&self.key);
        for (key, file) in &self.files {
            match file {
                File::Skeleton(skeleton) => result.extend(skeleton.dependencies(key)),
                File::Model(model) => result.extend(model.dependencies(key)),
                _ => {}
            }
        }
        result
    }
    /// Loads the complete supporting-document closure through a caller-owned resolver.
    ///
    /// Texture bytes are not required to understand model documents. Every other
    /// declared file must be supplied. Buffers returned by the resolver are borrowed.
    pub fn decode(
        data: &'data [u8],
        key: FileKey,
        mut resolve: impl FnMut(&Dependency) -> Result<Option<&'data [u8]>, ReadError>,
    ) -> Result<Self, ReadError> {
        let mut bundle = Self::from_model(Model::decode_m2(data)?).with_key(key);
        let mut attempted = BTreeSet::new();
        loop {
            let dependencies = bundle.dependencies();
            let pending: Vec<_> = dependencies
                .into_iter()
                .filter(|dependency| {
                    !matches!(dependency.kind, DependencyKind::Texture)
                        && !bundle.files.contains_key(&dependency.key)
                        && !attempted.contains(&dependency.key)
                })
                .collect();
            if pending.is_empty() {
                break;
            }
            for dependency in pending {
                if bundle.files.contains_key(&dependency.key) {
                    continue;
                }
                attempted.insert(dependency.key.clone());
                let data = resolve(&dependency)?
                    .ok_or_else(|| ReadError::invalid(0, "missing supporting document"))?;
                let (layout, flags, external) = bundle.owner_context(&dependency.owner);
                let file = match dependency.kind {
                    DependencyKind::Skin { .. } => {
                        if data.starts_with(b"SKIN") {
                            match layout.read_skin(data) {
                                Ok(skin) => File::Skin(Box::new(skin)),
                                Err(skin_error) => {
                                    let geometry =
                                        GeometryFile::read(data).map_err(|_| skin_error)?;
                                    File::Geometry(geometry)
                                }
                            }
                        } else {
                            File::Geometry(GeometryFile::read(data)?)
                        }
                    }
                    DependencyKind::Animation { .. } => File::Animation(AnimationFile::read(
                        data,
                        matches!(bundle.files.get(&dependency.owner), Some(File::Skeleton(_)))
                            || flags.chunked_animations(),
                    )?),
                    DependencyKind::Skeleton => {
                        bundle.load_skeleton(
                            &dependency.key,
                            data,
                            (layout, flags),
                            &external,
                            &mut BTreeSet::new(),
                            &mut resolve,
                        )?;
                        continue;
                    }
                    DependencyKind::Bone => File::Bone(BoneFile::read(data)?),
                    DependencyKind::Physics => File::Physics(PhysicsFile::read(data)?),
                    DependencyKind::Model => File::Model(Box::new(DynamicModel::decode_m2(data)?)),
                    DependencyKind::Texture => unreachable!(),
                };
                bundle.files.insert(dependency.key, file);
            }
        }
        bundle.validate_complete()?;
        Ok(bundle)
    }
    fn load_skeleton(
        &mut self,
        key: &FileKey,
        data: &'data [u8],
        layout: (LayoutKind, ModelFlags),
        inherited: &[bool],
        active: &mut BTreeSet<FileKey>,
        resolve: &mut impl FnMut(&Dependency) -> Result<Option<&'data [u8]>, ReadError>,
    ) -> Result<(), ReadError> {
        let (kind, flags) = layout;
        let version = kind.version();
        if !active.insert(key.clone()) {
            return Err(ReadError::invalid(0, "skeleton parent cycle"));
        }
        let mut external = inherited.to_vec();
        for chunk in Chunks::new(data, 0) {
            let chunk =
                chunk.map_err(|error| ReadError::invalid(error.offset, "skeleton framing"))?;
            if chunk.header.tag == *b"SKPD" {
                let mut input = Decoder::new(chunk.data, version, flags);
                input.bytes(8)?;
                let parent_id = input.u32()?;
                if parent_id == 0 {
                    continue;
                }
                let parent = FileKey::FileId(parent_id);
                let dependency = Dependency {
                    owner: key.clone(),
                    key: parent.clone(),
                    kind: DependencyKind::Skeleton,
                };
                if !self.files.contains_key(&parent) {
                    let source = resolve(&dependency)?.ok_or_else(|| {
                        ReadError::invalid(chunk.offset + 8, "missing parent skeleton")
                    })?;
                    self.load_skeleton(&parent, source, layout, inherited, active, resolve)?;
                }
                if let Some(File::Skeleton(skeleton)) = self.files.get(&parent) {
                    if let Some(sequences) = skeleton_sequences(skeleton) {
                        external = sequences
                            .iter()
                            .map(|sequence| sequence.flags.external_animation())
                            .collect();
                    }
                } else {
                    return Err(ReadError::invalid(
                        chunk.offset,
                        "parent skeleton document kind",
                    ));
                }
            }
        }
        let skeleton = kind.read_skeleton(data, flags, &external)?;
        self.files.insert(key.clone(), File::Skeleton(skeleton));
        active.remove(key);
        Ok(())
    }
    fn owner_context(&self, owner: &FileKey) -> (LayoutKind, ModelFlags, Vec<bool>) {
        if let Some(File::Model(model)) = self.files.get(owner) {
            return crate::visit_m2_model!(&**model, |typed| model_context(typed));
        }
        if let Some(File::Skeleton(skeleton)) = self.files.get(owner) {
            return (
                skeleton.layout(),
                skeleton.flags(),
                self.sequences_for(owner)
                    .iter()
                    .map(|s| s.flags.external_animation())
                    .collect(),
            );
        }
        model_context(&self.model)
    }
    /// Checks that all required documents exist and match their declared kinds.
    pub fn validate_complete(&self) -> Result<(), ReadError> {
        self.model.validate()?;
        for extension in self.model.extensions() {
            if let Chunk::Physics(physics) = extension {
                physics.validate()?;
            }
        }
        for dependency in self.dependencies() {
            if matches!(dependency.kind, DependencyKind::Texture) {
                continue;
            }
            let file = self
                .files
                .get(&dependency.key)
                .ok_or_else(|| ReadError::invalid(0, "missing supporting document"))?;
            let matches = matches!(
                (&dependency.kind, file),
                (
                    DependencyKind::Skin { .. },
                    File::Skin(_) | File::Geometry(_)
                ) | (DependencyKind::Animation { .. }, File::Animation(_))
                    | (DependencyKind::Skeleton, File::Skeleton(_))
                    | (DependencyKind::Bone, File::Bone(_))
                    | (DependencyKind::Physics, File::Physics(_))
                    | (DependencyKind::Model, File::Model(_))
            );
            let layout = self.owner_context(&dependency.owner).0;
            if matches!(file,File::Skin(skin) if skin.layout()!=layout)
                || matches!(file,File::Skeleton(skeleton) if skeleton.layout()!=layout)
            {
                return Err(ReadError::invalid(0, "supporting document owner layout"));
            }
            if !matches {
                return Err(ReadError::invalid(0, "supporting document kind"));
            }
            if let (DependencyKind::Animation { .. }, File::Animation(animation)) =
                (&dependency.kind, file)
            {
                let chunked = matches!(self.files.get(&dependency.owner), Some(File::Skeleton(_)))
                    || self.owner_context(&dependency.owner).1.chunked_animations();
                if chunked != matches!(animation, AnimationFile::Chunked(_)) {
                    return Err(ReadError::invalid(0, "animation stream framing"));
                }
            }
        }
        for (key, file) in &self.files {
            if let File::Physics(physics) = file {
                physics.validate()?;
            }
            if let File::Model(model) = file {
                model.validate()?;
            }
            if let File::Skeleton(skeleton) = file {
                skeleton.validate()?;
                let mut path = BTreeSet::new();
                let mut current = Some((key, skeleton));
                while let Some((key, skeleton)) = current {
                    if !path.insert(key.clone()) {
                        return Err(ReadError::invalid(0, "skeleton parent cycle"));
                    }
                    current = skeleton_parent(skeleton)
                        .and_then(|id| self.files.get_key_value(&FileKey::FileId(id)))
                        .and_then(|(key, file)| match file {
                            File::Skeleton(skeleton) => Some((key, skeleton)),
                            _ => None,
                        });
                }
            }
        }
        // Resolving every external leaf also validates its referenced stream bounds.
        self.relocated()
            .map(|_| ())
            .map_err(|_| ReadError::invalid(0, "external animation reference"))
    }
    /// Accesses an embedded, standalone, or modern chunked geometry profile.
    pub fn profile(
        &self,
        index: usize,
        lod: bool,
    ) -> Result<ProfileRef<'_, 'data, D::Layout>, ReadError> {
        if !lod {
            if let Some(profile) = self.model.core().embedded_skins().get(index) {
                return Ok(ProfileRef::Embedded(profile));
            }
        }
        let dependency = self.model.dependencies(&self.key).into_iter().find(|dependency| matches!(dependency.kind, DependencyKind::Skin { index: i, lod: l } if i == index && l == lod)).ok_or_else(|| ReadError::invalid(0, "skin profile index"))?;
        match self.files.get(&dependency.key) {
            Some(File::Skin(skin)) => skin
                .as_typed::<D::Layout>()
                .map(ProfileRef::Skin)
                .ok_or_else(|| ReadError::invalid(0, "skin owner layout")),
            Some(File::Geometry(geometry)) => Ok(ProfileRef::Geometry(geometry)),
            _ => Err(ReadError::invalid(0, "unloaded geometry profile")),
        }
    }
    /// Finds a loaded skin profile at the declared base/LOD index.
    pub fn skin(&self, index: usize, lod: bool) -> Result<&Skin<'data, D::Layout>, ReadError> {
        let dependency = self.model.dependencies(&self.key).into_iter().find(|dependency| matches!(dependency.kind, DependencyKind::Skin { index: i, lod: l } if i == index && l == lod)).ok_or_else(|| ReadError::invalid(0, "skin profile index"))?;
        match self.files.get(&dependency.key) {
            Some(File::Skin(skin)) => skin
                .as_typed::<D::Layout>()
                .ok_or_else(|| ReadError::invalid(0, "skin owner layout")),
            Some(File::Geometry(_)) => Err(ReadError::invalid(0, "profile uses chunked geometry")),
            _ => Err(ReadError::invalid(0, "unloaded skin profile")),
        }
    }
    /// Finds a modern geometry document at a declared base or LOD skin index.
    pub fn geometry(&self, index: usize, lod: bool) -> Result<&GeometryFile<'data>, ReadError> {
        let dependency = self.model.dependencies(&self.key).into_iter().find(|dependency| matches!(dependency.kind, DependencyKind::Skin { index: i, lod: l } if i == index && l == lod)).ok_or_else(|| ReadError::invalid(0, "geometry profile index"))?;
        match self.files.get(&dependency.key) {
            Some(File::Geometry(geometry)) => Ok(geometry),
            _ => Err(ReadError::invalid(0, "unloaded geometry profile")),
        }
    }
    /// Resolves native track leaves from the model or a selected skeleton owner.
    pub fn resolve_track<'view, T: Value, L: ModelLayout>(
        &'view self,
        track: &'view Track<'data, T, L>,
        owner: &FileKey,
        stream: [u8; 4],
        sequence: usize,
    ) -> Result<ResolvedTrack<'view, T>, ReadError>
    where
        [T; 3]: Value,
    {
        if self.owner_context(owner).0 != L::KIND {
            return Err(ReadError::invalid(0, "track owner layout"));
        }
        if self.owner_context(owner).0.version() >= 264
            && track.base.global_sequence == u16::MAX
            && sequence >= self.sequences_for(owner).len()
        {
            return Err(ReadError::invalid(0, "animation sequence index"));
        }
        let index = if self.owner_context(owner).0.version() < 264
            || track.base.global_sequence != u16::MAX
        {
            0
        } else {
            sequence
        };
        let timestamps = match track.base.timeline.leaf(index) {
            Some(values) => self.resolve_leaf(values, owner, stream, sequence)?,
            None => ZeroVec::new(),
        };
        let values = match track.values.leaf(index) {
            Some(ValuesLeaf::Values(values)) => {
                ResolvedValues::Values(self.resolve_leaf(values, owner, stream, sequence)?)
            }
            Some(ValuesLeaf::Splines(values)) => {
                ResolvedValues::Splines(self.resolve_leaf(values, owner, stream, sequence)?)
            }
            None if track.values.is_spline() => ResolvedValues::Splines(ZeroVec::new()),
            None => ResolvedValues::Values(ZeroVec::new()),
        };
        let value_count = match &values {
            ResolvedValues::Values(values) => values.len(),
            ResolvedValues::Splines(values) => values.len(),
        };
        if timestamps.len() != value_count {
            return Err(ReadError::invalid(0, "timestamp/value count mismatch"));
        }
        Ok(ResolvedTrack {
            interpolation: track.base.interpolation,
            global_sequence: track.base.global_sequence,
            timestamps,
            values,
        })
    }
    fn resolve_leaf<'view, T: Value>(
        &'view self,
        values: Array<'view, T>,
        owner: &FileKey,
        stream: [u8; 4],
        sequence: usize,
    ) -> Result<ZeroVec<'view, T>, ReadError> {
        match values {
            Array::Inline(values) => Ok(values),
            Array::External(reference) if reference.count == 0 => Ok(ZeroVec::new()),
            Array::External(reference) => {
                let key = animation_key(
                    &self.dependencies(),
                    &self.files,
                    owner,
                    &self.sequences_for(owner),
                    sequence,
                )
                .ok_or_else(|| ReadError::invalid(0, "animation file reference"))?;
                let file = match self.files.get(&key) {
                    Some(File::Animation(file)) => file,
                    _ => return Err(ReadError::invalid(0, "unloaded animation file")),
                };
                let bytes = file
                    .stream(stream)
                    .ok_or_else(|| ReadError::invalid(0, "missing animation stream"))?;
                Decoder::new(bytes, 0, ModelFlags::default()).array(reference)
            }
        }
    }
    fn sequences_for(&self, owner: &FileKey) -> Vec<SequenceKey> {
        if let Some(File::Model(model)) = self.files.get(owner) {
            return crate::visit_m2_model!(&**model, |typed| sequence_keys(
                &typed.core().sequences
            ));
        }
        let mut current = self.files.get_key_value(owner);
        let mut visited = BTreeSet::new();
        while let Some((key, File::Skeleton(skeleton))) = current {
            if !visited.insert(key) {
                break;
            }
            if let Some(sequences) = skeleton_sequences(skeleton) {
                return sequences;
            }
            current = skeleton_parent(skeleton)
                .and_then(|id| self.files.get_key_value(&FileKey::FileId(id)));
        }
        sequence_keys(&self.model.core().sequences)
    }
    /// Returns the nearest skeleton's bone records, falling back to the core.
    pub fn bones(&self) -> Result<BoneTable<'_, 'data, D::Layout>, ReadError> {
        let key = self.model.extensions().find_map(|chunk| match chunk {
            Chunk::SkeletonId(id) if *id != 0 => Some(FileKey::FileId(*id)),
            _ => None,
        });
        let mut current = key.as_ref().and_then(|key| self.files.get_key_value(key));
        let mut visited = BTreeSet::new();
        while let Some((key, file)) = current {
            if !visited.insert(key.clone()) {
                return Err(ReadError::invalid(0, "skeleton parent cycle"));
            }
            let skeleton = match file {
                File::Skeleton(skeleton) => skeleton,
                _ => return Err(ReadError::invalid(0, "skeleton document kind")),
            };
            let native = skeleton
                .as_typed::<D::Layout>()
                .ok_or_else(|| ReadError::invalid(0, "skeleton owner layout"))?;
            if let Some(bones) = native.chunks.iter().find_map(|chunk| match chunk {
                SkeletonChunk::Bones { bones, .. } => Some(bones.as_slice()),
                _ => None,
            }) {
                return Ok((key, bones));
            }
            current = skeleton_parent(skeleton)
                .and_then(|id| self.files.get_key_value(&FileKey::FileId(id)));
        }
        if key.is_some() {
            return Err(ReadError::invalid(0, "unloaded skeleton bones"));
        }
        Ok((&self.key, &self.model.core().bones))
    }
    /// Resolves one bone translation track using its owning animation stream.
    pub fn bone_translation(
        &self,
        sequence: usize,
        bone: usize,
    ) -> Result<ResolvedTrack<'_, [f32; 3]>, ReadError> {
        let (owner, bones) = self.bones()?;
        let bone = bones
            .get(bone)
            .ok_or_else(|| ReadError::invalid(0, "bone index"))?;
        self.resolve_track(
            &bone.translation,
            owner,
            if *owner == self.key {
                *b"AFM2"
            } else {
                *b"AFSB"
            },
            sequence,
        )
    }
    /// Replaces a bone translation sequence while retaining its native owner.
    ///
    /// The new buffers are stored inline in the editable representation. Bundle
    /// encoding places them into an external stream when the sequence flags require
    /// it. Spline tracks require the native spline values variant.
    pub fn set_bone_translation(
        &mut self,
        sequence: usize,
        bone: usize,
        timestamps: impl IntoZeroVec<'data, u32>,
        values: ResolvedValues<'data, [f32; 3]>,
    ) -> Result<(), ReadError> {
        let owner = self.bones()?.0.clone();
        let version = self.owner_context(&owner).0.version();
        if version < 264 {
            return Err(ReadError::invalid(
                0,
                "legacy timelines require native track editing",
            ));
        }
        let sequence_count = self.sequences_for(&owner).len();
        if sequence >= sequence_count {
            return Err(ReadError::invalid(0, "animation sequence index"));
        }
        let bones = if owner == self.key {
            &mut self.model.core_mut().bones
        } else {
            let skeleton = match self.files.get_mut(&owner) {
                Some(File::Skeleton(skeleton)) => skeleton,
                _ => return Err(ReadError::invalid(0, "skeleton owner")),
            };
            skeleton
                .as_typed_mut::<D::Layout>()
                .ok_or_else(|| ReadError::invalid(0, "skeleton owner layout"))?
                .chunks
                .iter_mut()
                .find_map(|chunk| match chunk {
                    SkeletonChunk::Bones { bones, .. } => Some(bones),
                    _ => None,
                })
                .ok_or_else(|| ReadError::invalid(0, "skeleton bones"))?
        };
        let track = &mut bones
            .get_mut(bone)
            .ok_or_else(|| ReadError::invalid(0, "bone index"))?
            .translation;
        let index = if version < 264 || track.base.global_sequence != u16::MAX {
            0
        } else {
            sequence
        };
        let timestamps = timestamps.into_zero_vec();
        if timestamps
            .iter()
            .zip(timestamps.iter().skip(1))
            .any(|(a, b)| a > b)
        {
            return Err(ReadError::invalid(0, "timestamp ordering"));
        }
        if track.values.is_spline() != matches!(values, ResolvedValues::Splines(_)) {
            return Err(ReadError::invalid(0, "track value representation"));
        }
        let count = match &values {
            ResolvedValues::Values(values) => values.len(),
            ResolvedValues::Splines(values) => values.len(),
        };
        if timestamps.len() != count {
            return Err(ReadError::invalid(0, "timestamp/value count mismatch"));
        }
        if track.base.interpolation.spline() != matches!(values, ResolvedValues::Splines(_)) {
            return Err(ReadError::invalid(0, "spline interpolation values"));
        }
        track
            .base
            .timeline
            .replace(index, Array::Inline(timestamps))?;
        track.values.replace(
            index,
            match values {
                ResolvedValues::Values(values) => ValuesLeaf::Values(Array::Inline(values)),
                ResolvedValues::Splines(values) => ValuesLeaf::Splines(Array::Inline(values)),
            },
        )?;
        Ok(())
    }
    /// Coordinates all external animation offsets and returns in-memory outputs.
    pub fn encode(&self) -> Result<EncodedBundle, WriteError> {
        self.validate_complete()
            .map_err(|_| WriteError::invalid("bundle validation"))?;
        let (model, files) = self.relocated()?;
        let mut output = EncodedBundle {
            model: model.encode_m2()?,
            files: BTreeMap::new(),
        };
        for (key, file) in files {
            let bytes = match file {
                File::Skin(file) => file.write()?,
                File::Geometry(file) => file.write()?,
                File::Animation(file) => file.write()?,
                File::Skeleton(file) => file.write()?,
                File::Bone(file) => file.write()?,
                File::Physics(file) => file.write()?,
                File::Model(file) => file.encode_m2()?,
            };
            output.files.insert(key, bytes);
        }
        Ok(output)
    }
    fn relocated(&self) -> Result<(Model<'_, D>, BTreeMap<FileKey, File<'_>>), WriteError> {
        let mut model = self.model.as_borrowed();
        let mut files = self.files.as_borrowed();
        let dependencies = self.dependencies();
        let mut streams = BTreeMap::new();
        {
            let sequences = sequence_keys(&self.model.core().sequences);
            let mut relocate = ExternalRelocator {
                owner: &self.key,
                sequences: &sequences,
                files: &self.files,
                dependencies: &dependencies,
                streams: &mut streams,
                stream: *b"AFM2",
                version: D::VERSION,
            };
            visit_core(model.core_mut(), &mut relocate)?;
            visit_extensions(&mut model, &mut relocate)?;
        }
        for (key, file) in &mut files {
            match file {
                File::Skeleton(skeleton) => {
                    let sequences = self.sequences_for(key);
                    let mut relocate = ExternalRelocator {
                        owner: key,
                        sequences: &sequences,
                        files: &self.files,
                        dependencies: &dependencies,
                        streams: &mut streams,
                        stream: *b"AFSB",
                        version: skeleton.layout().version(),
                    };
                    crate::visit_m2_skeleton!(skeleton, |native| visit_skeleton(
                        native,
                        &mut relocate
                    ))?;
                }
                File::Model(model) => {
                    let sequences = self.sequences_for(key);
                    let mut relocate = ExternalRelocator {
                        owner: key,
                        sequences: &sequences,
                        files: &self.files,
                        dependencies: &dependencies,
                        streams: &mut streams,
                        stream: *b"AFM2",
                        version: model.version(),
                    };
                    crate::visit_m2_model!(&mut **model, |typed| {
                        visit_core(typed.core_mut(), &mut relocate)?;
                        visit_extensions(typed, &mut relocate)
                    })?;
                }
                _ => {}
            }
        }
        for ((key, tag), encoder) in streams {
            let bytes = encoder.finish();
            let file = files
                .get_mut(&key)
                .ok_or_else(|| WriteError::invalid("missing animation output"))?;
            match file {
                File::Animation(AnimationFile::Raw(data)) if tag == *b"AFM2" => {
                    *data = bytes.into()
                }
                File::Animation(AnimationFile::Chunked(chunks)) => {
                    if let Some(chunk) = chunks.iter_mut().find(|chunk| chunk.tag == tag) {
                        chunk.data = bytes.into();
                    } else {
                        chunks.push(AnimationStream {
                            tag,
                            data: bytes.into(),
                        });
                    }
                }
                _ => return Err(WriteError::invalid("animation output stream kind")),
            }
        }
        Ok((model, files))
    }
}
#[derive(Clone, Copy)]
struct SequenceKey {
    id: u16,
    variation_index: u16,
    flags: SequenceFlags,
    alias_next: u16,
}
fn sequence_keys<V: ModelLayout>(sequences: &[Sequence<V>]) -> Vec<SequenceKey> {
    sequences
        .iter()
        .map(|s| SequenceKey {
            id: s.id,
            variation_index: s.variation_index,
            flags: s.flags,
            alias_next: s.alias_next,
        })
        .collect()
}
fn model_context<D: Dialect>(model: &Model<'_, D>) -> (LayoutKind, ModelFlags, Vec<bool>) {
    (
        D::Layout::KIND,
        model.core().flags,
        model
            .core()
            .sequences
            .iter()
            .map(|s| s.flags.external_animation())
            .collect(),
    )
}
fn skeleton_sequences(skeleton: &DynamicSkeleton<'_>) -> Option<Vec<SequenceKey>> {
    crate::visit_m2_skeleton!(skeleton, |native| native.chunks.iter().find_map(
        |chunk| match chunk {
            SkeletonChunk::Sequences { sequences, .. } => Some(sequence_keys(sequences)),
            _ => None,
        }
    ))
}
fn skeleton_parent(skeleton: &DynamicSkeleton<'_>) -> Option<u32> {
    crate::visit_m2_skeleton!(skeleton, |native| native.chunks.iter().find_map(
        |chunk| match chunk {
            SkeletonChunk::Parent { file_id, .. } if *file_id != 0 => Some(*file_id),
            _ => None,
        }
    ))
}
fn visit_skeleton<'data, 'source, L: ModelLayout>(
    skeleton: &mut Skeleton<'data, L>,
    visitor: &mut ExternalRelocator<'_, 'source>,
) -> Result<(), WriteError> {
    for chunk in &mut skeleton.chunks {
        match chunk {
            SkeletonChunk::Bones { bones, .. } => {
                visitor.stream = *b"AFSB";
                bones.visit_arrays(visitor)?;
            }
            SkeletonChunk::Attachments { attachments, .. } => {
                visitor.stream = *b"AFSA";
                attachments.visit_arrays(visitor)?;
            }
            _ => {}
        }
    }
    Ok(())
}
fn animation_key(
    dependencies: &[Dependency],
    files: &BTreeMap<FileKey, File<'_>>,
    owner: &FileKey,
    sequences: &[SequenceKey],
    sequence: usize,
) -> Option<FileKey> {
    let mut index = sequence;
    let mut seen = BTreeSet::new();
    let (animation_id, variation) = loop {
        if !seen.insert(index) {
            return None;
        }
        let animation = sequences.get(index)?;
        if animation.flags.alias() {
            index = animation.alias_next as usize;
            continue;
        }
        break (animation.id, animation.variation_index);
    };
    let mut current = owner.clone();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return None;
        }
        if let Some(dependency) = dependencies.iter().find(|dependency| dependency.owner == current && matches!(dependency.kind, DependencyKind::Animation { animation_id: id, variation_id } if id == animation_id && variation_id == variation)) {
            return Some(dependency.key.clone());
        }
        current = match files.get(&current) {
            Some(File::Skeleton(skeleton)) => FileKey::FileId(skeleton_parent(skeleton)?),
            _ => return None,
        };
    }
}

struct ExternalRelocator<'view, 'data> {
    owner: &'view FileKey,
    sequences: &'view [SequenceKey],
    files: &'view BTreeMap<FileKey, File<'data>>,
    dependencies: &'view [Dependency],
    streams: &'view mut BTreeMap<(FileKey, [u8; 4]), Encoder>,
    stream: [u8; 4],
    version: u32,
}
impl<'data, 'target> ArrayVisitor<'target> for ExternalRelocator<'_, 'data> {
    type Error = WriteError;
    fn visit<T: Value>(
        &mut self,
        sequence: usize,
        global_sequence: u16,
        values: &mut Array<'target, T>,
    ) -> Result<(), WriteError> {
        let external_sequence = self.version >= 264
            && global_sequence == u16::MAX
            && self
                .sequences
                .get(sequence)
                .is_some_and(|sequence| sequence.flags.external_animation());
        if !external_sequence && matches!(values, Array::Inline(_)) {
            return Ok(());
        }
        let count = match values {
            Array::Inline(values) => values.len(),
            Array::External(reference) => reference.count as usize,
        };
        if count == 0 {
            if matches!(values, Array::External(_)) {
                *values = Array::External(ArrayRef::default());
            }
            return Ok(());
        }
        let key = animation_key(
            self.dependencies,
            self.files,
            self.owner,
            self.sequences,
            sequence,
        )
        .ok_or_else(|| WriteError::invalid("animation identity"))?;
        let file = match self.files.get(&key) {
            Some(File::Animation(file)) => file,
            _ => return Err(WriteError::invalid("missing external animation document")),
        };
        let leaf = match values {
            Array::Inline(values) => values.as_slice().as_zerovec(),
            Array::External(reference) => {
                let stream = file
                    .stream(self.stream)
                    .ok_or_else(|| WriteError::invalid("missing external animation stream"))?;
                Decoder::new(stream, 0, ModelFlags::default())
                    .array(*reference)
                    .map_err(|_| WriteError::invalid("external animation bounds"))?
            }
        };
        let target_stream = if matches!(file, AnimationFile::Raw(_)) {
            *b"AFM2"
        } else {
            self.stream
        };
        let output = self
            .streams
            .entry((key, target_stream))
            .or_insert_with(|| Encoder::new(0, ModelFlags::default()));
        *values = Array::External(output.array(&leaf)?);
        Ok(())
    }
}
fn visit_core<'data, L: ModelLayout, V: ArrayVisitor<'data>>(
    core: &mut Core<'data, L>,
    visitor: &mut V,
) -> Result<(), V::Error> {
    macro_rules! visit { ($($field:ident),* $(,)?) => { $(for record in &mut core.$field { record.visit_arrays(visitor)?; })* }; }
    visit!(
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
    core.texture_flipbooks.visit_arrays(visitor)?;
    Ok(())
}
pub(crate) fn visit_extensions<'data, D: Dialect, V: ArrayVisitor<'data>>(
    model: &mut Model<'data, D>,
    visitor: &mut V,
) -> Result<(), V::Error> {
    for chunk in model.extensions_mut() {
        match chunk {
            Chunk::ParentTextureWeights(records) => {
                for record in records {
                    record.visit_arrays(visitor)?;
                }
            }
            Chunk::ParentEventTracks(records) => {
                for record in records {
                    record.visit_arrays(visitor)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

impl<'data, V: SupportsExternalSkins> From<Skin<'data, V>> for File<'data> {
    fn from(file: Skin<'data, V>) -> Self {
        Self::Skin(Box::new(DynamicSkin::from_typed(file)))
    }
}
impl<'data, V: ModelLayout> From<Skeleton<'data, V>> for File<'data> {
    fn from(file: Skeleton<'data, V>) -> Self {
        Self::Skeleton(DynamicSkeleton::from_typed(file))
    }
}
