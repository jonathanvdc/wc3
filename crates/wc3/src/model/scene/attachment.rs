//! Attachment records in `ATCH` chunks.
use super::node::{set_node_kind, validate_node_kind};
use crate::model::mdl::{MdlWriter, Parser, Span};
use crate::model::Encoder;
use crate::model::KnownChunk;
use crate::model::ModelVersion;
use crate::model::ValueError;
use crate::model::WriteError;
use crate::model::{mdl, mdx};
use std::io::Write as IoWrite;

use crate::model::{AttachmentVisibility, AttachmentsChunk, Cursor};

use std::borrow::Cow;

use crate::model::FixedText;
use crate::model::{AnimationTrack, Model, Node, ReadError};

const PATH_SIZE: usize = 256;

/// An attachment node with a model path and optional visibility track.
///
/// MDL reading reconstructs the attachment object-kind bit; writing requires
/// matching node bits and a zero reserved word.
#[derive(Clone, Debug, PartialEq)]
pub struct Attachment {
    node: Node,
    path: FixedText<PATH_SIZE>,
    reserved: u32,
    id: u32,
    visibility_track: Option<AnimationTrack<AttachmentVisibility>>,
}

impl Attachment {
    /// Creates an attachment from a node, model path, and attachment ID.
    pub fn new(node: Node, path: &str, id: u32) -> Result<Self, ValueError> {
        let mut attachment = Self {
            node,
            path: FixedText::default(),
            reserved: 0,
            id,
            visibility_track: None,
        };
        attachment.set_path(path)?;
        Ok(attachment)
    }

    /// Borrows the shared node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Borrows the shared node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Returns the model path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        self.path.text()
    }

    /// Sets the model path and clears the old path field.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        self.path.set_text(path)
    }

    /// Returns the attachment ID.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Sets the attachment ID.
    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    /// Borrows the optional visibility track.
    pub fn visibility_track(&self) -> Option<&AnimationTrack<AttachmentVisibility>> {
        self.visibility_track.as_ref()
    }

    /// Replaces the optional visibility track.
    pub fn set_visibility_track(&mut self, track: Option<&AnimationTrack<AttachmentVisibility>>) {
        self.visibility_track = track.cloned();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all attachments in `ATCH` chunks.
    pub fn attachments(&self) -> Vec<Attachment> {
        self.collect_chunk_records::<AttachmentsChunk>()
    }

    /// Replaces attachments in the first `ATCH` chunk.
    pub fn set_attachments(&mut self, attachments: &[Attachment]) {
        self.replace_chunk(AttachmentsChunk::new(attachments.to_vec()));
    }
}

impl mdx::Read for Attachment {
    fn read_mdx(source: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let mut cursor = source.slice_u32_sized()?;

        let mut probe = cursor;
        let node_size = probe.read::<u32>()? as usize;
        let node = Node::decode_mdx(cursor.read_exact(node_size)?)?;
        let path = cursor.read()?;
        let reserved = cursor.read()?;
        let id = cursor.read()?;
        let visibility_track = if cursor.remaining().is_empty() {
            None
        } else {
            let track = cursor.read::<AnimationTrack<AttachmentVisibility>>()?;

            Some(track)
        };
        cursor.finish()?;
        Ok(Self {
            node,
            path,
            reserved,
            id,
            visibility_track,
        })
    }
}

impl mdx::Write for Attachment {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        let marker = bytes.begin_sized();
        bytes.write(&self.node)?;
        bytes.write(&self.path)?;
        bytes.write(&self.reserved)?;
        bytes.write(&self.id)?;
        if let Some(track) = &self.visibility_track {
            bytes.write(track)?;
        }
        bytes.finish_sized(marker, AttachmentsChunk::TAG)?;
        Ok(())
    }
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(
    block = "Attachment",
    after_read = "finish_attachment",
    validate_write = "validate_attachment"
)]
struct AttachmentMdl {
    #[mdl(flatten)]
    node: Node,
    #[mdl(property = "Path", default)]
    path: FixedText<PATH_SIZE>,
    #[mdl(property = "AttachmentID", default)]
    id: u32,
    #[mdl(repeated = "Visibility", unique_by = "visibility_key")]
    visibility: Vec<AnimationTrack<AttachmentVisibility>>,
}
fn visibility_key(_: &AnimationTrack<AttachmentVisibility>) -> bool {
    true
}
fn finish_attachment(value: &mut AttachmentMdl, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x800);
    Ok(())
}
fn validate_attachment(value: &AttachmentMdl) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x800)
}
impl mdl::Read for Attachment {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let value = parser.read::<AttachmentMdl>()?;
        Ok(Self {
            node: value.node,
            path: value.path,
            reserved: 0,
            id: value.id,
            visibility_track: value.visibility.into_iter().next(),
        })
    }
}
impl mdl::Write for Attachment {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        if self.reserved != 0 {
            return Err(mdl::WriteError::Unsupported("attachment reserved word"));
        }
        writer.write(&AttachmentMdl {
            node: self.node.clone(),
            path: self.path,
            id: self.id,
            visibility: self.visibility_track.iter().cloned().collect(),
        })
    }
}
