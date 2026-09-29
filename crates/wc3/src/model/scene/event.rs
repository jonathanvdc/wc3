//! Timed events attached to model nodes.
use super::{set_node_kind, validate_node_kind};
use crate::model::mdl::{Dialect, Field, MdlWriter, Parser, Span, TokenKind};
use crate::model::Encoder;
use crate::model::KnownChunk;
use crate::model::ModelVersion;
use crate::model::Tag;
use crate::model::WriteError;
use crate::model::{mdl, mdx};
use std::io::Write as IoWrite;

use crate::model::{Cursor, EventObjectsChunk};
use crate::model::{Model, Node, ReadError};

const TRACK_TAG: Tag = *b"KEVT";

/// A named event triggered at specific times in an animation.
///
/// Event names identify game effects such as sounds or footprints. Frame times
/// use milliseconds; a global sequence makes the event loop independently.
#[derive(Clone, Debug, PartialEq)]
pub struct EventObject {
    /// Its shared node.
    pub node: Node,
    /// Global sequence ID, or `u32::MAX` when absent.
    pub global_sequence_id: u32,
    /// Event times in milliseconds, in source order. Negative times allow animation lead-in.
    pub frames: Vec<i32>,
}

impl EventObject {
    /// Creates an event object from a node, global sequence ID, and frame times.
    pub fn new(node: Node, global_sequence_id: u32, frames: &[i32]) -> Self {
        Self {
            node,
            global_sequence_id,
            frames: frames.to_vec(),
        }
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of event objects in `EVTS` chunks.
    pub fn event_objects(&self) -> Vec<EventObject> {
        self.collect_chunk_records::<EventObjectsChunk>()
    }

    /// Replaces event objects in the first `EVTS` chunk.
    pub fn set_event_objects(&mut self, events: &[EventObject]) {
        self.replace_chunk(EventObjectsChunk::new(events.to_vec()));
    }
}

impl mdx::Read for EventObject {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let mut probe = *cursor;
        let node_size = probe.read::<u32>()? as usize;
        let node = Node::decode_mdx(cursor.read_exact(node_size)?)?;
        let offset = cursor.absolute_position();
        if cursor.read_exact(4)? != TRACK_TAG {
            return Err(ReadError::MalformedRecord {
                tag: EventObjectsChunk::TAG,
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

impl mdx::Write for EventObject {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        let start = bytes.position();
        bytes.write(&self.node)?;
        bytes.write_bytes(&TRACK_TAG);
        let count = u32::try_from(self.frames.len()).map_err(|_| WriteError::ChunkTooLarge {
            tag: EventObjectsChunk::TAG,
            size: self.frames.len(),
        })?;
        bytes.write(&(count))?;
        bytes.write(&(self.global_sequence_id))?;
        for frame in &self.frames {
            bytes.write(frame)?;
        }
        if bytes.position() - start > u32::MAX as usize {
            return Err(WriteError::ChunkTooLarge {
                tag: EventObjectsChunk::TAG,
                size: bytes.position() - start,
            });
        }
        Ok(())
    }
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(entry)]
struct EventFrame(i32);
struct EventTrackMdl {
    sequence: u32,
    frames: Vec<i32>,
}
impl mdl::ReadProperty for EventTrackMdl {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        let (sequence, items) = parser.counted_with_header::<EventFrame, _>(|parser| {
            if parser
                .peek()?
                .is_some_and(|token| token.kind == TokenKind::Ident("GlobalSeqId"))
            {
                parser.next_token()?;
                let sequence = parser.read_property()?;
                if parser
                    .peek()?
                    .is_some_and(|token| token.kind == TokenKind::Ident("GlobalSeqId"))
                {
                    return Err(parser.error(mdl::ReadErrorKind::DuplicateField));
                }
                Ok(sequence)
            } else {
                Ok(u32::MAX)
            }
        })?;
        let frames = items
            .map(|item| item.map(|frame| frame.0))
            .collect::<Result<_, _>>()?;
        Ok(Self { sequence, frames })
    }
}
impl mdl::WriteProperty for EventTrackMdl {
    fn validate_mdl_property(&self, _: &'static str, _: Dialect) -> Result<(), mdl::WriteError> {
        if self.frames.len() > u32::MAX as usize {
            return Err(mdl::WriteError::Unsupported("event track count"));
        }
        Ok(())
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut MdlWriter<W>,
    ) -> Result<(), mdl::WriteError> {
        self.validate_mdl_property(name, writer.dialect())?;
        writer.begin_counted_block(name, self.frames.len())?;
        if self.sequence != u32::MAX {
            writer.property("GlobalSeqId", &self.sequence)?;
        }
        for frame in &self.frames {
            writer.entry(frame)?;
        }
        writer.end_block()
    }
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(
    block = "EventObject",
    after_read = "finish_event",
    validate_write = "validate_event"
)]
struct EventMdl {
    #[mdl(flatten)]
    node: Node,
    #[mdl(property = "EventTrack", delegate)]
    track: EventTrackMdl,
}
fn finish_event(value: &mut EventMdl, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x400);
    Ok(())
}
fn validate_event(value: &EventMdl) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x400)
}
impl mdl::Read for EventObject {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let value = parser.read::<EventMdl>()?;
        Ok(Self {
            node: value.node,
            global_sequence_id: value.track.sequence,
            frames: value.track.frames,
        })
    }
}
impl mdl::Write for EventObject {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        writer.write(&EventMdl {
            node: self.node.clone(),
            track: EventTrackMdl {
                sequence: self.global_sequence_id,
                frames: self.frames.clone(),
            },
        })
    }
}
