//! Attachment records in `ATCH` chunks.

use std::borrow::Cow;

use crate::{AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"ATCH";
const PATH_SIZE: usize = 256;
const FIXED_SIZE: usize = PATH_SIZE + 8;

/// An attachment node with a model path and optional visibility track.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attachment {
    bytes: Vec<u8>,
}

impl Attachment {
    /// Creates an attachment from a node, model path, and attachment ID.
    pub fn new(node: Node, path: &str, id: u32) -> Result<Self, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(node.as_bytes());
        bytes.resize(bytes.len() + FIXED_SIZE, 0);
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        let mut attachment = Self { bytes };
        attachment.set_path(path)?;
        attachment.set_id(id);
        Ok(attachment)
    }

    /// Wraps one inclusive-size attachment record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if record_end(bytes, 0)? != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the shared node.
    pub fn node(&self) -> Node {
        let size = self.node_size();
        Node::from_bytes(&self.bytes[4..4 + size]).expect("validated node")
    }

    /// Returns the model path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        let start = self.fixed_offset();
        let field = &self.bytes[start..start + PATH_SIZE];
        let end = field
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(PATH_SIZE);
        String::from_utf8_lossy(&field[..end])
    }

    /// Sets the model path without changing reserved bytes.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        if path.len() >= PATH_SIZE || path.as_bytes().contains(&0) {
            return Err(Error::InvalidString {
                max_bytes: PATH_SIZE - 1,
            });
        }
        let start = self.fixed_offset();
        self.bytes[start..start + PATH_SIZE].fill(0);
        self.bytes[start..start + path.len()].copy_from_slice(path.as_bytes());
        Ok(())
    }

    /// Returns the attachment ID.
    pub fn id(&self) -> u32 {
        let offset = self.fixed_offset() + PATH_SIZE + 4;
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("four-byte ID"),
        )
    }

    /// Sets the attachment ID.
    pub fn set_id(&mut self, id: u32) {
        let offset = self.fixed_offset() + PATH_SIZE + 4;
        self.bytes[offset..offset + 4].copy_from_slice(&id.to_le_bytes());
    }

    /// Returns the optional visibility track, if present.
    pub fn visibility_track(&self) -> Result<Option<AnimationTrack>, Error> {
        let offset = self.fixed_offset() + FIXED_SIZE;
        if offset == self.bytes.len() {
            return Ok(None);
        }
        let (track, consumed) = AnimationTrack::parse(&self.bytes, offset)?;
        if track.tag != *b"KATV" || offset + consumed != self.bytes.len() {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        Ok(Some(track))
    }

    /// Replaces the optional visibility track and updates the record size.
    pub fn set_visibility_track(&mut self, track: Option<&AnimationTrack>) -> Result<(), Error> {
        let start = self.fixed_offset() + FIXED_SIZE;
        let mut bytes = self.bytes[..start].to_vec();
        if let Some(track) = track {
            if track.tag != *b"KATV" {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: start,
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }

    fn node_size(&self) -> usize {
        u32::from_le_bytes(self.bytes[4..8].try_into().expect("validated node size")) as usize
    }

    fn fixed_offset(&self) -> usize {
        4 + self.node_size()
    }
}

fn record_end(data: &[u8], offset: usize) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
    let size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    let end = offset
        .checked_add(size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
    let node_start = offset + 4;
    let node_size_bytes =
        data.get(node_start..node_start.saturating_add(4))
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: node_start,
            })?;
    let node_size =
        u32::from_le_bytes(node_size_bytes.try_into().expect("four-byte size")) as usize;
    let node_end = node_start
        .checked_add(node_size)
        .filter(|&node_end| node_end <= end)
        .ok_or(Error::MalformedRecord {
            tag: TAG,
            offset: node_start,
        })?;
    Node::from_bytes(&data[node_start..node_end])?;
    if node_end
        .checked_add(FIXED_SIZE)
        .map_or(true, |required| required > end)
    {
        return Err(Error::MalformedRecord {
            tag: TAG,
            offset: node_end,
        });
    }
    Ok(end)
}

impl Model {
    /// Decodes all attachments in `ATCH` chunks.
    pub fn attachments(&self) -> Result<Vec<Attachment>, Error> {
        let mut attachments = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let end = record_end(&chunk.data, offset)?;
                attachments.push(Attachment::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(attachments)
    }

    /// Replaces attachments in the first `ATCH` chunk.
    pub fn set_attachments(&mut self, attachments: &[Attachment]) -> Result<(), Error> {
        let size = attachments.iter().try_fold(0usize, |sum, attachment| {
            sum.checked_add(attachment.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for attachment in attachments {
            data.extend_from_slice(attachment.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
