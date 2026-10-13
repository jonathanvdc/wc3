//! Exact integer controls interpreted by the target's numeric property fields.
use super::*;
fn float_track<T: Copy + Into<f64>>(track: &sm::Track<T>) -> Result<sm::Track, Error> {
    let components = |values: &[T]| -> Result<Vec<f32>, Error> {
        values
            .iter()
            .map(|&value| {
                let value = value.into();
                let encoded = value as f32;
                if f64::from(encoded) != value {
                    return Err(Error::new(
                        "animation.integer",
                        "integer cannot be represented exactly by this M2 numeric control",
                    ));
                }
                Ok(encoded)
            })
            .collect()
    };
    Ok(sm::Track {
        layer: track.layer.clone(),
        target: track.target,
        interpolation: track.interpolation,
        global_duration: track.global_duration,
        keys: track
            .keys
            .iter()
            .map(|key| {
                Ok(sm::Key {
                    time: key.time,
                    value: components(&key.value)?,
                    incoming: key.incoming.as_deref().map(components).transpose()?,
                    outgoing: key.outgoing.as_deref().map(components).transpose()?,
                })
            })
            .collect::<Result<_, Error>>()?,
    })
}
pub(in super::super) fn numeric_tracks(
    model: &mut sm::Model<'_>,
    context: &mut Context,
) -> Result<(), Error> {
    for (index, clip) in model.clips.iter_mut().enumerate() {
        for (channel, track) in clip.integer_tracks.iter().enumerate() {
            let path = format!("clips[{index}].integer_tracks[{channel}]");
            if matches!(track.target(), sm::Target::NativeProperty { .. }) {
                context.unsupported(
                    path,
                    "native integer control belongs to a different model profile",
                )?;
                continue;
            }
            let converted = match track {
                sm::IntegerTrack::Unsigned(track) | sm::IntegerTrack::Unsigned4(track) => {
                    float_track(track)
                }
                sm::IntegerTrack::Signed(track) => float_track(track),
                sm::IntegerTrack::Unsigned16(track) => float_track(track),
                sm::IntegerTrack::Bytes4(track) | sm::IntegerTrack::Boolean(track) => {
                    float_track(track)
                }
            };
            match converted {
                Ok(track) => clip.tracks.push(track),
                Err(error) => {
                    context.unsupported(path, error.description)?;
                }
            }
        }
        clip.integer_tracks.clear();
    }
    Ok(())
}
