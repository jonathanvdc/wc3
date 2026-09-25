//! Attachment records in `ATCH` chunks.

use crate::Record;
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
    pub fn new(node: Node, path: &str, id: u32) -> Result<Self, Error> {
        let mut attachment = Self {
            node,
            path: [0; PATH_SIZE],
            reserved: 0,
            id,
            visibility_track: None,
        };
        attachment.set_path(path)?;
        attachment.encode()?;
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
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
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
    pub fn set_visibility_track(&mut self, track: Option<&AnimationTrack>) -> Result<(), Error> {
        if let Some(track) = track {
            if track.tag != *b"KATV" {
                return Err(Error::MalformedRecord {
                    tag: Attachment::TAG,
                    offset: 0,
                });
            }
            track.encode()?;
        }
        self.visibility_track = track.cloned();
        Ok(())
    }
}

impl Model {
    /// Decodes all attachments in `ATCH` chunks.
    pub fn attachments(&self) -> Result<Vec<Attachment>, Error> {
        self.collect_chunk_records::<crate::AttachmentsChunk>(|chunk| match chunk {
            crate::ModelChunk::Attachments(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces attachments in the first `ATCH` chunk.
    pub fn set_attachments(&mut self, attachments: &[Attachment]) -> Result<(), Error> {
        let size = attachments.iter().try_fold(0usize, |sum, attachment| {
            sum.checked_add(attachment.encode()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: Attachment::TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for attachment in attachments {
            data.extend_from_slice(&attachment.encode()?);
        }
        self.replace_chunks(Attachment::TAG, data)?;
        Ok(())
    }
}

impl Record for Attachment {
    fn decode_one(source: &mut crate::Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;

        let mut probe = cursor;
        let node_size = probe.read_u32()? as usize;
        let node = Node::decode(cursor.read_exact(node_size)?, 0)?;
        let path = cursor
            .read_exact(PATH_SIZE)?
            .try_into()
            .expect("fixed-width path");
        let reserved = cursor.read_u32()?;
        let id = cursor.read_u32()?;
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

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&self.node.encode()?);
        bytes.extend_from_slice(&self.path);
        bytes.extend_from_slice(&self.reserved.to_le_bytes());
        bytes.extend_from_slice(&self.id.to_le_bytes());
        if let Some(track) = &self.visibility_track {
            if track.tag != *b"KATV" {
                return Err(Error::MalformedRecord {
                    tag: Attachment::TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.encode()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: Attachment::TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
    }
}

impl Attachment {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"ATCH";
}
