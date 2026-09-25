//! Shared node headers used by bones and helpers.

use crate::Record;
use crate::{BonesChunk, Cursor, HelpersChunk, ModelChunk};
use std::borrow::Cow;

use crate::utils::field;
use crate::{AnimationTrack, Error, Model};

pub(crate) const HEADER_SIZE: usize = 96;
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

    /// Returns the name up to the first NUL, replacing invalid UTF-8.
    pub fn name(&self) -> Cow<'_, str> {
        field::text(&self.name)
    }

    /// Sets the node name and clears unused bytes.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        field::set_text(&mut self.name, name)
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
                    tag: Node::TAG,
                    offset: size,
                });
            }
            let bytes = track.encode()?;
            size = size
                .checked_add(bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: Node::TAG,
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
        self.collect_chunk_records::<BonesChunk>(|chunk| match chunk {
            ModelChunk::Bones(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces all bones in the first `BONE` chunk.
    pub fn set_bones(&mut self, bones: &[Bone]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::Bones(BonesChunk::new(bones.to_vec())))
    }

    /// Decodes every helper node in `HELP` chunks.
    pub fn helpers(&self) -> Result<Vec<Node>, Error> {
        self.collect_chunk_records::<HelpersChunk>(|chunk| match chunk {
            ModelChunk::Helpers(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces all helpers in the first `HELP` chunk.
    pub fn set_helpers(&mut self, helpers: &[Node]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::Helpers(HelpersChunk::new(helpers.to_vec())))
    }
}

impl Record for Node {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let name = cursor
            .read_exact(NAME_SIZE)?
            .try_into()
            .expect("fixed-width node name");
        let object_id = cursor.read_u32()?;
        let parent_id = cursor.read_u32()?;
        let raw_flags = cursor.read_u32()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !matches!(&track.tag, b"KGTR" | b"KGRT" | b"KGSC") {
                return Err(Error::MalformedRecord {
                    tag: Self::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self {
            name,
            object_id,
            parent_id,
            raw_flags,
            tracks,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok({
            let mut bytes = vec![0; HEADER_SIZE];
            bytes[4..84].copy_from_slice(&self.name);
            bytes[84..88].copy_from_slice(&self.object_id.to_le_bytes());
            bytes[88..92].copy_from_slice(&self.parent_id.to_le_bytes());
            bytes[92..96].copy_from_slice(&self.raw_flags.to_le_bytes());
            for track in &self.tracks {
                bytes.extend_from_slice(&track.encode().expect("validated node track"));
            }
            let size = bytes.len() as u32;
            bytes[..4].copy_from_slice(&size.to_le_bytes());
            bytes
        })
    }
}

impl Record for Bone {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let node = Node::decode_one(cursor, 0)?;
        let geoset_id = cursor.read_u32()?;
        let geoset_animation_id = cursor.read_u32()?;
        Ok(Self {
            node,
            geoset_id,
            geoset_animation_id,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok({
            let mut bytes = self.node.encode()?;
            bytes.extend_from_slice(&self.geoset_id.to_le_bytes());
            bytes.extend_from_slice(&self.geoset_animation_id.to_le_bytes());
            bytes
        })
    }
}

impl Node {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"HELP";
}

impl Bone {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"BONE";
}
