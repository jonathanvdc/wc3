//! Transform hierarchy shared by helpers, bones, and scene objects.
use crate::model::mdl::{Parser, Span, TokenKind, WriteFields as _, Writer};
use crate::model::Encoder;
use crate::model::FixedText;
use crate::model::IoError;
use crate::model::KnownChunk;
use crate::model::Model;
use crate::model::ModelVersion;
use crate::model::ValueError;
use crate::model::{mdl, mdx};
use crate::model::{BonesChunk, Cursor, HelpersChunk};
use crate::model::{Quaternion, Track, Vec3};
use bitfield::bitfield;
use std::io::Write as IoWrite;

const NAME_SIZE: usize = 80;

bitfield! {
    /// Transform inheritance, billboarding, and object flags.
    ///
    /// For emitters, use the emitter's typed flags: the same bit can mean
    /// different behavior for Particle2 and Popcorn effects. Unknown bits are
    /// retained in MDX but cannot be exported to MDL.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
    #[mdl(
        flags(
            DontInheritTranslation = 1,
            DontInheritRotation = 2,
            DontInheritScaling = 4,
            Billboarded = 8,
            BillboardedLockX = 16,
            BillboardedLockY = 32,
            BillboardedLockZ = 64,
            CameraAnchored = 128,
        ),
        allow_bits = 0x7f00
    )]
    pub struct NodeFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
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

/// A named object in the model transform hierarchy.
///
/// `F` stores the complete flag word and selects its interpretation. Emitter
/// records use their own flag types; ordinary nodes default to `NodeFlags`.
/// MDL delegates the complete flag schema to `F`, including emitter flags.
/// Use [`Node::cast_flags`] to change interpretations without changing the bits.
/// Standalone MDL `Helper` blocks use the default interpretation.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = HelpersChunk::TAG))]
#[mdl(fields)]
pub struct Node<F = NodeFlags> {
    #[mdl(header)]
    /// Object name; event and attachment naming conventions may give it game-specific meaning.
    pub name: FixedText<NAME_SIZE>,
    #[mdl(property = "ObjectId")]
    /// Identifier used by parent references and as an index into model pivot points.
    pub object_id: u32,
    #[mdl(
        property = "Parent",
        default = "no_reference",
        skip_if = "is_no_reference"
    )]
    /// Parent ID, or `u32::MAX` for no parent.
    pub parent_id: u32,
    #[mdl(flatten)]
    /// Node flags.
    pub flags: F,
    #[mdx(tag = *b"KGTR")]
    #[mdl(property = "Translation")]
    /// Optional local translation animation.
    pub translation: Option<Track<Vec3>>,
    #[mdx(tag = *b"KGRT")]
    #[mdl(property = "Rotation")]
    /// Optional local quaternion rotation animation.
    pub rotation: Option<Track<Quaternion>>,
    #[mdx(tag = *b"KGSC")]
    #[mdl(property = "Scaling")]
    /// Optional local scale animation.
    pub scaling: Option<Track<Vec3>>,
}

/// A skeletal node that influences mesh geometry.
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(
    block = "Bone",
    after_read = "finish_bone",
    validate_write = "validate_bone"
)]
pub struct Bone {
    #[mdl(flatten)]
    /// Shared node.
    pub node: Node,
    #[mdl(
        property = "GeosetId",
        default = "no_reference",
        read_with = "read_geoset",
        write_with = "write_geoset"
    )]
    /// Geoset index, or `u32::MAX` for a bone that spans multiple geosets.
    pub geoset_id: u32,
    #[mdl(
        property = "GeosetAnimId",
        default = "no_reference",
        read_with = "read_geoset_animation",
        write_with = "write_geoset_animation"
    )]
    /// Geoset-animation index, or `u32::MAX` when none is assigned.
    pub geoset_animation_id: u32,
}

/// A lossless interpretation of the complete 32-bit node flag word.
pub trait NodeFlagInterpretation: Copy + Default {
    /// Returns every stored bit, including unknown and historical kind bits.
    fn bits(self) -> u32;
    /// Interprets a word without discarding any bits.
    fn from_bits(bits: u32) -> Self;
}

impl<F: NodeFlagInterpretation> Node<F> {
    /// Changes the flag interpretation while preserving every field and stored bit.
    pub fn cast_flags<G: NodeFlagInterpretation>(self) -> Node<G> {
        Node {
            name: self.name,
            object_id: self.object_id,
            parent_id: self.parent_id,
            flags: G::from_bits(self.flags.bits()),
            translation: self.translation,
            rotation: self.rotation,
            scaling: self.scaling,
        }
    }
}

impl Node {
    /// Creates a node without animation tracks.
    pub fn new(name: &str, object_id: u32) -> Result<Self, ValueError> {
        let mut node = Self {
            name: FixedText::default(),
            object_id,
            parent_id: u32::MAX,
            flags: NodeFlags::default(),
            translation: None,
            rotation: None,
            scaling: None,
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
    /// Returns owned copies of all bones in model order.
    pub fn bones(&self) -> Vec<Bone> {
        self.collect_chunk_records::<BonesChunk>()
    }

    /// Replaces all bones with one `BONE` chunk, removing any duplicate chunks.
    pub fn set_bones(&mut self, bones: &[Bone]) {
        self.replace_chunk(BonesChunk::new(bones.to_vec()));
    }

    /// Returns owned copies of all helper nodes in model order.
    pub fn helpers(&self) -> Vec<Node> {
        self.collect_chunk_records::<HelpersChunk>()
    }

    /// Replaces all helpers with one `HELP` chunk, removing any duplicate chunks.
    pub fn set_helpers(&mut self, helpers: &[Node]) {
        self.replace_chunk(HelpersChunk::new(helpers.to_vec()));
    }
}

impl mdx::Read for Bone {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
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
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
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
pub(crate) fn validate_node_kind<F: NodeFlagInterpretation>(
    node: &Node<F>,
    kind: u32,
) -> Result<(), mdl::WriteError> {
    if node.flags.bits() & !0xff != kind {
        return Err(mdl::WriteError::Unrepresentable {
            field: "node object-kind bits",
        });
    }
    Ok(())
}
pub(crate) fn set_node_kind<F: NodeFlagInterpretation>(node: &mut Node<F>, kind: u32) {
    node.flags = F::from_bits(node.flags.bits() | kind);
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
    writer: &mut Writer<W>,
    keyword: &str,
) -> Result<(), IoError<mdl::WriteError>> {
    if *value == u32::MAX {
        writer.identifier(keyword)
    } else {
        writer.write(value)
    }
}
fn write_geoset<W: IoWrite>(
    value: &u32,
    writer: &mut Writer<W>,
) -> Result<(), IoError<mdl::WriteError>> {
    write_reference(value, writer, "Multiple")
}
fn write_geoset_animation<W: IoWrite>(
    value: &u32,
    writer: &mut Writer<W>,
) -> Result<(), IoError<mdl::WriteError>> {
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
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
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

macro_rules! impl_node_flags {
    ($flags:ident) => {
        impl_node_flags!(@fields $flags,
            (dont_inherit_translation, set_dont_inherit_translation, 0),
            (dont_inherit_rotation, set_dont_inherit_rotation, 1),
            (dont_inherit_scaling, set_dont_inherit_scaling, 2),
            (billboarded, set_billboarded, 3),
            (billboard_lock_x, set_billboard_lock_x, 4),
            (billboard_lock_y, set_billboard_lock_y, 5),
            (billboard_lock_z, set_billboard_lock_z, 6),
            (camera_anchored, set_camera_anchored, 7)
        );
        impl NodeFlagInterpretation for $flags {
            fn bits(self) -> u32 { self.0 }
            fn from_bits(bits: u32) -> Self { Self(bits) }
        }
    };
    (@fields $flags:ident, $(($get:ident, $set:ident, $bit:literal)),+) => {
        impl $flags {
            $(
                #[doc = concat!("Returns the common `", stringify!($get), "` bit.")]
                pub fn $get(&self) -> bool { self.0 & (1 << $bit) != 0 }
                #[doc = concat!("Changes the common `", stringify!($get), "` bit, preserving all other bits.")]
                pub fn $set(&mut self, value: bool) {
                    self.0 = (self.0 & !(1 << $bit)) | (u32::from(value) << $bit);
                }
            )+
        }
    };
}
pub(crate) use impl_node_flags;

impl_node_flags!(NodeFlags);
