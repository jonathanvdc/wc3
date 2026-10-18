//! Authoring limits inherent to Warcraft III rather than a particular native dialect.
use super::Mdlx;
use crate::model::{Billboard, DynamicTrack, Error, Interpolation, Model, TimeSource, Transform};
pub(super) fn model(value: &Model<'_, Mdlx>) -> Result<(), Error> {
    value.validate_animation_consumers()?;
    if value.bounds().is_none()
        && (!value.name().is_empty()
            || !value.metadata().animation_file.is_empty()
            || value.metadata().blend_time != 0.)
    {
        return Err(Error::new(
            "model.info",
            "Warcraft III model name, animation filename, and blend time require a model-info declaration with bounds",
        ));
    }
    let mut ordinary_selection = None;
    for (_, clip) in value.clips() {
        if clip.animations.iter().any(|id| {
            let animation = value.animation(*id).expect("validated selection");
            animation
                .tracks
                .iter()
                .any(|channel| matches!(channel.clock(), TimeSource::Shared(_)))
                || animation.events.values().any(|event| {
                    matches!(
                        value
                            .event_source(event.source)
                            .expect("validated source")
                            .clock,
                        TimeSource::Shared(_)
                    )
                })
        }) {
            return Err(Error::new(
                "clips.animations",
                "Warcraft III shared-clock animations must be selected at model level",
            ));
        }

        let selection: Vec<_> = clip
            .animations
            .iter()
            .copied()
            .filter(|id| {
                let animation = value.animation(*id).expect("validated selection");
                animation
                    .tracks
                    .iter()
                    .any(|channel| channel.clock() == TimeSource::Animation)
                    || animation.events.values().any(|event| {
                        value
                            .event_source(event.source)
                            .expect("validated source")
                            .clock
                            == TimeSource::Animation
                    })
            })
            .collect();
        if ordinary_selection
            .as_ref()
            .is_some_and(|first| *first != selection)
        {
            return Err(Error::new("clips.animations", "Warcraft III clips must select the same ordered ordinary animations on their shared authoring timeline"));
        }
        ordinary_selection = Some(selection);
    }
    for channel in value.tracks() {
        validate_channel(channel)?;
    }
    for (_, instance) in value.instances() {
        let mesh = value.mesh(instance.mesh).expect("validated geometry");
        if !mesh.influences().is_empty() {
            let anchor = value.node(instance.node).expect("validated placement");
            let animated = value.is_animated(anchor.transform.translation)
                || value.is_animated(anchor.transform.rotation)
                || value.is_animated(anchor.transform.scale);
            if anchor.parent.is_some()
                || anchor.transform.defaults(value.values()) != Transform::default()
                || anchor.billboard != Billboard::None
                || anchor.camera_anchored
                || animated
            {
                return Err(Error::new(
                    "instance.transform",
                    "Warcraft III skinned geometry uses model-space joint transforms without an additional placed mesh transform",
                ));
            }
        }
    }
    for (_, node) in value.nodes() {
        if value.values().get(node.visible) != Some(&true) || value.is_animated(node.visible) {
            return Err(Error::new(
                "nodes.visible",
                "Warcraft III hierarchy nodes have no static hidden state",
            ));
        }
    }
    Ok(())
}
fn validate_channel(channel: &DynamicTrack<'_>) -> Result<(), Error> {
    let mode = match channel {
        DynamicTrack::Scalar(track) => track.curve.interpolation(),
        DynamicTrack::Vector3(track) => track.curve.interpolation(),
        DynamicTrack::Rotation(track) => track.curve.interpolation(),
        DynamicTrack::Unsigned(track) => track.curve.interpolation(),
        DynamicTrack::Texture(track) => track.curve.interpolation(),
        DynamicTrack::OptionalTexture(track) => track.curve.interpolation(),
        _ => {
            return Err(Error::new(
                "animation.value",
                "Warcraft III tracks carry scalar floats, three-component vectors, quaternions, integer selectors, or texture selections",
            ));
        }
    };
    if matches!(
        mode,
        Interpolation::Tcb | Interpolation::Slerp | Interpolation::Nlerp
    ) {
        return Err(Error::new(
            "animation.interpolation",
            "Warcraft III tracks do not encode explicit TCB, SLERP or NLERP interpolation directives",
        ));
    }
    Ok(())
}

pub(super) fn resources(value: &Model<'_, Mdlx>) -> Result<(), Error> {
    use crate::model::{AddressMode, ColorSpace, MdlxImageResource};
    for texture in value.sampled_images() {
        let controls = texture.mdlx_controls()?;
        if let Some(path) = controls.fallback_path {
            if !matches!(
                value.image_resource(texture.resource),
                Some(MdlxImageResource::Replaceable(_))
            ) || !matches!(value.image_resource(path), Some(MdlxImageResource::Path(_)))
            {
                return Err(Error::new(
                    "texture.fallback_path",
                    "fallback filenames require a path dependency on a replaceable image input",
                ));
            }
        }
        if texture.color_space != ColorSpace::Unspecified {
            return Err(Error::new(
                "textures.color_space",
                "Warcraft III texture definitions do not declare a color transfer function",
            ));
        }
        if !matches!(texture.sampler.u, AddressMode::Repeat | AddressMode::Clamp)
            || !matches!(texture.sampler.v, AddressMode::Repeat | AddressMode::Clamp)
        {
            return Err(Error::new(
                "textures.sampler",
                "Warcraft III texture definitions support repeat or clamp addressing",
            ));
        }
    }
    Ok(())
}
