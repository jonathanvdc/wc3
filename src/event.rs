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

impl Model {
    /// Decodes all event objects in `EVTS` chunks.
    pub fn event_objects(&self) -> Result<Vec<EventObject>, Error> {
        self.collect_chunk_records::<crate::EventObjectsChunk>(|chunk| match chunk {
            crate::ModelChunk::EventObjects(decoded) => Some(decoded),
            _ => None,
        })
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
        self.replace_raw_chunk(EventObject::TAG, data)?;
        Ok(())
    }
}

impl Record for EventObject {
    fn decode_one(cursor: &mut crate::Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut probe = *cursor;
        let node_size = probe.read_u32()? as usize;
        let node = Node::decode(cursor.read_exact(node_size)?, 0)?;
        let offset = cursor.absolute_position();
        if cursor.read_exact(4)? != TRACK_TAG {
            return Err(Error::MalformedRecord {
                tag: Self::TAG,
                offset,
            });
        }
        let count = cursor.read_u32()? as usize;
        let global_sequence_id = cursor.read_u32()?;
        let mut frames = Vec::new();
        for _ in 0..count {
            frames.push(cursor.read_u32()?);
        }
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
