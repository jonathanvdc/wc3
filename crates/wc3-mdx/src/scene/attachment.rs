//! Attachment records in `ATCH` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ModelVersion;
use crate::ValueError;

use crate::{AttachmentVisibility, AttachmentsChunk, Cursor};
use crate::{Readable, Writable};
use std::borrow::Cow;

use crate::FixedText;
use crate::{AnimationTrack, DecodeError, Model, Node};

const PATH_SIZE: usize = 256;

/// An attachment node with a model path and optional visibility track.
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

impl Readable for Attachment {
    fn read_from(source: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;

        let mut probe = cursor;
        let node_size = probe.read::<u32>()? as usize;
        let node = Node::decode(cursor.read_exact(node_size)?)?;
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

impl Writable for &Attachment {
    fn write_to(self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let marker = bytes.begin_sized();
        self.node.write_to(bytes)?;
        bytes.write(&self.path)?;
        bytes.write(self.reserved)?;
        bytes.write(self.id)?;
        if let Some(track) = &self.visibility_track {
            bytes.write(track)?;
        }
        bytes.finish_sized(marker, AttachmentsChunk::TAG)?;
        Ok(())
    }
}
