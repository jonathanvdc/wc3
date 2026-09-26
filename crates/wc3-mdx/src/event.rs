//! Event objects stored in `EVTS` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::Tag;

use crate::{Cursor, EventObjectsChunk};
use crate::{Decodable, Encodable};
use crate::{DecodeError, Model, Node};

const TRACK_TAG: Tag = *b"KEVT";

/// A node followed by an event track.
#[derive(Clone, Debug, PartialEq)]
pub struct EventObject {
    node: Node,
    global_sequence_id: u32,
    frames: Vec<u32>,
}

impl EventObject {
    /// Creates an event object from a node, global sequence ID, and frame times.
    pub fn new(node: Node, global_sequence_id: u32, frames: &[u32]) -> Self {
        Self {
            node,
            global_sequence_id,
            frames: frames.to_vec(),
        }
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
    pub fn set_frames(&mut self, frames: &[u32]) {
        self.frames = frames.to_vec();
    }
}

impl Model {
    /// Decodes all event objects in `EVTS` chunks.
    pub fn event_objects(&self) -> Vec<EventObject> {
        self.collect_chunk_records::<EventObjectsChunk>()
    }

    /// Replaces event objects in the first `EVTS` chunk.
    pub fn set_event_objects(&mut self, events: &[EventObject]) {
        self.replace_chunk(EventObjectsChunk::new(events.to_vec()));
    }
}

impl Decodable for EventObject {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, DecodeError> {
        let mut probe = *cursor;
        let node_size = probe.read::<u32>()? as usize;
        let node = Node::decode(cursor.read_exact(node_size)?, 0)?;
        let offset = cursor.absolute_position();
        if cursor.read_exact(4)? != TRACK_TAG {
            return Err(DecodeError::MalformedRecord {
                tag: Self::TAG,
                offset,
            });
        }
        let count = cursor.read::<u32>()? as usize;
        let global_sequence_id = cursor.read()?;
        let mut frames = Vec::new();
        for _ in 0..count {
            frames.push(cursor.read()?);
        }
        Ok(Self {
            node,
            global_sequence_id,
            frames,
        })
    }
}

impl Encodable for EventObject {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let start = bytes.position();
        self.node.encode_to(bytes)?;
        bytes.write_bytes(&TRACK_TAG);
        let count = u32::try_from(self.frames.len()).map_err(|_| EncodeError::ChunkTooLarge {
            tag: EventObject::TAG,
            size: self.frames.len(),
        })?;
        bytes.write(count);
        bytes.write(self.global_sequence_id);
        for frame in &self.frames {
            bytes.write(frame);
        }
        if bytes.position() - start > u32::MAX as usize {
            return Err(EncodeError::ChunkTooLarge {
                tag: EventObject::TAG,
                size: bytes.position() - start,
            });
        }
        Ok(())
    }
}

impl EventObject {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"EVTS";
}
