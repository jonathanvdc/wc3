//! Shared node headers used by bones and helpers.
use crate::ModelVersion;
crate::animation::track_group! {
    pub enum NodeTrack {
        Translation: NodeTranslation,
        Rotation: NodeRotation,
        Scaling: NodeScaling,
    }
}

use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ValueError;

use crate::{BonesChunk, Cursor, HelpersChunk};
use crate::{Readable, Writable};
use std::borrow::Cow;

use crate::FixedText;
use crate::{DecodeError, Model};

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
#[derive(Clone, Debug, PartialEq, Readable, Writable)]
#[mdx(sized(tag = HelpersChunk::TAG))]
pub struct Node {
    name: FixedText<NAME_SIZE>,
    object_id: u32,
    parent_id: u32,
    raw_flags: u32,
    tracks: Vec<NodeTrack>,
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
    pub fn new(name: &str, object_id: u32) -> Result<Self, ValueError> {
        let mut node = Self {
            name: FixedText::default(),
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
        self.name.text()
    }

    /// Sets the node name and clears unused bytes.
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.name.set_text(name)
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
    pub fn tracks(&self) -> &[NodeTrack] {
        &self.tracks
    }
    /// Replaces transform tracks.
    pub fn set_tracks(&mut self, tracks: &[NodeTrack]) {
        self.tracks = tracks.to_vec();
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

impl<V: ModelVersion> Model<V> {
    /// Decodes every bone in `BONE` chunks.
    pub fn bones(&self) -> Vec<Bone> {
        self.collect_chunk_records::<BonesChunk>()
    }

    /// Replaces all bones in the first `BONE` chunk.
    pub fn set_bones(&mut self, bones: &[Bone]) {
        self.replace_chunk(BonesChunk::new(bones.to_vec()));
    }

    /// Decodes every helper node in `HELP` chunks.
    pub fn helpers(&self) -> Vec<Node> {
        self.collect_chunk_records::<HelpersChunk>()
    }

    /// Replaces all helpers in the first `HELP` chunk.
    pub fn set_helpers(&mut self, helpers: &[Node]) {
        self.replace_chunk(HelpersChunk::new(helpers.to_vec()));
    }
}

impl Readable for Bone {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let node = cursor.read()?;
        let geoset_id = cursor.read()?;
        let geoset_animation_id = cursor.read()?;
        Ok(Self {
            node,
            geoset_id,
            geoset_animation_id,
        })
    }
}

impl Writable for Bone {
    fn write_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        bytes.write(&self.node)?;
        bytes.write(&self.geoset_id)?;
        bytes.write(&self.geoset_animation_id)?;
        Ok(())
    }
}
