//! Timed events attached to model nodes.
use super::{set_node_kind, validate_node_kind};
use crate::model::common::Ownership;
use crate::model::formats::mdlx::mdl::{
    Dialect as TextDialect, Field, Parser, Span, TokenKind, Writer,
};
use crate::model::formats::mdlx::mdx::read_array;
use crate::model::formats::mdlx::Dialect;
use crate::model::formats::mdlx::IoError;
use crate::model::formats::mdlx::KnownChunk;
use crate::model::formats::mdlx::Tag;
use crate::model::formats::mdlx::{mdl, mdx};
use crate::model::formats::mdlx::{Cursor, EventObjectsChunk};
use crate::model::formats::mdlx::{Document, Node};
use crate::model::formats::mdlx::{Encoder, ZeroVec};
use std::io::Write as IoWrite;

const TRACK_TAG: Tag = *b"KEVT";

/// A named event triggered at specific times in an animation.
///
/// Event names identify game effects such as sounds or footprints. Frame times
/// use milliseconds; a global sequence makes the event loop independently.
#[derive(Clone, Debug, PartialEq, Ownership, mdx::ReadOwned)]
pub struct EventObject<'data> {
    /// Its shared node.
    pub node: Node<'data>,
    /// Global sequence ID, or `u32::MAX` when absent.
    pub global_sequence_id: u32,
    /// Stored event times in milliseconds. Negative times allow animation lead-in.
    frames: ZeroVec<'data, i32>,
}

impl<'data> EventObject<'data> {
    /// Creates an event object, retaining its authored frame order.
    /// Duplicate timestamps are retained.
    pub fn new(node: Node<'data>, global_sequence_id: u32, frames: &[i32]) -> Self {
        Self {
            node,
            global_sequence_id,
            frames: ZeroVec::alloc_from_slice(frames),
        }
    }

    /// Returns event times in stored order, including duplicates.
    pub fn frames(&self) -> &ZeroVec<'data, i32> {
        &self.frames
    }

    /// Replaces event times without changing their order.
    pub fn set_frames(&mut self, frames: &[i32]) {
        self.frames = ZeroVec::alloc_from_slice(frames);
    }
}

impl<'data, D: Dialect> Document<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn event_objects<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s EventObject<'data>> + use<'data, 's, D> {
        self.records::<EventObjectsChunk<'_>>()
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn event_objects_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut EventObject<'data>> + use<'data, 's, D> {
        self.records_mut::<EventObjectsChunk<'_>>()
    }

    /// Replaces event objects with one `EVTS` chunk, removing any duplicate chunks.
    pub fn set_event_objects(&mut self, events: impl IntoIterator<Item = EventObject<'data>>) {
        self.replace_chunk(EventObjectsChunk::new(events.into_iter().collect()));
    }
}

impl<'data> mdx::Read<'data> for EventObject<'data> {
    fn read_mdx(cursor: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        let mut probe = *cursor;
        let node_size = probe.read::<u32>()? as usize;
        let mut node_cursor = cursor.subcursor(node_size)?;
        let node = node_cursor.read::<Node<'data>>()?;
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
        let size = count.checked_mul(4).ok_or_else(|| {
            mdx::ReadError::new(cursor.absolute_position(), mdx::ReadErrorKind::SizeOverflow)
        })?;
        let frames = read_array::<i32>(cursor.subcursor(size)?, 4)?;
        Ok(Self {
            node,
            global_sequence_id,
            frames,
        })
    }
}

impl<'data> mdx::Write for EventObject<'data> {
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
        bytes.write_bytes(self.frames.as_bytes());
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
struct EventTrackMdl<'data> {
    sequence: u32,
    frames: ZeroVec<'data, i32>,
}
impl mdl::ReadProperty for EventTrackMdl<'_> {
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
impl mdl::WriteProperty for EventTrackMdl<'_> {
    fn validate_mdl_property(
        &self,
        _: &'static str,
        _: TextDialect,
    ) -> Result<(), mdl::WriteError> {
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
        for frame in self.frames.iter() {
            writer.entry(&frame)?;
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
struct EventMdl<'data> {
    #[mdl(flatten)]
    node: Node<'data>,
    #[mdl(property = "EventTrack", delegate)]
    track: EventTrackMdl<'data>,
}
fn finish_event<'data>(value: &mut EventMdl<'data>, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x400);
    Ok(())
}
fn validate_event<'data>(value: &EventMdl<'data>) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x400)
}
impl<'data> mdl::Read for EventObject<'data> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let value = parser.read::<EventMdl<'data>>()?;
        let frames = value.track.frames;
        Ok(Self {
            node: value.node,
            global_sequence_id: value.track.sequence,
            frames,
        })
    }
}
impl<'data> mdl::Write for EventObject<'data> {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        writer.write(&EventMdl {
            node: self.node.as_borrowed(),
            track: EventTrackMdl {
                sequence: self.global_sequence_id,
                frames: self.frames.as_slice().as_zerovec(),
            },
        })
    }
}
