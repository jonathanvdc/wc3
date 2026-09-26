//! Attachment records in `ATCH` chunks.
use crate::Encoder;
use crate::Tag;
use crate::ValueError;

use crate::{AttachmentsChunk, Cursor};
use crate::{Decodable, Encodable};
use std::borrow::Cow;

use crate::utils::field;
use crate::{AnimationTrack, Error, Model, Node};

const PATH_SIZE: usize = 256;

/// An attachment node with a model path and optional visibility track.
#[derive(Clone, Debug, PartialEq)]
pub struct Attachment {
    node: Node,
    path: [u8; PATH_SIZE],
    reserved: u32,
    id: u32,
    visibility_track: Option<AnimationTrack>,
}

impl Attachment {
    /// Creates an attachment from a node, model path, and attachment ID.
    pub fn new(node: Node, path: &str, id: u32) -> Result<Self, ValueError> {
        let mut attachment = Self {
            node,
            path: [0; PATH_SIZE],
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
        field::text(&self.path)
    }

    /// Sets the model path and clears the old path field.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        field::set_text(&mut self.path, path)
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
    pub fn visibility_track(&self) -> Option<&AnimationTrack> {
        self.visibility_track.as_ref()
    }

    /// Replaces the optional visibility track.
    pub fn set_visibility_track(
        &mut self,
        track: Option<&AnimationTrack>,
    ) -> Result<(), ValueError> {
        if let Some(track) = track {
            if track.tag != *b"KATV" {
                return Err(ValueError::InvalidTrackTag {
                    record: Attachment::TAG,
                    track: track.tag,
                });
            }
        }
        self.visibility_track = track.cloned();
        Ok(())
    }
}

impl Model {
    /// Decodes all attachments in `ATCH` chunks.
    pub fn attachments(&self) -> Vec<Attachment> {
        self.collect_chunk_records::<AttachmentsChunk>()
    }

    /// Replaces attachments in the first `ATCH` chunk.
    pub fn set_attachments(&mut self, attachments: &[Attachment]) {
        self.replace_chunk(AttachmentsChunk::new(attachments.to_vec()));
    }
}

impl Decodable for Attachment {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;

        let mut probe = cursor;
        let node_size = probe.read::<u32>()? as usize;
        let node = Node::decode(cursor.read_exact(node_size)?, 0)?;
        let path = cursor
            .read_exact(PATH_SIZE)?
            .try_into()
            .expect("fixed-width path");
        let reserved = cursor.read()?;
        let id = cursor.read()?;
        let visibility_track = if cursor.remaining().is_empty() {
            None
        } else {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if track.tag != *b"KATV" {
                return Err(Error::MalformedRecord {
                    tag: Self::TAG,
                    offset,
                });
            }

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

impl Encodable for Attachment {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        self.node.encode_to(bytes)?;
        bytes.write_bytes(&self.path);
        bytes.write(self.reserved);
        bytes.write(self.id);
        if let Some(track) = &self.visibility_track {
            if track.tag != *b"KATV" {
                return Err(Error::MalformedRecord {
                    tag: Attachment::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, Attachment::TAG)?;
        Ok(())
    }
}

impl Attachment {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"ATCH";
}
