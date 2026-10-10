//! Timed events attached to model nodes.
use super::{set_node_kind, validate_node_kind};
use crate::model::mdl::{Dialect, Field, Parser, Span, TokenKind, Writer};
use crate::model::Encoder;
use crate::model::IoError;
use crate::model::KnownChunk;
use crate::model::ModelDialect;
use crate::model::Tag;
use crate::model::{mdl, mdx};
use crate::model::{Cursor, EventObjectsChunk};
use crate::model::{Model, Node};
use std::io::Write as IoWrite;

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
    /// Sorted event times in milliseconds. Negative times allow animation lead-in.
    frames: Vec<i32>,
}

impl EventObject {
    /// Creates an event object, sorting its frame times in ascending order.
    /// Duplicate timestamps are retained.
    pub fn new(node: Node, global_sequence_id: u32, frames: &[i32]) -> Self {
        let mut frames = frames.to_vec();
        frames.sort();
        Self {
            node,
            global_sequence_id,
            frames,
        }
    }

    /// Returns the event times in ascending order, including duplicates.
    pub fn frames(&self) -> &[i32] {
        &self.frames
    }

    /// Replaces the event times, sorting them in ascending order.
    pub fn set_frames(&mut self, frames: &[i32]) {
        self.frames = frames.to_vec();
        self.frames.sort();
    }
}

impl<D: ModelDialect> Model<D> {
    /// Returns owned copies of event objects in `EVTS` chunks.
    pub fn event_objects(&self) -> Vec<EventObject> {
        self.collect_chunk_records::<EventObjectsChunk>()
    }

    /// Replaces event objects with one `EVTS` chunk, removing any duplicate chunks.
    pub fn set_event_objects(&mut self, events: &[EventObject]) {
        self.replace_chunk(EventObjectsChunk::new(events.to_vec()));
    }
}

impl mdx::Read for EventObject {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        let mut probe = *cursor;
        let node_size = probe.read::<u32>()? as usize;
        let mut node_cursor = cursor.subcursor(node_size)?;
        let node = node_cursor.read::<Node>()?;
        node_cursor.finish()?;
        let offset = cursor.absolute_position();
        let actual = cursor.read()?;
        if actual != TRACK_TAG {
            return Err(mdx::ReadError::new(
                offset,
                mdx::ReadErrorKind::UnexpectedTag {
                    expected: TRACK_TAG,
                    actual,
                },
            )
            .with_tag(EventObjectsChunk::TAG));
        }
        let count = cursor.read::<u32>()? as usize;
        let global_sequence_id = cursor.read()?;
        let mut frames = Vec::new();
        for _ in 0..count {
            frames.push(cursor.read()?);
        }
        Ok(Self::new(node, global_sequence_id, &frames))
    }
}

impl mdx::Write for EventObject {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        let start = bytes.position();
        bytes.write(&self.node)?;
        bytes.write_bytes(&TRACK_TAG);
        let count =
            u32::try_from(self.frames.len()).map_err(|_| mdx::WriteError::SizeOverflow {
                field: "frame count",
                tag: EventObjectsChunk::TAG,
                size: self.frames.len(),
            })?;
        bytes.write(&(count))?;
        bytes.write(&(self.global_sequence_id))?;
        for frame in &self.frames {
            bytes.write(frame)?;
        }
        if bytes.position() - start > u32::MAX as usize {
            return Err(mdx::WriteError::SizeOverflow {
                field: "encoded size",
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
            return Err(mdl::WriteError::SizeOverflow {
                field: "event track count",
            });
        }
        Ok(())
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
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
        Ok(Self::new(
            value.node,
            value.track.sequence,
            &value.track.frames,
        ))
    }
}
impl mdl::Write for EventObject {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        writer.write(&EventMdl {
            node: self.node.clone(),
            track: EventTrackMdl {
                sequence: self.global_sequence_id,
                frames: self.frames.clone(),
            },
        })
    }
}
