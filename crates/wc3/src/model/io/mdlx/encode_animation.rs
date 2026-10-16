//! Native clock restrictions applied only when exporting logical curves.
use super::animation::Value;
use crate::model::conversion::{Context, Error};
use crate::model::formats::mdlx::{
    animation::{TangentKeyframe, Track as NativeTrack, TrackValue, ValueKeyframe},
    Animatable,
};
use crate::model::profiles::{Mdlx, Universal};
use crate::model::{
    Animated, BindingValue, ClipId, Clock, ClockId, CurveDomain, Interpolation, Model,
    PropertyTarget,
};
use std::collections::{BTreeMap, HashSet};

type NativeKey<T> = (T, Option<T>, Option<T>);
pub(super) struct Encoder<'a, 'b, 'c> {
    pub model: &'b Model<'a, Mdlx>,
    pub starts: BTreeMap<ClipId, f64>,
    pub globals: BTreeMap<ClockId, u32>,
    pub context: &'c mut Context,
    pub used: HashSet<(PropertyTarget, &'static str)>,
}
impl Encoder<'_, '_, '_> {
    pub fn audit(&mut self) -> Result<(), Error> {
        let model = self.model;
        for channel in model
            .clips()
            .flat_map(|(id, _)| {
                model
                    .clip_program(id)
                    .expect("validated clip alias")
                    .channels()
                    .iter()
            })
            .chain(model.shared_channels().iter())
            .chain(model.timeline_channels().iter())
        {
            if !self
                .used
                .iter()
                .any(|(target, _)| target == channel.target())
            {
                self.context.unsupported(
                    "animation.target",
                    format!(
                        "Warcraft III has no output binding for {:?}",
                        channel.target()
                    ),
                )?;
            }
        }
        Ok(())
    }
    pub fn value<T, U>(
        &mut self,
        source: &Animated<U>,
        map: impl Fn(U) -> Result<T, Error> + Clone,
        path: &str,
    ) -> Result<Animatable<'static, T>, Error>
    where
        T: TrackValue,
        U: Value,
    {
        let value = map(source.default)?;
        let Some(property) = &source.binding else {
            return Ok(Animatable::Static(value));
        };
        Ok(match self.track(property, map, path)? {
            Some(track) => Animatable::Both { value, track },
            None => Animatable::Static(value),
        })
    }
    pub fn track<T, B>(
        &mut self,
        binding: &B,
        map: impl Fn(B::Value) -> Result<T, Error> + Clone,
        path: &str,
    ) -> Result<Option<NativeTrack<'static, T>>, Error>
    where
        T: TrackValue,
        B: BindingValue<Profile = Universal>,
        B::Value: Value,
    {
        let Some(property) = binding.property() else {
            self.context.unsupported(
                path,
                "Warcraft III scalar tracks cannot directly encode a converted boolean binding",
            )?;
            return Ok(None);
        };
        self.target(property.target(), map, path)
    }
    pub fn target<T, U>(
        &mut self,
        property: &PropertyTarget,
        map: impl Fn(U) -> Result<T, Error> + Clone,
        path: &str,
    ) -> Result<Option<NativeTrack<'static, T>>, Error>
    where
        T: TrackValue,
        U: Value,
    {
        let tangent_map = map.clone();
        self.target_maps(property, map, move |value, _| tangent_map(value), path)
    }
    pub fn target_maps<T, U>(
        &mut self,
        property: &PropertyTarget,
        map: impl Fn(U) -> Result<T, Error> + Clone,
        tangent_map: impl Fn(U, Interpolation) -> Result<T, Error> + Clone,
        path: &str,
    ) -> Result<Option<NativeTrack<'static, T>>, Error>
    where
        T: TrackValue,
        U: Value,
    {
        self.used.insert((property.clone(), U::kind()));
        let mut keys = BTreeMap::<i32, Vec<NativeKey<T>>>::new();
        let mut behavior = None;
        let channels = self
            .model
            .clips()
            .flat_map(|(id, _)| {
                self.model
                    .clip_program(id)
                    .expect("validated clip alias")
                    .channels()
                    .iter()
                    .map(move |channel| (Some(id), channel))
            })
            .chain(
                self.model
                    .shared_channels()
                    .iter()
                    .map(|channel| (None, channel)),
            )
            .chain(
                self.model
                    .timeline_channels()
                    .iter()
                    .map(|channel| (None, channel)),
            );
        for (clip_id, channel) in channels {
            let Some(track) = U::track(channel) else {
                continue;
            };
            if track.target.target() != property {
                continue;
            }
            let mode = track.curve.interpolation();
            if U::Domain::SUPPORTS_ROTATION
                && matches!(mode, Interpolation::Hermite | Interpolation::Bezier)
                && !track.curve.uses_spherical_controls()
            {
                self.context.unsupported(path, "Warcraft III rotation splines require spherical controls; component splines cannot be encoded faithfully")?;
                continue;
            }
            let global = match track.clock {
                Clock::Clip | Clock::Timeline => None,
                Clock::Shared(id) => Some(
                    *self
                        .globals
                        .get(&id)
                        .ok_or_else(|| Error::new(path, "shared clock does not exist"))?,
                ),
            };
            if behavior.is_some_and(|old| old != (mode, global)) {
                self.context.unsupported(path, "Warcraft III cannot combine different interpolation or evaluation clocks in one property")?;
                continue;
            }
            behavior = Some((mode, global));
            let offset = if global.is_some() || matches!(track.clock, Clock::Timeline) {
                0.
            } else {
                self.starts[&clip_id.expect("clip channel has a clip")]
            };
            let mut track_keys = BTreeMap::<i32, Vec<NativeKey<T>>>::new();
            for index in 0..track.curve.times().len() {
                let time = track
                    .curve
                    .times()
                    .get(index)
                    .expect("checked key index")
                    .as_seconds();
                let frame = frame_at(time, offset, self.context, path)?;
                let value = map(track
                    .curve
                    .values()
                    .get(index)
                    .expect("checked value index"))?;
                let incoming = track
                    .curve
                    .incoming_value(index)
                    .map(|value| tangent_map(value, mode))
                    .transpose()?;
                let outgoing = track
                    .curve
                    .outgoing_value(index)
                    .map(|value| tangent_map(value, mode))
                    .transpose()?;
                let key = (value, incoming, outgoing);
                track_keys.entry(frame).or_default().push(key);
            }
            for (frame, frame_keys) in track_keys {
                if keys.get(&frame).is_some_and(|old| *old != frame_keys) {
                    self.context.unsupported(path, "clip curves assign conflicting coincident values or tangents at one native frame")?;
                }
                keys.insert(frame, frame_keys);
            }
        }
        let Some((interpolation, global)) = behavior else {
            return Ok(None);
        };
        let plain = || {
            keys.iter()
                .flat_map(|(frame, entries)| {
                    entries.iter().map(move |(value, _, _)| ValueKeyframe {
                        frame: *frame,
                        value: *value,
                    })
                })
                .collect::<Vec<_>>()
        };
        let spline = || {
            keys.iter()
                .flat_map(|(frame, entries)| {
                    entries
                        .iter()
                        .map(move |(value, incoming, outgoing)| TangentKeyframe {
                            frame: *frame,
                            value: *value,
                            in_tangent: incoming.unwrap_or(*value),
                            out_tangent: outgoing.unwrap_or(*value),
                        })
                })
                .collect::<Vec<_>>()
        };
        let track = match interpolation {
            Interpolation::Step => NativeTrack::step(plain(), global),
            Interpolation::Linear => NativeTrack::linear(plain(), global),
            Interpolation::Hermite => NativeTrack::hermite(spline(), global),
            Interpolation::Bezier => NativeTrack::bezier(spline(), global),
            Interpolation::Tcb | Interpolation::Slerp | Interpolation::Nlerp => {
                return Err(Error::new(
                    path,
                    "Warcraft III tracks cannot encode explicit TCB, SLERP or NLERP directives",
                ));
            }
        }
        .map_err(|error| Error::new(path, error.to_string()))?;
        Ok(Some(track))
    }
}
fn milliseconds(seconds: f64, context: &mut Context, path: &str) -> Result<f64, Error> {
    let value = seconds * 1000.;
    let rounded = value.round();
    if !value.is_finite() {
        return Err(Error::new(
            path,
            "time exceeds the finite native millisecond range",
        ));
    }
    let difference = (value - rounded).abs();
    if difference > f64::EPSILON * value.abs().max(1.) * 8. {
        context.approximate(
            path,
            (difference / 1000.) as f32,
            "time rounded to Warcraft III milliseconds",
        )?;
    }
    Ok(rounded)
}
pub(super) fn frame_at(
    seconds: f64,
    offset: f64,
    context: &mut Context,
    path: &str,
) -> Result<i32, Error> {
    let value = milliseconds(seconds, context, path)? + milliseconds(offset, context, path)?;
    if value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(Error::new(
            path,
            "key time exceeds the signed native millisecond range",
        ));
    }
    Ok(value as i32)
}
pub(super) fn unsigned_frame(
    seconds: f64,
    context: &mut Context,
    path: &str,
) -> Result<u32, Error> {
    let value = seconds * 1000.;
    let rounded = value.round();
    if !value.is_finite() || rounded < 0. || rounded > f64::from(u32::MAX) {
        return Err(Error::new(
            path,
            "duration or interval exceeds the unsigned native millisecond range",
        ));
    }
    let difference = (value - rounded).abs();
    if difference > f64::EPSILON * value.abs().max(1.) * 8. {
        context.approximate(
            path,
            (difference / 1000.) as f32,
            "time rounded to Warcraft III milliseconds",
        )?;
    }
    Ok(rounded as u32)
}
