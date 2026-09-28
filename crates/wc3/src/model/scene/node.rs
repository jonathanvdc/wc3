//! Shared node headers used by bones and helpers.
use crate::model::mdx;
use crate::model::ModelVersion;
use bitfield::bitfield;
crate::model::animation::track_group! {
    pub enum NodeTrack {
        Translation: NodeTranslation,
        Rotation: NodeRotation,
        Scaling: NodeScaling,
    }
}

use crate::model::Encoder;
use crate::model::KnownChunk;
use crate::model::ValueError;
use crate::model::WriteError;

use crate::model::{BonesChunk, Cursor, HelpersChunk};

use std::borrow::Cow;

use crate::model::FixedText;
use crate::model::{Model, ReadError};

const NAME_SIZE: usize = 80;

bitfield! {
    /// Node behavior and object-kind bits. Unrecognized bits survive conversion.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct NodeFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `DONT_INHERIT_TRANSLATION` bit.
    pub dont_inherit_translation, set_dont_inherit_translation: 0;
    /// Returns or changes the `DONT_INHERIT_ROTATION` bit.
    pub dont_inherit_rotation, set_dont_inherit_rotation: 1;
    /// Returns or changes the `DONT_INHERIT_SCALING` bit.
    pub dont_inherit_scaling, set_dont_inherit_scaling: 2;
    /// Returns or changes the `BILLBOARDED` bit.
    pub billboarded, set_billboarded: 3;
    /// Returns or changes the `BILLBOARD_LOCK_X` bit.
    pub billboard_lock_x, set_billboard_lock_x: 4;
    /// Returns or changes the `BILLBOARD_LOCK_Y` bit.
    pub billboard_lock_y, set_billboard_lock_y: 5;
    /// Returns or changes the `BILLBOARD_LOCK_Z` bit.
    pub billboard_lock_z, set_billboard_lock_z: 6;
    /// Returns or changes the `CAMERA_ANCHORED` bit.
    pub camera_anchored, set_camera_anchored: 7;
    /// Returns or changes the `BONE` bit.
    pub bone, set_bone: 8;
    /// Returns or changes the `LIGHT` bit.
    pub light, set_light: 9;
    /// Returns or changes the `EVENT_OBJECT` bit.
    pub event_object, set_event_object: 10;
    /// Returns or changes the `ATTACHMENT` bit.
    pub attachment, set_attachment: 11;
    /// Returns or changes the `PARTICLE_EMITTER` bit.
    pub particle_emitter, set_particle_emitter: 12;
    /// Returns or changes the `COLLISION_SHAPE` bit.
    pub collision_shape, set_collision_shape: 13;
    /// Returns or changes the `RIBBON_EMITTER` bit.
    pub ribbon_emitter, set_ribbon_emitter: 14;
    /// Returns or changes the `EMITTER_USES_MDL_OR_UNSHADED` bit.
    pub emitter_uses_mdl_or_unshaded, set_emitter_uses_mdl_or_unshaded: 15;
    /// Returns or changes the `EMITTER_USES_TGA_OR_SORT_FAR_Z` bit.
    pub emitter_uses_tga_or_sort_far_z, set_emitter_uses_tga_or_sort_far_z: 16;
    /// Returns or changes the `LINE_EMITTER` bit.
    pub line_emitter, set_line_emitter: 17;
    /// Returns or changes the `UNFOGGED` bit.
    pub unfogged, set_unfogged: 18;
    /// Returns or changes the `MODEL_SPACE` bit.
    pub model_space, set_model_space: 19;
    /// Returns or changes the `XY_QUAD` bit.
    pub xy_quad, set_xy_quad: 20;
}

/// A shared node header with decoded transform tracks.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
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
        NodeFlags(self.raw_flags)
    }
    /// Changes decoded node flags.
    pub fn set_flags(&mut self, flags: NodeFlags) {
        self.raw_flags = flags.bits();
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

impl mdx::Read for Bone {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
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

impl mdx::Write for Bone {
    fn write_to(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        bytes.write(&self.node)?;
        bytes.write(&self.geoset_id)?;
        bytes.write(&self.geoset_animation_id)?;
        Ok(())
    }
}
