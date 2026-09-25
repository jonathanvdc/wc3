//! Attachment records in `ATCH` chunks.

use crate::Record;
use std::borrow::Cow;

use crate::utils::field;
use crate::{AnimationTrack, ChunkRecord, Error, Model, Node};

const PATH_SIZE: usize = 256;
const FIXED_SIZE: usize = PATH_SIZE + 8;

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

fn record_end(data: &[u8], offset: usize) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord {
            tag: Attachment::TAG,
            offset,
        })?;
    let size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    let end = offset
        .checked_add(size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord {
            tag: Attachment::TAG,
            offset,
        })?;
    let node_start = offset + 4;
    let node_size_bytes =
        data.get(node_start..node_start.saturating_add(4))
            .ok_or(Error::MalformedRecord {
                tag: Attachment::TAG,
                offset: node_start,
            })?;
    let node_size =
        u32::from_le_bytes(node_size_bytes.try_into().expect("four-byte size")) as usize;
    let node_end = node_start
        .checked_add(node_size)
        .filter(|&node_end| node_end <= end)
        .ok_or(Error::MalformedRecord {
            tag: Attachment::TAG,
            offset: node_start,
        })?;
    Node::decode(&data[node_start..node_end], 0)?;
    if node_end
        .checked_add(FIXED_SIZE)
        .map_or(true, |required| required > end)
    {
        return Err(Error::MalformedRecord {
            tag: Attachment::TAG,
            offset: node_end,
        });
    }
    Ok(end)
}

impl Model {
    /// Decodes all attachments in `ATCH` chunks.
    pub fn attachments(&self) -> Result<Vec<Attachment>, Error> {
        let mut attachments = Vec::new();
        for chunk in self
            .chunks()
            .iter()
            .filter(|chunk| chunk.tag == Attachment::TAG)
        {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let end = record_end(&chunk.data, offset)?;
                attachments.push(Attachment::decode(&chunk.data[offset..end], 0)?);
                offset = end;
            }
        }
        Ok(attachments)
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
        self.replace_chunks(Attachment::TAG, data);
        Ok(())
    }
}

impl Record for Attachment {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        if record_end(bytes, 0)? != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: Attachment::TAG,
                offset: 0,
            });
        }
        let node_size =
            u32::from_le_bytes(bytes[4..8].try_into().expect("validated node size")) as usize;
        let fixed = 4 + node_size;
        let node = Node::decode(&bytes[4..fixed], 0)?;
        let path = bytes[fixed..fixed + PATH_SIZE]
            .try_into()
            .expect("validated path");
        let reserved = u32::from_le_bytes(
            bytes[fixed + PATH_SIZE..fixed + PATH_SIZE + 4]
                .try_into()
                .expect("validated reserved field"),
        );
        let id = u32::from_le_bytes(
            bytes[fixed + PATH_SIZE + 4..fixed + FIXED_SIZE]
                .try_into()
                .expect("validated ID"),
        );
        let offset = fixed + FIXED_SIZE;
        let visibility_track = if offset == bytes.len() {
            None
        } else {
            let (track, consumed) = AnimationTrack::parse(bytes, offset)?;
            if track.tag != *b"KATV" || offset + consumed != bytes.len() {
                return Err(Error::MalformedRecord {
                    tag: Attachment::TAG,
                    offset,
                });
            }
            Some(track)
        };
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

impl ChunkRecord for Attachment {
    const TAG: [u8; 4] = *b"ATCH";
}
