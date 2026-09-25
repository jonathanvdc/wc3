//! Shared node headers used by bones and helpers.

use std::borrow::Cow;

use crate::{AnimationTrack, Error, Model};

const BONE_TAG: [u8; 4] = *b"BONE";
const HELP_TAG: [u8; 4] = *b"HELP";
const HEADER_SIZE: usize = 96;
const NAME_SIZE: usize = 80;

/// A generic node with its optional animation tracks retained as raw bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    bytes: Vec<u8>,
}

/// A bone node followed by geoset and geoset-animation references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bone {
    bytes: Vec<u8>,
}

fn read_size(data: &[u8], offset: usize, tag: [u8; 4]) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord { tag, offset })?;
    let size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    if size < HEADER_SIZE
        || offset
            .checked_add(size)
            .map_or(true, |end| end > data.len())
    {
        return Err(Error::MalformedRecord { tag, offset });
    }
    Ok(size)
}

impl Node {
    /// Creates a node without animation tracks.
    pub fn new(name: &str, object_id: u32) -> Result<Self, Error> {
        let mut bytes = vec![0; HEADER_SIZE];
        bytes[..4].copy_from_slice(&(HEADER_SIZE as u32).to_le_bytes());
        bytes[84..88].copy_from_slice(&object_id.to_le_bytes());
        bytes[88..92].copy_from_slice(&u32::MAX.to_le_bytes());
        let mut node = Self { bytes };
        node.set_name(name)?;
        Ok(node)
    }

    /// Wraps one inclusive-size node record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if read_size(bytes, 0, HELP_TAG)? != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: HELP_TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete node, including its size field and animation data.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the name up to the first NUL, replacing invalid UTF-8.
    pub fn name(&self) -> Cow<'_, str> {
        let field = &self.bytes[4..84];
        let end = field
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(NAME_SIZE);
        String::from_utf8_lossy(&field[..end])
    }

    /// Sets the node name, clearing unused bytes in the field.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        if name.len() >= NAME_SIZE || name.as_bytes().contains(&0) {
            return Err(Error::InvalidString {
                max_bytes: NAME_SIZE - 1,
            });
        }
        self.bytes[4..84].fill(0);
        self.bytes[4..4 + name.len()].copy_from_slice(name.as_bytes());
        Ok(())
    }

    /// Returns the object ID.
    pub fn object_id(&self) -> u32 {
        self.u32_at(84)
    }

    /// Sets the object ID.
    pub fn set_object_id(&mut self, id: u32) {
        self.set_u32_at(84, id);
    }

    /// Returns the parent ID; `u32::MAX` means no parent.
    pub fn parent_id(&self) -> u32 {
        self.u32_at(88)
    }

    /// Sets the parent ID; use `u32::MAX` for no parent.
    pub fn set_parent_id(&mut self, id: u32) {
        self.set_u32_at(88, id);
    }

    /// Returns raw node flags.
    pub fn flags(&self) -> u32 {
        self.u32_at(92)
    }

    /// Sets raw node flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.set_u32_at(92, flags);
    }

    /// Returns undecoded animation track bytes after the shared header.
    pub fn track_bytes(&self) -> &[u8] {
        &self.bytes[HEADER_SIZE..]
    }

    /// Decodes node translation, rotation, and scaling tracks.
    pub fn tracks(&self) -> Result<Vec<AnimationTrack>, Error> {
        let mut tracks = Vec::new();
        let mut offset = HEADER_SIZE;
        while offset < self.bytes.len() {
            let (track, consumed) = AnimationTrack::parse(&self.bytes, offset)?;
            if !matches!(&track.tag, b"KGTR" | b"KGRT" | b"KGSC") {
                return Err(Error::MalformedRecord {
                    tag: *b"HELP",
                    offset,
                });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(tracks)
    }

    /// Replaces the node's transform tracks and updates its inclusive size.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut bytes = self.bytes[..HEADER_SIZE].to_vec();
        for track in tracks {
            if !matches!(&track.tag, b"KGTR" | b"KGRT" | b"KGSC") {
                return Err(Error::MalformedRecord {
                    tag: *b"HELP",
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: *b"HELP",
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }

    fn u32_at(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("four-byte field"),
        )
    }

    fn set_u32_at(&mut self, offset: usize, value: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

impl Bone {
    /// Creates a bone from a node and two references.
    pub fn new(node: Node, geoset_id: u32, geoset_animation_id: u32) -> Self {
        let mut bytes = node.bytes;
        bytes.extend_from_slice(&geoset_id.to_le_bytes());
        bytes.extend_from_slice(&geoset_animation_id.to_le_bytes());
        Self { bytes }
    }

    /// Wraps one bone record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let node_size = read_size(bytes, 0, BONE_TAG)?;
        if node_size.checked_add(8) != Some(bytes.len()) {
            return Err(Error::MalformedRecord {
                tag: BONE_TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete bone record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the shared node record.
    pub fn node(&self) -> Node {
        let size = self.bytes.len() - 8;
        Node {
            bytes: self.bytes[..size].to_vec(),
        }
    }

    /// Returns the geoset ID, or `u32::MAX` if unbound.
    pub fn geoset_id(&self) -> u32 {
        let offset = self.bytes.len() - 8;
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("four-byte field"),
        )
    }

    /// Returns the geoset animation ID, or `u32::MAX` if unbound.
    pub fn geoset_animation_id(&self) -> u32 {
        let offset = self.bytes.len() - 4;
        u32::from_le_bytes(self.bytes[offset..].try_into().expect("four-byte field"))
    }
}

impl Model {
    /// Decodes every bone in `BONE` chunks.
    pub fn bones(&self) -> Result<Vec<Bone>, Error> {
        let mut bones = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == BONE_TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let size = read_size(&chunk.data, offset, BONE_TAG)?;
                let end = offset
                    .checked_add(size)
                    .and_then(|end| end.checked_add(8))
                    .filter(|&end| end <= chunk.data.len())
                    .ok_or(Error::MalformedRecord {
                        tag: BONE_TAG,
                        offset,
                    })?;
                bones.push(Bone::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(bones)
    }

    /// Replaces all bones in the first `BONE` chunk.
    pub fn set_bones(&mut self, bones: &[Bone]) -> Result<(), Error> {
        let size = bones.iter().try_fold(0usize, |sum, bone| {
            sum.checked_add(bone.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: BONE_TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for bone in bones {
            data.extend_from_slice(bone.as_bytes());
        }
        self.replace_chunks(BONE_TAG, data);
        Ok(())
    }

    /// Decodes every helper node in `HELP` chunks.
    pub fn helpers(&self) -> Result<Vec<Node>, Error> {
        let mut helpers = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == HELP_TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let size = read_size(&chunk.data, offset, HELP_TAG)?;
                let end = offset + size;
                helpers.push(Node::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(helpers)
    }

    /// Replaces all helpers in the first `HELP` chunk.
    pub fn set_helpers(&mut self, helpers: &[Node]) -> Result<(), Error> {
        let size = helpers.iter().try_fold(0usize, |sum, helper| {
            sum.checked_add(helper.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: HELP_TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for helper in helpers {
            data.extend_from_slice(helper.as_bytes());
        }
        self.replace_chunks(HELP_TAG, data);
        Ok(())
    }
}
