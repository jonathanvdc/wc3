//! Shared node headers used by bones and helpers.

use std::borrow::Cow;

use crate::{AnimationTrack, Error, Model};

const BONE_TAG: [u8; 4] = *b"BONE";
const HELP_TAG: [u8; 4] = *b"HELP";
const HEADER_SIZE: usize = 96;
const NAME_SIZE: usize = 80;

/// Node behavior and object-kind bits. Unrecognized bits survive conversion.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NodeFlags(u32);

impl NodeFlags {
    pub const DONT_INHERIT_TRANSLATION: Self = Self(0x1);
    pub const DONT_INHERIT_ROTATION: Self = Self(0x2);
    pub const DONT_INHERIT_SCALING: Self = Self(0x4);
    pub const BILLBOARDED: Self = Self(0x8);
    pub const BILLBOARD_LOCK_X: Self = Self(0x10);
    pub const BILLBOARD_LOCK_Y: Self = Self(0x20);
    pub const BILLBOARD_LOCK_Z: Self = Self(0x40);
    pub const CAMERA_ANCHORED: Self = Self(0x80);
    pub const BONE: Self = Self(0x100);
    pub const LIGHT: Self = Self(0x200);
    pub const EVENT_OBJECT: Self = Self(0x400);
    pub const ATTACHMENT: Self = Self(0x800);
    pub const PARTICLE_EMITTER: Self = Self(0x1000);
    pub const COLLISION_SHAPE: Self = Self(0x2000);
    pub const RIBBON_EMITTER: Self = Self(0x4000);
    pub const EMITTER_USES_MDL_OR_UNSHADED: Self = Self(0x8000);
    pub const EMITTER_USES_TGA_OR_SORT_FAR_Z: Self = Self(0x10000);
    pub const LINE_EMITTER: Self = Self(0x20000);
    pub const UNFOGGED: Self = Self(0x40000);
    pub const MODEL_SPACE: Self = Self(0x80000);
    pub const XY_QUAD: Self = Self(0x100000);

    /// Wraps all bits, including values not yet assigned a name.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    /// Returns the exact stored bits.
    pub const fn bits(self) -> u32 {
        self.0
    }
    /// Reports whether every bit in `other` is set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    /// Changes only the requested bits.
    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.0 |= other.0;
        } else {
            self.0 &= !other.0;
        }
    }
}

/// A shared node header with decoded transform tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    name: [u8; NAME_SIZE],
    object_id: u32,
    parent_id: u32,
    raw_flags: u32,
    tracks: Vec<AnimationTrack>,
}

/// A bone with a decoded node and two geoset references.
#[derive(Clone, Debug, PartialEq)]
pub struct Bone {
    node: Node,
    geoset_id: u32,
    geoset_animation_id: u32,
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
        let mut node = Self {
            name: [0; NAME_SIZE],
            object_id,
            parent_id: u32::MAX,
            raw_flags: 0,
            tracks: Vec::new(),
        };
        node.set_name(name)?;
        Ok(node)
    }

    /// Parses one inclusive-size node record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if read_size(bytes, 0, HELP_TAG)? != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: HELP_TAG,
                offset: 0,
            });
        }
        let name = bytes[4..84].try_into().expect("fixed-width node name");
        let word = |offset: usize| {
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("node field"))
        };
        let mut tracks = Vec::new();
        let mut offset = HEADER_SIZE;
        while offset < bytes.len() {
            let (track, size) = AnimationTrack::parse(bytes, offset)?;
            if !matches!(&track.tag, b"KGTR" | b"KGRT" | b"KGSC") {
                return Err(Error::MalformedRecord {
                    tag: HELP_TAG,
                    offset,
                });
            }
            tracks.push(track);
            offset += size;
        }
        Ok(Self {
            name,
            object_id: word(84),
            parent_id: word(88),
            raw_flags: word(92),
            tracks,
        })
    }

    /// Serializes the node and its validated transform tracks.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; HEADER_SIZE];
        bytes[4..84].copy_from_slice(&self.name);
        bytes[84..88].copy_from_slice(&self.object_id.to_le_bytes());
        bytes[88..92].copy_from_slice(&self.parent_id.to_le_bytes());
        bytes[92..96].copy_from_slice(&self.raw_flags.to_le_bytes());
        for track in &self.tracks {
            bytes.extend_from_slice(&track.to_bytes().expect("validated node track"));
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        bytes
    }

    /// Returns the name up to the first NUL, replacing invalid UTF-8.
    pub fn name(&self) -> Cow<'_, str> {
        let end = self
            .name
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(NAME_SIZE);
        String::from_utf8_lossy(&self.name[..end])
    }

    /// Sets the node name and clears unused bytes.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        if name.len() >= NAME_SIZE || name.as_bytes().contains(&0) {
            return Err(Error::InvalidString {
                max_bytes: NAME_SIZE - 1,
            });
        }
        self.name.fill(0);
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        Ok(())
    }

    /// Returns the object ID.
    pub fn object_id(&self) -> u32 {
        self.object_id
    }
    /// Changes the object ID.
    pub fn set_object_id(&mut self, id: u32) {
        self.object_id = id;
    }
    /// Returns the parent ID, or `u32::MAX` for no parent.
    pub fn parent_id(&self) -> u32 {
        self.parent_id
    }
    /// Changes the parent ID.
    pub fn set_parent_id(&mut self, id: u32) {
        self.parent_id = id;
    }
    /// Returns decoded node flags.
    pub fn flags(&self) -> NodeFlags {
        NodeFlags::from_bits(self.raw_flags)
    }
    /// Returns the exact flag bits.
    pub fn raw_flags(&self) -> u32 {
        self.raw_flags
    }
    /// Changes decoded node flags.
    pub fn set_flags(&mut self, flags: NodeFlags) {
        self.raw_flags = flags.bits();
    }
    /// Changes exact flag bits.
    pub fn set_raw_flags(&mut self, flags: u32) {
        self.raw_flags = flags;
    }
    /// Borrows decoded transform tracks without reparsing.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
    /// Replaces transform tracks after validating their tags and size.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut size = HEADER_SIZE;
        for track in tracks {
            if !matches!(&track.tag, b"KGTR" | b"KGRT" | b"KGSC") {
                return Err(Error::MalformedRecord {
                    tag: HELP_TAG,
                    offset: size,
                });
            }
            let bytes = track.to_bytes()?;
            size = size
                .checked_add(bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: HELP_TAG,
                    size: usize::MAX,
                })?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

impl Bone {
    /// Creates a bone from a node and two references.
    pub fn new(node: Node, geoset_id: u32, geoset_animation_id: u32) -> Self {
        Self {
            node,
            geoset_id,
            geoset_animation_id,
        }
    }

    /// Parses one node followed by its bone references.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let node_size = read_size(bytes, 0, BONE_TAG)?;
        if node_size.checked_add(8) != Some(bytes.len()) {
            return Err(Error::MalformedRecord {
                tag: BONE_TAG,
                offset: 0,
            });
        }
        let node = Node::from_bytes(&bytes[..node_size])?;
        let geoset_id =
            u32::from_le_bytes(bytes[node_size..node_size + 4].try_into().expect("bone ID"));
        let geoset_animation_id =
            u32::from_le_bytes(bytes[node_size + 4..].try_into().expect("animation ID"));
        Ok(Self {
            node,
            geoset_id,
            geoset_animation_id,
        })
    }

    /// Serializes the node and its bone references.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.node.to_bytes();
        bytes.extend_from_slice(&self.geoset_id.to_le_bytes());
        bytes.extend_from_slice(&self.geoset_animation_id.to_le_bytes());
        bytes
    }
    /// Borrows the shared node.
    pub fn node(&self) -> &Node {
        &self.node
    }
    /// Mutably borrows the shared node.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }
    /// Returns the geoset reference.
    pub fn geoset_id(&self) -> u32 {
        self.geoset_id
    }
    /// Returns the geoset animation reference.
    pub fn geoset_animation_id(&self) -> u32 {
        self.geoset_animation_id
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
        let mut data = Vec::new();
        for bone in bones {
            data.extend_from_slice(&bone.to_bytes());
            if data.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: BONE_TAG,
                    size: data.len(),
                });
            }
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
        let mut data = Vec::new();
        for helper in helpers {
            data.extend_from_slice(&helper.to_bytes());
            if data.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: HELP_TAG,
                    size: data.len(),
                });
            }
        }
        self.replace_chunks(HELP_TAG, data);
        Ok(())
    }
}
