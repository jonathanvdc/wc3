//! Named attachment points and optional attached models.
use super::node::{set_node_kind, validate_node_kind};
use crate::model::common::Ownership;
use crate::model::mdlx::mdl::Span;
use crate::model::mdlx::AttachmentsChunk;
use crate::model::mdlx::FixedText;
use crate::model::mdlx::KnownChunk;
use crate::model::mdlx::ModelDialect;
use crate::model::mdlx::ValueError;
use crate::model::mdlx::{mdl, mdx};
use crate::model::mdlx::{Model, Node, Track};

const PATH_SIZE: usize = 260;

/// An attachment node with a model path and optional visibility track.
#[derive(
    Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write, Ownership, mdx::ReadOwned,
)]
#[mdx(sized(tag = AttachmentsChunk::TAG))]
#[mdl(
    block = "Attachment",
    after_read = "finish_attachment",
    validate_write = "validate_attachment"
)]
pub struct Attachment<'data> {
    /// Shared node.
    #[mdl(flatten)]
    pub node: Node<'data>,
    /// Fixed-width path preserving every stored byte.
    #[mdl(property = "Path", default)]
    pub path: FixedText<PATH_SIZE>,
    /// Attachment ID.
    #[mdl(property = "AttachmentID", default)]
    pub id: u32,
    /// Optional visibility track.
    #[mdx(tag = *b"KATV")]
    #[mdl(property = "Visibility")]
    pub visibility: Option<Track<'data, f32>>,
}

impl<'data> Attachment<'data> {
    /// Creates an attachment from a node, model path, and attachment ID.
    pub fn new(node: Node<'data>, path: &str, id: u32) -> Result<Self, ValueError> {
        let mut attachment = Self {
            node,
            path: FixedText::default(),
            id,
            visibility: None,
        };
        attachment.path.set_text(path)?;
        Ok(attachment)
    }
}

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn attachments<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s Attachment<'data>> + use<'data, 's, D> {
        self.records::<AttachmentsChunk<'_>>()
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn attachments_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut Attachment<'data>> + use<'data, 's, D> {
        self.records_mut::<AttachmentsChunk<'_>>()
    }

    /// Replaces attachments with one `ATCH` chunk, removing any duplicate chunks.
    pub fn set_attachments(&mut self, attachments: impl IntoIterator<Item = Attachment<'data>>) {
        self.replace_chunk(AttachmentsChunk::new(attachments.into_iter().collect()));
    }
}

fn finish_attachment<'data>(value: &mut Attachment<'data>, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x800);
    Ok(())
}
fn validate_attachment<'data>(value: &Attachment<'data>) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x800)
}
