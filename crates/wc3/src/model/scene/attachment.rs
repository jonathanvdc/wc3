//! Named attachment points and optional attached models.
use super::node::{set_node_kind, validate_node_kind};
use crate::model::mdl::Span;
use crate::model::Encoder;
use crate::model::KnownChunk;
use crate::model::ModelVersion;
use crate::model::ValueError;
use crate::model::WriteError;
use crate::model::{mdl, mdx};

use crate::model::{AttachmentVisibility, AttachmentsChunk, Cursor};

use crate::model::FixedText;
use crate::model::{AnimationTrack, Model, Node, ReadError};

const PATH_SIZE: usize = 260;

/// An attachment node with a model path and optional visibility track.
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(
    block = "Attachment",
    after_read = "finish_attachment",
    validate_write = "validate_attachment",
    virtual_fields(
        #[mdl(repeated = "Visibility", unique_by = "visibility_key", get = "Self::mdl_visibility", set = "Self::set_mdl_visibility")]
        visibility: Vec<AnimationTrack<AttachmentVisibility>>,
    )
)]
pub struct Attachment {
    /// Shared node.
    #[mdl(flatten)]
    pub node: Node,
    /// Fixed-width path preserving every stored byte.
    #[mdl(property = "Path", default)]
    pub path: FixedText<PATH_SIZE>,
    /// Attachment ID.
    #[mdl(property = "AttachmentID", default)]
    pub id: u32,
    /// Optional visibility track.
    #[mdl(skip, default)]
    pub visibility_track: Option<AnimationTrack<AttachmentVisibility>>,
}

impl Attachment {
    fn mdl_visibility(&self) -> &[AnimationTrack<AttachmentVisibility>] {
        self.visibility_track.as_slice()
    }

    fn set_mdl_visibility(
        &mut self,
        visibility: Vec<AnimationTrack<AttachmentVisibility>>,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.visibility_track = visibility.into_iter().next();
        Ok(())
    }

    /// Creates an attachment from a node, model path, and attachment ID.
    pub fn new(node: Node, path: &str, id: u32) -> Result<Self, ValueError> {
        let mut attachment = Self {
            node,
            path: FixedText::default(),
            id,
            visibility_track: None,
        };
        attachment.path.set_text(path)?;
        Ok(attachment)
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of attachments in `ATCH` chunks.
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
        bytes.write(&self.id)?;
        if let Some(track) = &self.visibility_track {
            bytes.write(track)?;
        }
        bytes.finish_sized(marker, AttachmentsChunk::TAG)?;
        Ok(())
    }
}

fn visibility_key(_: &AnimationTrack<AttachmentVisibility>) -> bool {
    true
}
fn finish_attachment(value: &mut Attachment, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x800);
    Ok(())
}
fn validate_attachment(value: &Attachment) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x800)
}
