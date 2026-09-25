//! Event objects stored in `EVTS` chunks.

use crate::Record;
use crate::{Error, Model, Node};

const TRACK_TAG: [u8; 4] = *b"KEVT";

/// A node followed by an event track.
#[derive(Clone, Debug, PartialEq)]
pub struct EventObject {
    node: Node,
    global_sequence_id: u32,
    frames: Vec<u32>,
}

impl EventObject {
    /// Creates an event object from a node, global sequence ID, and frame times.
    pub fn new(node: Node, global_sequence_id: u32, frames: &[u32]) -> Result<Self, Error> {
        let event = Self {
            node,
            global_sequence_id,
            frames: frames.to_vec(),
        };
        event.encode()?;
        Ok(event)
    }

    /// Borrows its shared node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Borrows its shared node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Returns the global sequence ID, or `u32::MAX` when absent.
    pub fn global_sequence_id(&self) -> u32 {
        self.global_sequence_id
    }

    /// Sets the global sequence reference without changing event frames.
    pub fn set_global_sequence_id(&mut self, id: u32) {
        self.global_sequence_id = id;
    }

    /// Borrows event frame times in source order.
    pub fn frames(&self) -> &[u32] {
        &self.frames
    }

    /// Replaces event frame times.
    pub fn set_frames(&mut self, frames: &[u32]) -> Result<(), Error> {
        let mut replacement = self.clone();
        replacement.frames = frames.to_vec();
        replacement.encode()?;
        self.frames = replacement.frames;
        Ok(())
    }
}

fn record_end(data: &[u8], offset: usize) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord {
            tag: EventObject::TAG,
            offset,
        })?;
    let node_size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    let node_end = offset
        .checked_add(node_size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord {
            tag: EventObject::TAG,
            offset,
        })?;
    Node::decode(&data[offset..node_end], 0)?;
    let header = data
        .get(node_end..node_end.saturating_add(12))
        .ok_or(Error::MalformedRecord {
            tag: EventObject::TAG,
            offset: node_end,
        })?;
    if header[..4] != TRACK_TAG {
        return Err(Error::MalformedRecord {
            tag: EventObject::TAG,
            offset: node_end,
        });
    }
    let count = u32::from_le_bytes(header[4..8].try_into().expect("four-byte count")) as usize;
    node_end
        .checked_add(12)
        .and_then(|start| count.checked_mul(4).and_then(|n| start.checked_add(n)))
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord {
            tag: EventObject::TAG,
            offset: node_end,
        })
}

impl Model {
    /// Decodes all event objects in `EVTS` chunks.
    pub fn event_objects(&self) -> Result<Vec<EventObject>, Error> {
        let mut events = Vec::new();
        for chunk in self
            .chunks()
            .iter()
            .filter(|chunk| chunk.tag == EventObject::TAG)
        {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let end = record_end(&chunk.data, offset)?;
                events.push(EventObject::decode(&chunk.data[offset..end], 0)?);
                offset = end;
            }
        }
        Ok(events)
    }

    /// Replaces event objects in the first `EVTS` chunk.
    pub fn set_event_objects(&mut self, events: &[EventObject]) -> Result<(), Error> {
        let size = events.iter().try_fold(0usize, |sum, event| {
            sum.checked_add(event.encode()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: EventObject::TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for event in events {
            data.extend_from_slice(&event.encode()?);
        }
        self.replace_chunks(EventObject::TAG, data);
        Ok(())
    }
}

impl Record for EventObject {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        let end = record_end(bytes, 0)?;
        if end != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: EventObject::TAG,
                offset: end,
            });
        }
        let node_size =
            u32::from_le_bytes(bytes[..4].try_into().expect("validated node size")) as usize;
        let node = Node::decode(&bytes[..node_size], 0)?;
        let global_sequence_id = u32::from_le_bytes(
            bytes[node_size + 8..node_size + 12]
                .try_into()
                .expect("validated ID"),
        );
        let frames = bytes[node_size + 12..]
            .chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four-byte frame")))
            .collect();
        Ok(Self {
            node,
            global_sequence_id,
            frames,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = self.node.encode()?;
        bytes.extend_from_slice(&TRACK_TAG);
        let count = u32::try_from(self.frames.len()).map_err(|_| Error::ChunkTooLarge {
            tag: EventObject::TAG,
            size: self.frames.len(),
        })?;
        bytes.extend_from_slice(&count.to_le_bytes());
        bytes.extend_from_slice(&self.global_sequence_id.to_le_bytes());
        for frame in &self.frames {
            bytes.extend_from_slice(&frame.to_le_bytes());
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: EventObject::TAG,
                size: bytes.len(),
            });
        }
        Ok(bytes)
    }
}

impl EventObject {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"EVTS";
}
