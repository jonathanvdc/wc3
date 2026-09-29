//! Named attachment points and optional attached models.
use super::node::{set_node_kind, validate_node_kind};
use crate::model::mdl::Span;
use crate::model::AttachmentsChunk;
use crate::model::FixedText;
use crate::model::KnownChunk;
use crate::model::ModelVersion;
use crate::model::ValueError;
use crate::model::{mdl, mdx};
use crate::model::{Model, Node, Track};

const PATH_SIZE: usize = 260;

/// An attachment node with a model path and optional visibility track.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = AttachmentsChunk::TAG))]
#[mdl(
    block = "Attachment",
    after_read = "finish_attachment",
    validate_write = "validate_attachment"
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
    #[mdx(tag = *b"KATV")]
    #[mdl(property = "Visibility")]
    pub visibility: Option<Track<f32>>,
}

impl Attachment {
    /// Creates an attachment from a node, model path, and attachment ID.
    pub fn new(node: Node, path: &str, id: u32) -> Result<Self, ValueError> {
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

fn finish_attachment(value: &mut Attachment, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x800);
    Ok(())
}
fn validate_attachment(value: &Attachment) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x800)
}
