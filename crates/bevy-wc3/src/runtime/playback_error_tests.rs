//! Playback retains typed evidence and suppresses only genuinely repeated inputs.
use super::record_playback_failure;
use crate::runtime::{Playback, PlaybackError};
use bevy::prelude::*;
use std::error::Error as _;
use std::io;
use wc3::model::error::{AnimationError, ErrorKind, Location, Segment};
use wc3::model::profiles::Mdlx;
use wc3::model::{Clip, Error as ModelError, Model};

fn invalid_time(clock: &'static str, value: f64) -> ModelError {
    ModelError::at(
        Location::new()
            .field("playback")
            .field(clock)
            .field("value"),
        AnimationError::NonFiniteTime { value },
    )
}

#[test]
fn repeated_time_failures_compare_bits_and_clock_context() {
    let nan = f64::from_bits(0x7ff8_0000_0000_0001);
    let different_nan = f64::from_bits(0x7ff8_0000_0000_0002);
    let error = PlaybackError(Some(invalid_time("clip_time", nan)));
    assert!(error.matches(&invalid_time("clip_time", nan)));
    assert!(!error.matches(&invalid_time("clip_time", different_nan)));
    assert!(!error.matches(&invalid_time("shared_time", nan)));
    let error = PlaybackError(Some(invalid_time("clip_time", f64::INFINITY)));
    assert!(error.matches(&invalid_time("clip_time", f64::INFINITY)));
    assert!(!error.matches(&invalid_time("clip_time", f64::NEG_INFINITY)));
}

#[test]
fn unknown_clip_deduplication_compares_the_actual_requested_identity() {
    let mut builder = Model::<Mdlx>::builder("Clips");
    let first = builder.add_clip(Clip::named("First")).unwrap();
    let second = builder.add_clip(Clip::named("Second")).unwrap();
    let failure = |clip| {
        ModelError::at(
            Location::new().field("clip"),
            AnimationError::UnknownClip { clip },
        )
    };
    let error = PlaybackError(Some(failure(first)));
    assert!(error.matches(&failure(first)));
    assert!(!error.matches(&failure(second)));
    assert!(
        matches!(error.0.unwrap().kind(), ErrorKind::Animation(AnimationError::UnknownClip { clip }) if *clip == first)
    );
}

#[test]
fn identical_display_text_does_not_equate_arbitrary_sources() {
    let failure = || {
        ModelError::extension(
            Location::new().field("custom"),
            io::Error::other("same explanation"),
        )
    };
    let error = PlaybackError(Some(failure()));
    assert!(!error.matches(&failure()));
    assert!(error
        .0
        .as_ref()
        .unwrap()
        .source()
        .unwrap()
        .is::<io::Error>());
}

#[test]
fn error_publication_preserves_change_detection_and_success_clears_it() {
    let mut world = World::new();
    let entity = world.spawn(PlaybackError::default()).id();
    let failure = || invalid_time("clip_time", f64::NAN);
    record_playback_failure(
        &mut world.get_mut::<PlaybackError>(entity).unwrap(),
        Some(failure()),
    );
    world.clear_trackers();
    record_playback_failure(
        &mut world.get_mut::<PlaybackError>(entity).unwrap(),
        Some(failure()),
    );
    assert!(!world
        .entity(entity)
        .get_ref::<PlaybackError>()
        .unwrap()
        .is_changed());
    record_playback_failure(
        &mut world.get_mut::<PlaybackError>(entity).unwrap(),
        Some(invalid_time("shared_time", f64::NAN)),
    );
    assert!(world
        .entity(entity)
        .get_ref::<PlaybackError>()
        .unwrap()
        .is_changed());
    world.clear_trackers();
    record_playback_failure(&mut world.get_mut::<PlaybackError>(entity).unwrap(), None);
    assert!(world.get::<PlaybackError>(entity).unwrap().0.is_none());
    assert!(world
        .entity(entity)
        .get_ref::<PlaybackError>()
        .unwrap()
        .is_changed());
    world.clear_trackers();
    record_playback_failure(&mut world.get_mut::<PlaybackError>(entity).unwrap(), None);
    assert!(!world
        .entity(entity)
        .get_ref::<PlaybackError>()
        .unwrap()
        .is_changed());
}

#[test]
fn clock_validation_adds_context_without_changing_rejected_inputs() {
    for (clock, value) in [("clip_time", f64::NAN), ("shared_time", f64::INFINITY)] {
        let mut playback = Playback::default();
        if clock == "clip_time" {
            playback.clip_time = value;
        } else {
            playback.shared_time = value;
        }
        let error = playback.selection().unwrap_err();
        assert!(
            matches!(&error.location().segments()[..2], [Segment::Field(playback), Segment::Field(input)] if playback == "playback" && input == clock)
        );
        assert_eq!(
            playback.clip_time.to_bits(),
            if clock == "clip_time" {
                value.to_bits()
            } else {
                0f64.to_bits()
            }
        );
        assert_eq!(
            playback.shared_time.to_bits(),
            if clock == "shared_time" {
                value.to_bits()
            } else {
                0f64.to_bits()
            }
        );
        assert!(error.source().is_some());
    }
}

#[test]
fn rejected_seek_has_distinct_context_and_leaves_both_clocks_unchanged() {
    let mut playback = Playback {
        clip_time: 3.,
        shared_time: 7.,
        ..Default::default()
    };
    let error = playback.seek(f64::NEG_INFINITY).unwrap_err();
    assert!(
        matches!(&error.location().segments()[..2], [Segment::Field(playback), Segment::Field(input)] if playback == "playback" && input == "seek")
    );
    assert_eq!(playback.clip_time, 3.);
    assert_eq!(playback.shared_time, 7.);
}
