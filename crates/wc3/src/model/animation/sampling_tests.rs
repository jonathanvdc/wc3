use super::*;
use crate::model::ModelMetadata;
use crate::model::{Animation, Clip, Curve, ModelBuilder, PackedArray, SharedClock, TcbParameters};
use crate::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use glam::Vec3;
use std::ptr::eq;
fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
}
fn dynamic_track(property: Property<Vec3>, clock: TimeSource, end: f32) -> DynamicTrack<'static> {
    DynamicTrack::Vector3(Track {
        target: property,
        curve: Curve::linear(
            [time(0.), time(1.), time(2.)],
            ([[0.; 3], [end / 2.; 3], [end; 3]])
                .into_iter()
                .map(Vec3::from_array)
                .collect::<Vec<_>>(),
        )
        .unwrap(),
        clock,
    })
}
#[test]
fn frozen_shared_clock_keeps_authored_origin_without_a_loop_period() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("ornament").unwrap();
    let node_translation = builder.model().node(node).unwrap().transform.translation;
    let _node_scale = builder.model().node(node).unwrap().transform.scale;
    let clock = builder.add_clock(SharedClock::Frozen).unwrap();
    builder
        .add_global_track(DynamicTrack::Vector3(Track {
            target: node_translation,
            curve: Curve::linear(
                [time(0.), time(2.)],
                ([[3.; 3], [9.; 3]])
                    .into_iter()
                    .map(Vec3::from_array)
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            clock: TimeSource::Shared(clock),
        }))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let program = AnimationProgram::compile(&model);
    for elapsed in [0., 1., 100.] {
        let sampled = program
            .sample(AnimationSample {
                shared_time: time(elapsed),
                ..SlotConstruct::construct(&mut slot_values)
            })
            .unwrap();
        assert_eq!(
            sampled.override_value(&node_translation),
            Some(Vec3::from_array([3.; 3]))
        );
        let track = sampled.track(&node_translation).unwrap();
        assert_eq!(track.time, time(0.));
        assert_eq!(track.period, None);
    }
}
#[test]
fn composed_programs_share_clocks_and_keep_property_owners_disjoint() {
    let mut slot_values = SlotValues::default();
    let mut host = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = host.add_named_node("host bone").unwrap();
    let node_translation = host.model().node(node).unwrap().transform.translation;
    let _node_scale = host.model().node(node).unwrap().transform.scale;
    let clip = host
        .add_clip(Clip::named("host clip").with_duration(time(2.)))
        .unwrap();
    host.add_track(
        clip,
        dynamic_track(node_translation, TimeSource::Animation, 2.),
    )
    .unwrap();
    let unmapped_clip = host
        .add_clip(Clip::named("unmapped clip").with_duration(time(2.)))
        .unwrap();
    let host = AnimationProgram::compile(
        &{
            host.import_values(&slot_values).unwrap();
            host.finish()
        }
        .unwrap(),
    );
    let mut part = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let extra = part.add_named_node("part bone").unwrap();
    let extra_translation = part.model().node(extra).unwrap().transform.translation;
    let extra_scale = part.model().node(extra).unwrap().transform.scale;
    let part_clip = part
        .add_clip(Clip::named("part clip").with_duration(time(2.)))
        .unwrap();
    part.add_track(
        part_clip,
        dynamic_track(extra_translation, TimeSource::Animation, 4.),
    )
    .unwrap();
    part.add_global_track(dynamic_track(extra_scale, TimeSource::Animation, 6.))
        .unwrap();
    let part = AnimationProgram::compile(
        &{
            part.import_values(&slot_values).unwrap();
            part.finish()
        }
        .unwrap(),
    );
    let combined = host.compose(&part, &[(clip, part_clip)], |_| true).unwrap();
    let sampled = combined
        .sample(AnimationSample {
            clip: Some(clip),
            clip_time: time(1.),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    assert_eq!(
        sampled.override_value(&node_translation),
        Some(Vec3::from_array([1.; 3]))
    );
    assert_eq!(
        sampled.override_value(&extra_translation),
        Some(Vec3::from_array([2.; 3]))
    );
    assert_eq!(
        sampled.override_value(&extra_scale),
        Some(Vec3::from_array([3.; 3]))
    );
    let globals = combined
        .sample(AnimationSample {
            clip_time: time(1.),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(globals.override_value(&node_translation), None);
    assert_eq!(globals.override_value(&extra_translation), None);
    assert_eq!(globals.override_value(&extra_scale), Some(Vec3::splat(3.)));
    let unmapped = combined
        .sample(AnimationSample {
            clip: Some(unmapped_clip),
            clip_time: time(1.),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(unmapped.override_value(&node_translation), None);
    assert_eq!(unmapped.override_value(&extra_translation), None);
    assert_eq!(unmapped.override_value(&extra_scale), Some(Vec3::splat(3.)));
    let filtered = host
        .compose(&part, &[(clip, part_clip)], |track| {
            track.target() != extra_translation.slot()
        })
        .unwrap();
    let filtered = filtered
        .sample(AnimationSample {
            clip: Some(clip),
            clip_time: time(1.),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        filtered.override_value(&node_translation),
        Some(Vec3::splat(1.))
    );
    assert_eq!(filtered.override_value(&extra_translation), None);
    assert_eq!(filtered.override_value(&extra_scale), Some(Vec3::splat(3.)));
    assert!(host
        .compose(&part, &[(clip, part_clip), (clip, part_clip)], |_| true)
        .is_err());
    assert!(host
        .compose(&part, &[(part_clip, part_clip)], |_| true)
        .is_err());
    assert!(host.compose(&host, &[(clip, clip)], |_| true).is_err());
    assert!(host
        .sample(AnimationSample {
            clip: Some(clip),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap()
        .override_value(&extra_translation)
        .is_none());
}
#[test]
fn event_boundary_override_keeps_own_clock_end_and_other_clock_start() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("bone").unwrap();
    let node_translation = builder.model().node(node).unwrap().transform.translation;
    let node_scale = builder.model().node(node).unwrap().transform.scale;
    let clock = builder
        .add_clock(SharedClock::Repeating { period: time(2.) })
        .unwrap();
    let clip = builder
        .add_clip(Clip::named("loop").with_duration(time(2.)))
        .unwrap();
    builder
        .add_track(
            clip,
            dynamic_track(node_translation, TimeSource::Animation, 4.),
        )
        .unwrap();
    builder
        .add_global_track(dynamic_track(node_scale, TimeSource::Shared(clock), 8.))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let sampled = model
        .animation_sample(AnimationSample {
            clip: Some(clip),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let end = sampled
        .at_times(time(2.), time(2.))
        .at_occurrence(TimeSource::Animation, time(2.));
    assert_eq!(
        end.override_value(&node_translation),
        Some(Vec3::from_array([4.; 3]))
    );
    assert_eq!(
        end.override_value(&node_scale),
        Some(Vec3::from_array([0.; 3]))
    );
    assert_eq!(end.track(&node_translation).unwrap().time, time(2.));
    let shared_end = sampled
        .at_times(time(2.), time(2.))
        .at_occurrence(TimeSource::Shared(clock), time(2.));
    assert_eq!(
        shared_end.override_value(&node_translation),
        Some(Vec3::from_array([0.; 3]))
    );
    assert_eq!(
        shared_end.override_value(&node_scale),
        Some(Vec3::from_array([8.; 3]))
    );
}
#[test]
fn alias_local_clock_and_owned_program_survive_source_drop() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("bone").unwrap();
    let node_translation = builder.model().node(node).unwrap().transform.translation;
    let _node_scale = builder.model().node(node).unwrap().transform.scale;
    let property = node_translation;
    let original = builder
        .add_clip(Clip::named("original").with_duration(time(2.)))
        .unwrap();
    builder
        .add_track(original, dynamic_track(property, TimeSource::Animation, 4.))
        .unwrap();
    let alias = builder
        .add_clip(
            Clip::named("alias")
                .with_animation(builder.model().clip(original).unwrap().animations[0])
                .with_duration(time(1.))
                .once(),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let sample = AnimationSample {
        clip: Some(alias),
        clip_time: time(8.),
        ..SlotConstruct::construct(&mut slot_values)
    };
    assert_eq!(
        model
            .animation_sample(sample)
            .unwrap()
            .override_value(&property),
        Some(Vec3::from_array([2.; 3]))
    );
    let program = AnimationProgram::compile(&model);
    drop(model);
    let sampled = program.sample(sample).unwrap();
    let TrackSelection::Indexed(selection) = &sampled.tracks else {
        panic!("owned programs must use indexed selection")
    };
    assert_eq!(selection.matching(property.slot()).count(), 1);
    assert_eq!(
        sampled.override_value(&property),
        Some(Vec3::from_array([2.; 3]))
    );
}
#[test]
fn independent_clocks_are_explicit() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("bone").unwrap();
    let node_translation = builder.model().node(node).unwrap().transform.translation;
    let node_scale = builder.model().node(node).unwrap().transform.scale;
    let translation = node_translation;
    let scale = node_scale;
    let clock = builder
        .add_clock(SharedClock::Repeating { period: time(2.) })
        .unwrap();
    builder
        .add_global_track(dynamic_track(translation, TimeSource::Animation, 4.))
        .unwrap();
    builder
        .add_global_track(dynamic_track(scale, TimeSource::Shared(clock), 8.))
        .unwrap();
    let fallback = builder.add_value(7f32);
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let sample = AnimationSample {
        clip_time: time(1.),
        shared_time: time(2.5),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let selected = model.animation_sample(sample).unwrap();
    assert_eq!(
        selected.override_value(&translation),
        Some(Vec3::from_array([2.; 3]))
    );
    assert_eq!(
        selected.override_value(&scale),
        Some(Vec3::from_array([2.; 3]))
    );
    assert_eq!(selected.animated(&fallback), 7.);
    let program = AnimationProgram::compile(&model);
    assert_eq!(
        program.sample(sample).unwrap().override_value(&translation),
        selected.override_value(&translation)
    );
}
#[test]
fn indexed_empty_override_falls_back_and_timeline_keys_are_bounded() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("bone").unwrap();
    let node_translation = builder.model().node(node).unwrap().transform.translation;
    let _node_scale = builder.model().node(node).unwrap().transform.scale;
    let property = node_translation;
    builder
        .add_global_track(DynamicTrack::Vector3(Track {
            target: property,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                [time(0.), time(1.), time(4.)],
                ([[0.; 3], [2.; 3], [100.; 3]])
                    .into_iter()
                    .map(Vec3::from_array)
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        }))
        .unwrap();
    let animation_2 = builder.add_animation(crate::model::Animation {
        tracks: vec![DynamicTrack::Vector3(Track {
            target: property,
            clock: TimeSource::Animation,
            curve: Curve::step(Vec::<Time>::new(), Vec::<Vec3>::new()).unwrap(),
        })],
        ..Default::default()
    });
    let clip = builder
        .add_clip(
            Clip::named("bounded")
                .with_duration(time(2.))
                .once()
                .with_source_start(time(0.))
                .with_animation(animation_2),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = AnimationSample {
        clip: Some(clip),
        clip_time: time(2.),
        ..SlotConstruct::construct(&mut slot_values)
    };
    assert_eq!(
        program.sample(sample).unwrap().override_value(&property),
        Some(Vec3::from_array([2.; 3]))
    );
    assert_eq!(
        model
            .animation_sample(sample)
            .unwrap()
            .override_value(&property),
        Some(Vec3::from_array([2.; 3]))
    );
}

fn assert_same_track<T: TrackValue>(
    expected: Option<SampledTrack<'_, '_, T>>,
    actual: Option<SampledTrack<'_, '_, T>>,
) {
    assert_eq!(expected.is_some(), actual.is_some());
    if let (Some(expected), Some(actual)) = (expected, actual) {
        assert_eq!(expected.track.target, actual.track.target);
        assert_eq!(expected.track.clock, actual.track.clock);
        assert_eq!(
            expected.track.curve.interpolation(),
            actual.track.curve.interpolation()
        );
        assert_eq!(expected.track.curve.times(), actual.track.curve.times());
        assert_eq!(expected.time, actual.time);
        assert_eq!(expected.period, actual.period);
        assert_eq!(expected.origin, actual.origin);
        assert_eq!(expected.interval, actual.interval);
    }
}

#[test]
fn compiled_selection_matches_flat_sampling_across_precedence_and_clock_boundaries() {
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let scalar = builder.add_value(-1f32);
    let absent = builder.add_value(42f32);
    let shared = builder.add_value(-2f32);
    let frozen = builder.add_value(-3f32);
    let rotation = builder.add_value(Quat::IDENTITY);
    let clock = builder
        .add_clock(SharedClock::Repeating { period: time(2.) })
        .unwrap();
    let frozen_clock = builder.add_clock(SharedClock::Frozen).unwrap();
    let scalar_track = |curve, clock| {
        DynamicTrack::Scalar(Track {
            target: scalar,
            curve,
            clock,
        })
    };
    let base = builder.add_animation(Animation {
        tracks: vec![
            scalar_track(
                Curve::linear(
                    [time(10.), time(11.), time(11.), time(12.)],
                    [0., 1., 3., 4.],
                )
                .unwrap(),
                TimeSource::Animation,
            ),
            DynamicTrack::Scalar(Track {
                target: shared,
                curve: Curve::linear([time(0.), time(2.)], [0., 8.]).unwrap(),
                clock: TimeSource::Shared(clock),
            }),
            DynamicTrack::Scalar(Track {
                target: frozen,
                curve: Curve::step([time(0.)], [7.]).unwrap(),
                clock: TimeSource::Shared(frozen_clock),
            }),
            DynamicTrack::Rotation(Track {
                target: rotation,
                curve: Curve::slerp(
                    [time(10.), time(12.)],
                    [Quat::IDENTITY, Quat::from_rotation_z(1.)],
                )
                .unwrap(),
                clock: TimeSource::Animation,
            }),
        ],
        ..Default::default()
    });
    let priority = builder.add_animation(Animation {
        tracks: vec![
            scalar_track(
                Curve::step([time(10.5), time(11.5)], [20., 30.]).unwrap(),
                TimeSource::Animation,
            ),
            scalar_track(
                Curve::step([time(50.)], [99.]).unwrap(),
                TimeSource::Animation,
            ),
            scalar_track(
                Curve::step(Vec::<Time>::new(), Vec::<f32>::new()).unwrap(),
                TimeSource::Animation,
            ),
        ],
        ..Default::default()
    });
    // Animations reused across global and clip selections retain writer precedence.
    builder.set_global_animations(vec![base, priority]);
    let clips = [
        Clip::named("loop")
            .with_source_start(time(10.))
            .with_duration(time(2.)),
        Clip::named("once")
            .with_source_start(time(10.))
            .with_duration(time(2.))
            .once(),
        Clip::named("zero duration")
            .with_source_start(time(11.))
            .with_duration(time(0.)),
        Clip::named("no keys")
            .with_source_start(time(20.))
            .with_duration(time(2.)),
    ]
    .map(|clip| {
        builder
            .add_clip(clip.with_animation(priority).with_animation(base))
            .unwrap()
    });
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let global_sample = program.sample(AnimationSample::default()).unwrap();
    let TrackSelection::Indexed(global_selection) = &global_sample.tracks else {
        unreachable!()
    };
    for clip in [None].into_iter().chain(clips.into_iter().map(Some)) {
        for seconds in [-3., 0., 0.5, 1., 1.5, 2., 3., 10., 11., 50.] {
            let selection = AnimationSample {
                clip,
                clip_time: time(seconds),
                shared_time: time(seconds),
            };
            let flat = model.animation_sample(selection).unwrap();
            let indexed = program.sample(selection).unwrap();
            let TrackSelection::Indexed(indexed_selection) = &indexed.tracks else {
                unreachable!()
            };
            assert!(eq(global_selection.global, indexed_selection.global));
            for property in [scalar, absent, shared, frozen] {
                assert_eq!(
                    flat.override_value(&property),
                    indexed.override_value(&property)
                );
                assert_eq!(flat.animated(&property), indexed.animated(&property));
                assert_same_track(flat.track(&property), indexed.track(&property));
            }
            assert_eq!(
                flat.override_value(&rotation),
                indexed.override_value(&rotation)
            );
            assert_same_track(flat.track(&rotation), indexed.track(&rotation));
            for occurrence in [
                None,
                Some((TimeSource::Animation, time(12.))),
                Some((TimeSource::Shared(clock), time(2.))),
            ] {
                let mut flat = flat.at_times(time(-seconds), time(seconds + 2.));
                let mut indexed = indexed.at_times(time(-seconds), time(seconds + 2.));
                if let Some((clock, time)) = occurrence {
                    flat = flat.at_occurrence(clock, time);
                    indexed = indexed.at_occurrence(clock, time);
                }
                for property in [scalar, absent, shared, frozen] {
                    assert_eq!(
                        flat.override_value(&property),
                        indexed.override_value(&property)
                    );
                    assert_same_track(flat.track(&property), indexed.track(&property));
                }
                assert_eq!(
                    flat.override_value(&rotation),
                    indexed.override_value(&rotation)
                );
                assert_same_track(flat.track(&rotation), indexed.track(&rotation));
            }
        }
    }
}
#[test]
fn retiming_borrows_indexed_programmes_and_preserves_alias_clock_policy() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("node").unwrap();
    let node_translation = builder.model().node(node).unwrap().transform.translation;
    let _node_scale = builder.model().node(node).unwrap().transform.scale;
    let property = node_translation;
    let base = builder
        .add_clip(Clip::named("base").with_duration(time(2.)))
        .unwrap();
    builder
        .add_track(base, dynamic_track(property, TimeSource::Animation, 4.))
        .unwrap();
    let alias = builder
        .add_clip(
            Clip::named("alias")
                .with_animation(builder.model().clip(base).unwrap().animations[0])
                .with_duration(time(1.)),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let program = AnimationProgram::compile(&model);
    let selected = program
        .sample(AnimationSample {
            clip: Some(alias),
            clip_time: time(0.1),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let original = selected.track(&property).unwrap();
    let retimed = selected.at_times(time(2.25), time(100.));
    assert!(eq(retimed.selection, &selected));
    assert_eq!(
        retimed.override_value(&property),
        Some(Vec3::from_array([0.5; 3]))
    );
    let effective = retimed.track(&property).unwrap();
    assert!(eq(original.track, effective.track));
    assert_eq!(effective.time, time(0.25));
    assert_eq!(effective.period, Some(time(1.)));
    assert_eq!(
        selected.override_value(&property),
        Some(Vec3::from_array([0.2; 3]))
    );
}
#[test]
fn retimed_timeline_and_shared_clocks_keep_offsets_intervals_and_fallback() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("node").unwrap();
    let node_translation = builder.model().node(node).unwrap().transform.translation;
    let node_scale = builder.model().node(node).unwrap().transform.scale;
    let translation = node_translation;
    let scale = node_scale;
    let period = builder
        .add_clock(SharedClock::Repeating { period: time(2.) })
        .unwrap();
    builder
        .add_global_track(DynamicTrack::Vector3(Track {
            target: translation,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                [time(9.), time(10.), time(12.), time(13.)],
                ([[90.; 3], [2.; 3], [6.; 3], [130.; 3]])
                    .into_iter()
                    .map(Vec3::from_array)
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        }))
        .unwrap();
    builder
        .add_global_track(dynamic_track(scale, TimeSource::Shared(period), 8.))
        .unwrap();
    let animation_3 = builder.add_animation(crate::model::Animation {
        tracks: vec![DynamicTrack::Vector3(Track {
            target: translation,
            clock: TimeSource::Animation,
            curve: Curve::step(Vec::<Time>::new(), Vec::<Vec3>::new()).unwrap(),
        })],
        ..Default::default()
    });
    let clip = builder
        .add_clip(
            Clip::named("interval")
                .with_duration(time(2.))
                .with_source_start(time(10.))
                .with_animation(animation_3),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let selected = model
        .animation_sample(AnimationSample {
            clip: Some(clip),
            clip_time: time(0.5),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let retimed = selected.at_times(time(3.5), time(5.));
    assert_eq!(
        retimed.override_value(&translation),
        Some(Vec3::from_array([5.; 3]))
    );
    assert_eq!(
        retimed.override_value(&scale),
        Some(Vec3::from_array([4.; 3]))
    );
    let track = retimed.track(&translation).unwrap();
    assert_eq!(track.origin, time(10.));
    assert_eq!(track.period, Some(time(2.)));
    assert_eq!(track.interval, Some((time(10.), time(12.))));
    assert_eq!(track.track.curve.values().len(), 4);
    let shared = retimed.track(&scale).unwrap();
    assert_eq!(shared.time, time(1.));
    assert_eq!(shared.period, Some(time(2.)));
    assert_eq!(shared.origin, Time::default());
}

fn scalar_model<'data>(
    curve: Curve<'data, f32>,
) -> (Model<'data>, Property<f32>, AnimationId, ClipId) {
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let property = builder.add_value(0.0);
    let animation = builder.add_animation(Animation {
        tracks: vec![DynamicTrack::Scalar(Track {
            target: property,
            curve,
            clock: TimeSource::Animation,
        })],
        ..Default::default()
    });
    let clip = builder
        .add_clip(
            Clip::named("keys")
                .with_animation(animation)
                .with_duration(time(2.))
                .once(),
        )
        .unwrap();
    (builder.finish().unwrap(), property, animation, clip)
}

fn compiled_scalar_curve(
    program: &AnimationProgram,
    animation: AnimationId,
) -> &Curve<'static, f32> {
    let DynamicTrack::Scalar(track) = &program.tracks[program.animations[&animation].start] else {
        panic!("expected the scalar fixture track")
    };
    &track.curve
}

fn scalar_buffers<'view>(curve: &'view Curve<'_, f32>) -> Vec<&'view [u8]> {
    let mut buffers = vec![curve.times().as_bytes(), curve.values().as_bytes()];
    if let Some(controls) = curve.hermite_derivatives() {
        buffers.extend([controls.incoming.as_bytes(), controls.outgoing.as_bytes()]);
    }
    if let Some(controls) = curve.bezier_controls() {
        buffers.extend([controls.incoming.as_bytes(), controls.outgoing.as_bytes()]);
    }
    if let Some(parameters) = curve.tcb_parameters() {
        buffers.push(parameters.as_bytes());
    }
    buffers
}

fn scalar_at(
    program: &AnimationProgram,
    property: Property<f32>,
    clip: ClipId,
    seconds: f64,
) -> f32 {
    program
        .sample(AnimationSample {
            clip: Some(clip),
            clip_time: time(seconds),
            ..Default::default()
        })
        .unwrap()
        .override_value(&property)
        .unwrap()
}

#[test]
fn separately_compiled_owned_programs_share_keys_and_all_spline_controls() {
    let keys = [time(0.), time(1.), time(2.)];
    let curves = [
        Curve::linear(keys, [0., 4., 8.]).unwrap(),
        Curve::hermite(keys, [0., 4., 8.], [4.; 3], [4.; 3]).unwrap(),
        Curve::bezier(keys, [0., 4., 8.], [0., 2., 6.], [2., 6., 8.]).unwrap(),
        Curve::tcb(keys, [0., 4., 8.], [TcbParameters::default(); 3]).unwrap(),
    ];
    for curve in curves {
        let authored_buffers = scalar_buffers(&curve)
            .into_iter()
            .map(|bytes| bytes.as_ptr())
            .collect::<Vec<_>>();
        let (model, property, animation, clip) = scalar_model(curve);
        let first = AnimationProgram::compile(&model);
        let second = AnimationProgram::compile(&model);
        assert!(!eq(
            &first.animations[&animation],
            &second.animations[&animation]
        ));
        for program in [&first, &second] {
            let buffers = scalar_buffers(compiled_scalar_curve(program, animation));
            assert_eq!(buffers.len(), authored_buffers.len());
            for (bytes, authored) in buffers.iter().zip(&authored_buffers) {
                assert!(!bytes.is_empty());
                assert_eq!(bytes.as_ptr(), *authored);
            }
        }
        let expected: Vec<_> = [0.25, 1., 1.75]
            .into_iter()
            .map(|seconds| {
                model
                    .animation_sample(AnimationSample {
                        clip: Some(clip),
                        clip_time: time(seconds),
                        ..Default::default()
                    })
                    .unwrap()
                    .override_value(&property)
                    .unwrap()
            })
            .collect();
        drop(model);
        for (seconds, expected) in [0.25, 1., 1.75].into_iter().zip(expected) {
            assert_eq!(scalar_at(&first, property, clip, seconds), expected);
            assert_eq!(scalar_at(&second, property, clip, seconds), expected);
        }
    }
}

#[test]
fn borrowed_program_keys_detach_before_source_bytes_are_changed_and_dropped() {
    let mut times = PackedArray::alloc_from_slice(&[time(0.), time(1.), time(2.)])
        .as_bytes()
        .to_vec();
    let mut values = PackedArray::alloc_from_slice(&[0f32, 4., 8.])
        .as_bytes()
        .to_vec();
    let mut incoming = PackedArray::alloc_from_slice(&[4f32; 3])
        .as_bytes()
        .to_vec();
    let mut outgoing = incoming.clone();
    let curve = Curve::hermite(
        PackedArray::parse_bytes(&times).unwrap(),
        PackedArray::parse_bytes(&values).unwrap(),
        PackedArray::parse_bytes(&incoming).unwrap(),
        PackedArray::parse_bytes(&outgoing).unwrap(),
    )
    .unwrap();
    let (model, property, animation, clip) = scalar_model(curve);
    let first = AnimationProgram::compile(&model);
    let second = AnimationProgram::compile(&model);
    for ((first, second), source) in scalar_buffers(compiled_scalar_curve(&first, animation))
        .into_iter()
        .zip(scalar_buffers(compiled_scalar_curve(&second, animation)))
        .zip([&times, &values, &incoming, &outgoing])
    {
        assert_eq!(first, source.as_slice());
        assert_eq!(second, source.as_slice());
        assert_ne!(first.as_ptr(), source.as_ptr());
        assert_ne!(second.as_ptr(), source.as_ptr());
        assert_ne!(first.as_ptr(), second.as_ptr());
    }
    drop(model);
    times.fill(0);
    values.fill(0);
    incoming.fill(0);
    outgoing.fill(0);
    drop((times, values, incoming, outgoing));
    assert_eq!(scalar_at(&first, property, clip, 0.25), 1.);
    assert_eq!(scalar_at(&second, property, clip, 1.75), 7.);
}

#[test]
fn program_clone_and_composition_share_keys_but_keep_property_tables_independent() {
    let (host_model, host_property, host_animation, host_clip) =
        scalar_model(Curve::linear([time(0.), time(2.)], [0., 8.]).unwrap());
    let (part_model, part_property, part_animation, part_clip) =
        scalar_model(Curve::hermite([time(0.), time(2.)], [2., 6.], [4.; 2], [4.; 2]).unwrap());
    assert_ne!(host_property, part_property);
    let host = AnimationProgram::compile(&host_model);
    let part = AnimationProgram::compile(&part_model);
    let mut cloned = host.clone();
    let combined = host
        .compose(&part, &[(host_clip, part_clip)], |_| true)
        .unwrap();
    for (original, reused, animation) in [
        (&host, &cloned, host_animation),
        (&host, &combined, host_animation),
        (&part, &combined, part_animation),
    ] {
        assert!(!eq(
            &original.animations[&animation],
            &reused.animations[&animation]
        ));
        for (original, reused) in scalar_buffers(compiled_scalar_curve(original, animation))
            .into_iter()
            .zip(scalar_buffers(compiled_scalar_curve(reused, animation)))
        {
            assert_eq!(original.as_ptr(), reused.as_ptr());
        }
    }
    cloned.values.set(host_property, -5.).unwrap();
    assert_eq!(host.values().get(host_property), Some(&0.));
    assert_eq!(combined.values().get(host_property), Some(&0.));
    assert_eq!(cloned.values().get(host_property), Some(&-5.));
    assert!(host.values().get(part_property).is_none());
    assert!(cloned.values().get(part_property).is_none());
    assert_eq!(combined.values().get(part_property), Some(&0.));
    assert_eq!(host.animations.len(), 1);
    assert_eq!(part.animations.len(), 1);
    assert_eq!(combined.animations.len(), 2);
    assert!(!combined.clips.contains_key(&part_clip));
    drop((host_model, part_model, host, part));
    assert_eq!(scalar_at(&cloned, host_property, host_clip, 1.), 4.);
    assert_eq!(scalar_at(&combined, host_property, host_clip, 1.), 4.);
    assert_eq!(scalar_at(&combined, part_property, host_clip, 1.), 4.);
}

#[test]
fn editing_a_shared_model_detaches_changed_keys_and_keeps_existing_programs_valid() {
    let (model, property, animation, clip) = scalar_model(
        Curve::hermite(
            [time(0.), time(1.), time(2.)],
            [0., 4., 8.],
            [4.; 3],
            [4.; 3],
        )
        .unwrap(),
    );
    let original = AnimationProgram::compile(&model);
    let mut edited = model.clone();
    edited
        .edit(|builder| {
            let mut definition = builder.model().animation(animation).unwrap().clone();
            let DynamicTrack::Scalar(track) = &mut definition.tracks[0] else {
                unreachable!()
            };
            let mut values = track.curve.values().clone();
            assert_eq!(values.set(1, 12.), Some(4.));
            let derivatives = track.curve.hermite_derivatives().unwrap();
            let mut incoming = derivatives.incoming.clone();
            incoming.for_each_mut(|value| *value = 0.);
            track.curve = Curve::hermite(
                track.curve.times().clone(),
                values,
                incoming,
                derivatives.outgoing.clone(),
            )?;
            builder.set_animation(animation, definition)?;
            builder.values_mut().set(property, 100.)?;
            Ok(())
        })
        .unwrap();
    let changed = AnimationProgram::compile(&edited);
    let original_buffers = scalar_buffers(compiled_scalar_curve(&original, animation));
    let changed_curve = compiled_scalar_curve(&changed, animation);
    let changed_buffers = scalar_buffers(changed_curve);
    assert_eq!(original_buffers[0].as_ptr(), changed_buffers[0].as_ptr());
    assert_ne!(original_buffers[1].as_ptr(), changed_buffers[1].as_ptr());
    assert_ne!(original_buffers[2].as_ptr(), changed_buffers[2].as_ptr());
    assert_eq!(original_buffers[3].as_ptr(), changed_buffers[3].as_ptr());
    assert_eq!(changed_curve.values().to_vec(), [0., 12., 8.]);
    assert_eq!(
        changed_curve
            .hermite_derivatives()
            .unwrap()
            .incoming
            .to_vec(),
        [0.; 3]
    );
    assert_eq!(model.values().get(property), Some(&0.));
    assert_eq!(edited.values().get(property), Some(&100.));
    drop((model, edited));
    assert_eq!(scalar_at(&original, property, clip, 1.), 4.);
    assert_eq!(scalar_at(&changed, property, clip, 1.), 12.);
}

#[test]
#[ignore = "explicit animation throughput measurement"]
fn compiled_property_sampling_throughput() {
    let mut slot_values = SlotValues::default();
    use std::hint::black_box;
    use std::time::Instant;
    for node_count in [128, 1024, 4096] {
        let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
        let mut properties = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            let node = builder.add_named_node("bone").unwrap();
            let node_translation = builder.model().node(node).unwrap().transform.translation;
            let _node_scale = builder.model().node(node).unwrap().transform.scale;
            let property = node_translation;
            builder
                .add_global_track(dynamic_track(property, TimeSource::Animation, 4.))
                .unwrap();
            properties.push(property);
        }
        let model = {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap();
        let program = AnimationProgram::compile(&model);
        let sampled = program
            .sample(AnimationSample {
                clip_time: time(1.),
                ..SlotConstruct::construct(&mut slot_values)
            })
            .unwrap();
        let start = Instant::now();
        let mut checksum = 0.;
        let rounds = 100;
        for _ in 0..rounds {
            for property in &properties {
                checksum += black_box(sampled.override_value(black_box(property)).unwrap()[0]);
            }
        }
        let count = rounds * node_count;
        eprintln!(
            "{node_count} animated nodes: {:.1} ns/property",
            start.elapsed().as_nanos() as f64 / count as f64
        );
        assert_eq!(checksum, (count * 2) as f32);
    }
}
