//! Shared node headers used by bones and helpers.
use crate::model::mdl::{MdlWriter, Parser, Span, TokenKind, WriteFields as _};
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
use bitfield::bitfield;
use std::io::Write as IoWrite;
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

use crate::model::FixedText;
use crate::model::{Model, ReadError};

const NAME_SIZE: usize = 80;

bitfield! {
    /// Node behavior and historical object-kind bits. Unrecognized bits survive conversion.
    ///
    /// Bits 15–20 depend on the containing emitter record. Prefer the emitter's
    /// `flags()` and `set_flags()` methods for those bits: for example, bit 17
    /// means Particle2 line emission but Popcorn unfogged rendering.
    /// Historical kind bits are preserved storage, not an authoritative record kind.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write)]
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
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = HelpersChunk::TAG))]
#[mdl(fields)]
pub struct Node {
    #[mdl(header)]
    pub name: FixedText<NAME_SIZE>,
    #[mdl(property = "ObjectId")]
    pub object_id: u32,
    #[mdl(
        property = "Parent",
        default = "no_reference",
        skip_if = "is_no_reference"
    )]
    pub parent_id: u32,
    #[mdl(
        flags(
            DontInheritTranslation = 1,
            DontInheritRotation = 2,
            DontInheritScaling = 4,
            Billboarded = 8,
            BillboardedLockX = 16,
            BillboardedLockY = 32,
            BillboardedLockZ = 64,
            CameraAnchored = 128
        ),
        allow_bits = 0x1fff00
    )]
    pub flags: NodeFlags,
    #[mdl(repeated(Translation, Rotation, Scaling), unique_by = "NodeTrack::tag")]
    pub tracks: Vec<NodeTrack>,
}

/// A bone with a decoded node and two geoset references.
///
/// Reading MDL reconstructs the node's bone bit. MDL writing requires that bit
/// (and no other object-kind bits), preserving the exact binary representation.
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(
    block = "Bone",
    after_read = "finish_bone",
    validate_write = "validate_bone"
)]
pub struct Bone {
    #[mdl(flatten)]
    pub node: Node,
    #[mdl(
        property = "GeosetId",
        default = "no_reference",
        read_with = "read_geoset",
        write_with = "write_geoset"
    )]
    pub geoset_id: u32,
    #[mdl(
        property = "GeosetAnimId",
        default = "no_reference",
        read_with = "read_geoset_animation",
        write_with = "write_geoset_animation"
    )]
    pub geoset_animation_id: u32,
}

impl Node {
    pub(crate) fn mdl_flags(&self) -> NodeFlags {
        self.flags
    }

    pub(crate) fn set_mdl_flags(&mut self, flags: NodeFlags) {
        self.flags = flags;
    }

    /// Creates a node without animation tracks.
    pub fn new(name: &str, object_id: u32) -> Result<Self, ValueError> {
        let mut node = Self {
            name: FixedText::default(),
            object_id,
            parent_id: u32::MAX,
            flags: NodeFlags::default(),
            tracks: Vec::new(),
        };
        node.name.set_text(name)?;
        Ok(node)
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
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
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
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        bytes.write(&self.node)?;
        bytes.write(&self.geoset_id)?;
        bytes.write(&self.geoset_animation_id)?;
        Ok(())
    }
}

fn no_reference() -> u32 {
    u32::MAX
}
fn is_no_reference(value: &u32) -> bool {
    *value == u32::MAX
}
fn finish_bone(value: &mut Bone, _: Span) -> Result<(), mdl::ReadError> {
    value.node.flags.0 |= 0x100;
    Ok(())
}
fn validate_bone(value: &Bone) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x100)
}
pub(crate) fn validate_node_kind(node: &Node, kind: u32) -> Result<(), mdl::WriteError> {
    if node.flags.bits() & !0xff != kind {
        return Err(mdl::WriteError::Unsupported("node object-kind bits"));
    }
    Ok(())
}
pub(crate) fn set_node_kind(node: &mut Node, kind: u32) {
    node.flags.0 |= kind;
}
fn read_reference(parser: &mut Parser<'_>, keyword: &str) -> Result<u32, mdl::ReadError> {
    if parser
        .peek()?
        .is_some_and(|token| token.kind == TokenKind::Ident(keyword))
    {
        parser.next_token()?;
        Ok(u32::MAX)
    } else {
        parser.read()
    }
}
fn read_geoset(parser: &mut Parser<'_>) -> Result<u32, mdl::ReadError> {
    read_reference(parser, "Multiple")
}
fn read_geoset_animation(parser: &mut Parser<'_>) -> Result<u32, mdl::ReadError> {
    read_reference(parser, "None")
}
fn write_reference<W: IoWrite>(
    value: &u32,
    writer: &mut MdlWriter<W>,
    keyword: &str,
) -> Result<(), mdl::WriteError> {
    if *value == u32::MAX {
        writer.identifier(keyword)
    } else {
        writer.write(value)
    }
}
fn write_geoset<W: IoWrite>(value: &u32, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
    write_reference(value, writer, "Multiple")
}
fn write_geoset_animation<W: IoWrite>(
    value: &u32,
    writer: &mut MdlWriter<W>,
) -> Result<(), mdl::WriteError> {
    write_reference(value, writer, "None")
}
impl mdl::Read for Node {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let start = parser.peek()?.map_or(0, |token| token.span.start);
        parser.expect_ident("Helper")?;
        mdl::read_mdl_body(parser, start)
    }
}
impl mdl::Write for Node {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        validate_node_kind(self, 0)?;
        let state = self.prepare_mdl_fields(writer.dialect())?;
        writer.indent()?;
        writer.identifier("Helper")?;
        self.write_mdl_headers(writer)?;
        writer.open_body()?;
        self.write_mdl_fields(state, writer)?;
        writer.end_block()
    }
}
