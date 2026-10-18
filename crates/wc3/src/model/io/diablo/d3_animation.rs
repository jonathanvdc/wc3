//! Animation resources bind by authored bone names, with permutation-local seconds.
use super::{i32_count, name, native, text, text_owned};
use crate::model::conversion::{Context, Error, ExportOptions};
use crate::model::formats::diablo::d3::{Animation, V118};
use crate::model::io::materialize;
use crate::model::profiles::diablo::d3::ClipControls;
use crate::model::profiles::diablo::d3::{NodeRole, D3};
use crate::model::Model;
use crate::model::Quaternion;
use crate::model::{Clip, Curve, DynamicTrack, Interpolation, Time, TimeSource, Track};
use std::collections::BTreeMap;
use zerovec::ZeroVec;
pub(super) fn read<'a>(
    asset: &mut Model<'a, D3>,
    sources: &[&'a Animation<'_, V118>],
    context: &mut Context,
) -> Result<(), Error> {
    let model = &asset;
    let mut bones = BTreeMap::new();
    for (id, node) in model.nodes() {
        if matches!(node.controls, NodeRole::Bone(_))
            && bones.insert(node.name.to_string(), id).is_some()
        {
            return Err(Error::new(
                "d3.animation.bone_names",
                "skeleton bone names must be unique for animation binding",
            ));
        }
    }
    let mut programs = Vec::new();
    let mut resource_ids = Vec::new();
    for source in sources {
        let root = &source.root;
        if root.appearance >= 0 && root.appearance != model.metadata().sno_id {
            return Err(Error::new(
                "d3.animation.appearance",
                "animation names a different appearance resource",
            ));
        }
        if resource_ids.contains(&root.sno_id) {
            return Err(Error::new(
                "d3.animation.identity",
                "duplicate animation SNO identity",
            ));
        }
        resource_ids.push(root.sno_id);
        if root.permutations.is_empty() {
            context.unsupported(
                "d3.animation.permutations",
                "empty native animation resources have no playback clip representation",
            )?;
        }
        for p in &root.permutations {
            let fps = f64::from(p.frames_per_tick) * 60.;
            if !fps.is_finite() || fps <= 0. || p.frame_count < 0 {
                return Err(Error::new(
                    "d3.animation.timing",
                    "animation requires positive finite frame rate and nonnegative frame count",
                ));
            }
            if !p.attachments.is_empty() {
                context.unsupported("d3.animation.attachments","native trigger-event payloads have no established logical event representation")?;
            }
            if [
                p.translation_curves.len(),
                p.rotation_curves.len(),
                p.scale_curves.len(),
            ]
            .iter()
            .any(|n| *n != p.bone_names.len())
            {
                return Err(Error::new(
                    "d3.animation.curves",
                    "curve arrays must match the declared bone palette",
                ));
            }
            let mut nodes = Vec::new();
            for bone in p.bone_names.iter() {
                let name = text(&bone.bone_name)?;
                let id = *bones.get(name).ok_or_else(|| {
                    Error::new(
                        "d3.animation.bone_names",
                        format!("animation bone {name} is absent from appearance"),
                    )
                })?;
                if nodes.contains(&id) {
                    return Err(Error::new(
                        "d3.animation.bone_names",
                        "animation palette repeats a bone",
                    ));
                }
                nodes.push(id);
            }
            let mut clip = Clip::<D3>::named(text(&p.name)?).with_duration(Time::seconds(
                f64::from(p.frame_count.saturating_sub(1)) / fps,
            )?);
            clip.controls = ClipControls {
                sno_id: root.sno_id,
                appearance_sno_id: root.appearance,
                bone_order: nodes.clone(),
                root_motion: p.root_motion.iter().collect(),
                root_motion_in_place: p.root_motion_in_place.iter().collect(),
                hardpoint_name: text_owned(&p.hardpoint.name)?,
                hardpoint_index: p.hardpoint.index,
                animation_flags: [root.flags, root.flags2],
                name_hash: root.name_hash,
                flags: p.flags,
                frames_per_second: fps,
                frame_count: p.frame_count as u32,
                frames_per_key: [
                    p.frames_per_translation_key,
                    p.frames_per_rotation_key,
                    p.frames_per_scale_key,
                ],
                blend_ticks: [p.blend_ticks_from_other_anim, p.blend_ticks_same_perm_swap],
                selection_weight: p.selection_weight,
                speed_scalar: p.speed_scalar,
                marker_frames: [
                    p.marker_frame0,
                    p.marker_frame1,
                    p.marker_frame2,
                    p.marker_frame3,
                ],
                marker_points: [p.point0, p.point1, p.point2, p.point3],
                event_frame: p.event_frame,
                movement_velocity: p.movement_velocity,
                unclassified_controls: [p.unknown4_c, p.unknown6_c, p.unknown70, p.unknown74],
            };
            let mut channels = Vec::new();
            for (i, &node) in nodes.iter().enumerate() {
                let keys = &p.translation_curves[i].keys;
                if !keys.is_empty() {
                    let times = materialize(keys.len(), move |k| {
                        Time::seconds(f64::from(keys.get(k).unwrap().frame) / fps).unwrap()
                    });
                    let values = materialize(keys.len(), move |k| keys.get(k).unwrap().position);
                    channels.push(DynamicTrack::Vector3(Track {
                        target: model.node(node).unwrap().transform.translation,
                        curve: Curve::linear(times, values)?,
                        clock: TimeSource::Animation,
                    }));
                }
                let keys = &p.rotation_curves[i].keys;
                if !keys.is_empty() {
                    let times = materialize(keys.len(), move |k| {
                        Time::seconds(f64::from(keys.get(k).unwrap().frame) / fps).unwrap()
                    });
                    let values = materialize(keys.len(), move |k| {
                        Quaternion(keys.get(k).unwrap().rotation.components())
                    });
                    channels.push(DynamicTrack::Rotation(Track {
                        target: model.node(node).unwrap().transform.rotation,
                        curve: Curve::nlerp(times, values)?,
                        clock: TimeSource::Animation,
                    }));
                }
                let keys = &p.scale_curves[i].keys;
                if !keys.is_empty() {
                    let times = materialize(keys.len(), move |k| {
                        Time::seconds(f64::from(keys.get(k).unwrap().frame) / fps).unwrap()
                    });
                    let values = materialize(keys.len(), move |k| [keys.get(k).unwrap().scale; 3]);
                    channels.push(DynamicTrack::Vector3(Track {
                        target: model.node(node).unwrap().transform.scale,
                        curve: Curve::linear(times, values)?,
                        clock: TimeSource::Animation,
                    }));
                }
            }
            programs.push((clip, channels));
        }
    }
    asset.edit(move |builder| {
        for (clip, channels) in programs {
            let id = builder.add_clip(clip)?;
            for channel in channels {
                builder.add_track(id, channel)?;
            }
        }
        Ok(())
    })?;
    Ok(())
}
fn frame(time: Time, fps: f64, context: &mut Context) -> Result<i32, Error> {
    let value = (time.as_seconds() * fps).round();
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(Error::new(
            "d3.animation.time",
            "key time exceeds native signed frame range",
        ));
    }
    let frame = value as i32;
    context.approximate(
        "d3.animation.time",
        (time.as_seconds() - f64::from(frame) / fps).abs() as f32,
        "key time requires native frame quantization",
    )?;
    Ok(frame)
}
fn rotation(value: Quaternion, context: &mut Context) -> Result<native::Quaternion16, Error> {
    let mut out = [0u16; 4];
    for (i, v) in value.0.into_iter().enumerate() {
        let scaled = (v * 32767.).round();
        if !scaled.is_finite() || scaled < f32::from(i16::MIN) || scaled > f32::from(i16::MAX) {
            return Err(Error::new(
                "d3.animation.rotation",
                "quaternion channel exceeds signed normalized native storage",
            ));
        }
        let packed = scaled as i16;
        context.approximate(
            "d3.animation.rotation",
            (v - f32::from(packed) / 32767.).abs(),
            "quaternion channel requires native quantization",
        )?;
        out[i] = packed as u16;
    }
    Ok(native::Quaternion16 {
        x: out[0],
        y: out[1],
        z: out[2],
        w: out[3],
    })
}
pub(super) fn write(
    asset: &Model<'_, D3>,
    _options: &ExportOptions,
    context: &mut Context,
) -> Result<Vec<Animation<'static, V118>>, Error> {
    asset.validate()?;
    let model = asset;
    let mut groups: Vec<native::AnimationRecord<'static>> = Vec::new();
    for (id, clip) in model.clips() {
        if !clip.is_looping() {
            return Err(Error::new(
                "d3.animation.playback",
                "native permutations require independent looping programs",
            ));
        }
        if model.clip_events(id).next().is_some() {
            context.unsupported(
                "d3.animation.events",
                "logical events cannot be exported as unestablished native trigger payloads",
            )?;
        }
        let c = &clip.controls;
        let expected = f64::from(c.frame_count.saturating_sub(1)) / c.frames_per_second;
        if clip.duration().as_seconds() != expected {
            return Err(Error::new(
                "d3.animation.duration",
                "duration disagrees with authored frame count and rate",
            ));
        }
        let tick = (c.frames_per_second / 60.) as f32;
        if f64::from(tick) * 60. != c.frames_per_second {
            return Err(Error::new(
                "d3.animation.rate",
                "frame rate is not exactly representable by native frames per tick",
            ));
        }
        let names = c
            .bone_order
            .iter()
            .map(|id| {
                let node = model.node(*id).ok_or_else(|| {
                    Error::new("d3.animation.bone", "animation palette target is absent")
                })?;
                if !matches!(node.controls, NodeRole::Bone(_)) {
                    return Err(Error::new(
                        "d3.animation.bone",
                        "animation palette target is not a skeleton bone",
                    ));
                }
                Ok(native::BoneName {
                    bone_name: name(&node.name)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let mut p = native::AnimationPermutation {
            name: name(clip.name())?,
            flags: c.flags,
            frames_per_tick: tick,
            unknown4_c: c.unclassified_controls[0],
            frames_per_translation_key: c.frames_per_key[0],
            frames_per_rotation_key: c.frames_per_key[1],
            frames_per_scale_key: c.frames_per_key[2],
            blend_ticks_from_other_anim: c.blend_ticks[0],
            blend_ticks_same_perm_swap: c.blend_ticks[1],
            selection_weight: c.selection_weight,
            speed_scalar: c.speed_scalar,
            unknown6_c: c.unclassified_controls[1],
            unknown70: c.unclassified_controls[2],
            unknown74: c.unclassified_controls[3],
            bone_count: i32_count(names.len())?,
            bone_names: ZeroVec::alloc_from_slice(&names),
            frame_count: i32::try_from(c.frame_count).map_err(|_| {
                Error::new(
                    "d3.animation.frames",
                    "frame count exceeds native signed field",
                )
            })?,
            translation_curves: vec![native::TranslationCurve::default(); names.len()],
            rotation_curves: vec![native::RotationCurve::default(); names.len()],
            scale_curves: vec![native::ScaleCurve::default(); names.len()],
            marker_frame0: c.marker_frames[0],
            marker_frame1: c.marker_frames[1],
            marker_frame2: c.marker_frames[2],
            marker_frame3: c.marker_frames[3],
            point0: c.marker_points[0],
            point1: c.marker_points[1],
            point2: c.marker_points[2],
            point3: c.marker_points[3],
            event_frame: c.event_frame,
            root_motion: ZeroVec::alloc_from_slice(&c.root_motion),
            root_motion_in_place: ZeroVec::alloc_from_slice(&c.root_motion_in_place),
            movement_velocity: c.movement_velocity,
            hardpoint: native::HardpointLink {
                name: name(&c.hardpoint_name)?,
                index: c.hardpoint_index,
            },
            ..Default::default()
        };
        let mut seen = Vec::new();
        for channel in model.clip_tracks(id) {
            if channel.clock() != TimeSource::Animation {
                return Err(Error::new(
                    "d3.animation.clock",
                    "native permutation channels require ordinary animation time",
                ));
            }
            let destinations: Vec<_> = model
                .nodes()
                .iter()
                .filter_map(|(id, node)| {
                    if channel.target() == node.transform.translation.slot() {
                        Some((id, 0))
                    } else if channel.target() == node.transform.rotation.slot() {
                        Some((id, 1))
                    } else if channel.target() == node.transform.scale.slot() {
                        Some((id, 2))
                    } else {
                        None
                    }
                })
                .collect();
            if destinations.is_empty() {
                context.unsupported(
                    "d3.animation.target",
                    "native bone curves support translation, rotation and uniform scale",
                )?;
                continue;
            };
            for (node, kind) in destinations {
                let index = c
                    .bone_order
                    .iter()
                    .position(|id| *id == node)
                    .ok_or_else(|| {
                        Error::new(
                            "d3.animation.palette",
                            "channel target is absent from authored animation palette",
                        )
                    })?;
                if seen.contains(&(index, kind)) {
                    return Err(Error::new(
                        "d3.animation.channel",
                        "duplicate channel targets cannot share one native curve",
                    ));
                }
                seen.push((index, kind));
                match (kind, channel) {
                    (0, DynamicTrack::Vector3(track)) => {
                        if track.curve.interpolation() != Interpolation::Linear {
                            return Err(Error::new(
                                "d3.animation.interpolation",
                                "translation requires linear interpolation",
                            ));
                        }
                        let keys = track
                            .curve
                            .times()
                            .iter()
                            .zip(track.curve.values().iter())
                            .map(|(time, position)| {
                                Ok(native::TranslationKey {
                                    frame: frame(
                                        Time::seconds(
                                            time.as_seconds() - clip.source_start.as_seconds(),
                                        )?,
                                        c.frames_per_second,
                                        context,
                                    )?,
                                    position,
                                })
                            })
                            .collect::<Result<Vec<_>, Error>>()?;
                        p.translation_curves[index] = native::TranslationCurve {
                            key_count: i32_count(keys.len())?,
                            keys: ZeroVec::alloc_from_slice(&keys),
                            ..Default::default()
                        };
                    }
                    (1, DynamicTrack::Rotation(track)) => {
                        if track.curve.interpolation() != Interpolation::Nlerp {
                            return Err(Error::new(
                                "d3.animation.interpolation",
                                "rotation requires normalized-linear quaternion interpolation",
                            ));
                        }
                        let keys = track
                            .curve
                            .times()
                            .iter()
                            .zip(track.curve.values().iter())
                            .filter(|(time, _)| {
                                *time >= clip.source_start
                                    && time.as_seconds()
                                        <= clip.source_start.as_seconds()
                                            + clip.duration().as_seconds()
                            })
                            .map(|(time, value)| {
                                Ok(native::RotationKey {
                                    frame: frame(
                                        Time::seconds(
                                            time.as_seconds() - clip.source_start.as_seconds(),
                                        )?,
                                        c.frames_per_second,
                                        context,
                                    )?,
                                    rotation: rotation(value, context)?,
                                })
                            })
                            .collect::<Result<Vec<_>, Error>>()?;
                        p.rotation_curves[index] = native::RotationCurve {
                            key_count: i32_count(keys.len())?,
                            keys: ZeroVec::alloc_from_slice(&keys),
                            ..Default::default()
                        };
                    }
                    (2, DynamicTrack::Vector3(track)) => {
                        if track.curve.interpolation() != Interpolation::Linear {
                            return Err(Error::new(
                                "d3.animation.interpolation",
                                "scale requires linear interpolation",
                            ));
                        }
                        let keys = track
                            .curve
                            .times()
                            .iter()
                            .zip(track.curve.values().iter())
                            .filter(|(time, _)| {
                                *time >= clip.source_start
                                    && time.as_seconds()
                                        <= clip.source_start.as_seconds()
                                            + clip.duration().as_seconds()
                            })
                            .map(|(time, value)| {
                                if value[0] != value[1] || value[0] != value[2] {
                                    return Err(Error::new(
                                        "d3.animation.scale",
                                        "native scale curves require uniform scale",
                                    ));
                                }
                                Ok(native::ScaleKey {
                                    frame: frame(
                                        Time::seconds(
                                            time.as_seconds() - clip.source_start.as_seconds(),
                                        )?,
                                        c.frames_per_second,
                                        context,
                                    )?,
                                    scale: value[0],
                                })
                            })
                            .collect::<Result<Vec<_>, Error>>()?;
                        p.scale_curves[index] = native::ScaleCurve {
                            key_count: i32_count(keys.len())?,
                            keys: ZeroVec::alloc_from_slice(&keys),
                            ..Default::default()
                        };
                    }
                    _ => {
                        return Err(Error::new(
                            "d3.animation.channel",
                            "channel value type does not match its target",
                        ))
                    }
                }
            }
        }
        if let Some(group) = groups.iter_mut().find(|g| g.sno_id == c.sno_id) {
            if [group.flags, group.flags2] != c.animation_flags
                || group.name_hash != c.name_hash
                || group.appearance != c.appearance_sno_id
            {
                return Err(Error::new(
                    "d3.animation.resource",
                    "permutations sharing one SNO disagree on animation-wide declarations",
                ));
            }
            group.permutations.push(p);
        } else {
            groups.push(native::AnimationRecord {
                sno_id: c.sno_id,
                flags: c.animation_flags[0],
                flags2: c.animation_flags[1],
                appearance: c.appearance_sno_id,
                name_hash: c.name_hash,
                permutations: vec![p],
                ..Default::default()
            });
        }
    }
    groups
        .into_iter()
        .map(|mut root| {
            root.permutation_count = i32_count(root.permutations.len())?;
            Ok(Animation::<V118>::new(root))
        })
        .collect()
}
