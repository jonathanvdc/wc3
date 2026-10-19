use super::*;
use crate::model::ModelMetadata;
use crate::model::{Clip, Curve, ModelBuilder, SharedClock};
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
    assert_eq!(selection.iter().count(), 1);
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
