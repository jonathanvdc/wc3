//! Timeline, channel, and interpolation conversion.
use super::*;
mod clock;
pub(super) use clock::{
    clip_starts, merge_track, native_globals, native_track, native_track_data, read_track,
    semantic_track, write_track,
};
mod import;
pub(super) use import::import_properties;
mod export;
pub(super) use clock::TrackData;
pub(super) use export::overlay_properties;

mod integer;
pub(super) use integer::overlay_integer;
