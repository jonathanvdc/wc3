//! Event objects stored in `EVTS` chunks.

use crate::{Error, Model, Node};

const TAG: [u8; 4] = *b"EVTS";
const TRACK_TAG: [u8; 4] = *b"KEVT";

/// A node followed by an event track.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventObject {
    bytes: Vec<u8>,
}

impl EventObject {
    /// Creates an event object from a node, global sequence ID, and frame times.
    pub fn new(node: Node, global_sequence_id: u32, frames: &[u32]) -> Result<Self, Error> {
        let mut bytes = node.to_bytes();
        bytes.extend_from_slice(&TRACK_TAG);
        if frames.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            });
        }
        bytes.extend_from_slice(&(frames.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&global_sequence_id.to_le_bytes());
        for frame in frames {
            bytes.extend_from_slice(&frame.to_le_bytes());
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        Ok(Self { bytes })
    }

    /// Wraps one complete event-object record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let end = record_end(bytes, 0)?;
        if end != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: end,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete event-object record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns its shared node record.
    pub fn node(&self) -> Node {
        let size =
            u32::from_le_bytes(self.bytes[..4].try_into().expect("validated node size")) as usize;
        Node::from_bytes(&self.bytes[..size]).expect("validated node")
    }

    /// Returns the global sequence ID, or `u32::MAX` when absent.
    pub fn global_sequence_id(&self) -> u32 {
        let offset = self.node_size() + 8;
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("validated field"),
        )
    }

    /// Sets the global sequence reference without changing event frames.
    pub fn set_global_sequence_id(&mut self, id: u32) {
        let offset = self.node_size() + 8;
        self.bytes[offset..offset + 4].copy_from_slice(&id.to_le_bytes());
    }

    /// Returns event frame times in source order.
    pub fn frames(&self) -> Vec<u32> {
        let start = self.node_size() + 12;
        self.bytes[start..]
            .chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four-byte frame")))
            .collect()
    }

    /// Replaces event frame times and updates their count.
    pub fn set_frames(&mut self, frames: &[u32]) -> Result<(), Error> {
        if frames.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: frames.len(),
            });
        }
        let start = self.node_size();
        let size = start
            .checked_add(12)
            .and_then(|n| frames.len().checked_mul(4).and_then(|m| n.checked_add(m)))
            .filter(|&size| size <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            })?;
        let mut bytes = self.bytes[..start + 12].to_vec();
        bytes[start + 4..start + 8].copy_from_slice(&(frames.len() as u32).to_le_bytes());
        bytes.reserve(size - bytes.len());
        for frame in frames {
            bytes.extend_from_slice(&frame.to_le_bytes());
        }
        self.bytes = bytes;
        Ok(())
    }

    fn node_size(&self) -> usize {
        u32::from_le_bytes(self.bytes[..4].try_into().expect("validated node size")) as usize
    }
}

fn record_end(data: &[u8], offset: usize) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
    let node_size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    let node_end = offset
        .checked_add(node_size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
    Node::from_bytes(&data[offset..node_end])?;
    let header = data
        .get(node_end..node_end.saturating_add(12))
        .ok_or(Error::MalformedRecord {
            tag: TAG,
            offset: node_end,
        })?;
    if header[..4] != TRACK_TAG {
        return Err(Error::MalformedRecord {
            tag: TAG,
            offset: node_end,
        });
    }
    let count = u32::from_le_bytes(header[4..8].try_into().expect("four-byte count")) as usize;
    node_end
        .checked_add(12)
        .and_then(|start| count.checked_mul(4).and_then(|n| start.checked_add(n)))
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord {
            tag: TAG,
            offset: node_end,
        })
}

impl Model {
    /// Decodes all event objects in `EVTS` chunks.
    pub fn event_objects(&self) -> Result<Vec<EventObject>, Error> {
        let mut events = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let end = record_end(&chunk.data, offset)?;
                events.push(EventObject::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(events)
    }

    /// Replaces event objects in the first `EVTS` chunk.
    pub fn set_event_objects(&mut self, events: &[EventObject]) -> Result<(), Error> {
        let size = events.iter().try_fold(0usize, |sum, event| {
            sum.checked_add(event.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for event in events {
            data.extend_from_slice(event.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
