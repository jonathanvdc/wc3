//! Authoring limits inherent to Warcraft III rather than a particular native dialect.
use super::{Mdlx, Node};
use crate::model::error::{Location, ProfileError, ProfileKind, ProfileViolation};
use crate::model::AnimationId;
use crate::model::{Billboard, DynamicTrack, Error, Interpolation, Model, TimeSource, Transform};
pub(super) fn model(value: &Model<'_, Mdlx>) -> Result<(), Error> {
    value.validate_animation_consumers()?;
    for (id, feature) in value.features() {
        if let Node::Camera(camera) = &feature.payload {
            if !camera.is_mdlx() {
                return Err(Error::at(
                    Location::for_owner(id).field("projection"),
                    ProfileError::new(ProfileKind::Mdlx, ProfileViolation::CameraProjection),
                ));
            }
        }
    }
    let mut ordinary_selection: Option<Vec<AnimationId>> = None;
    for (id, clip) in value.clips() {
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
            return Err(Error::at(
                Location::for_owner(id).field("animations"),
                ProfileError::new(ProfileKind::Mdlx, ProfileViolation::SharedClockSelection),
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
            return Err(Error::at(
                Location::for_owner(id).field("animations"),
                ProfileError::new(
                    ProfileKind::Mdlx,
                    ProfileViolation::OrdinaryAnimationSelection {
                        expected: ordinary_selection
                            .as_ref()
                            .expect("previous selection")
                            .clone(),
                        actual: selection,
                    },
                ),
            ));
        }
        ordinary_selection = Some(selection);
    }
    for channel in value.tracks() {
        validate_channel(channel)?;
    }
    for (id, instance) in value.instances() {
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
                return Err(Error::at(
                    Location::for_owner(id).field("node"),
                    ProfileError::new(
                        ProfileKind::Mdlx,
                        ProfileViolation::SkinnedPlacementTransform,
                    ),
                ));
            }
        }
    }
    for (id, node) in value.nodes() {
        if value.values().get(node.visible) != Some(&true) || value.is_animated(node.visible) {
            return Err(Error::at(
                Location::for_owner(id).field("visible"),
                ProfileError::new(ProfileKind::Mdlx, ProfileViolation::NodeVisibility),
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
            return Err(Error::at(
                Location::for_owner(channel.target()),
                ProfileError::new(
                    ProfileKind::Mdlx,
                    ProfileViolation::TrackValue {
                        target: channel.target(),
                    },
                ),
            ));
        }
    };
    if matches!(
        mode,
        Interpolation::Tcb | Interpolation::Slerp | Interpolation::Nlerp
    ) {
        return Err(Error::at(
            Location::for_owner(channel.target()).field("interpolation"),
            ProfileError::new(
                ProfileKind::Mdlx,
                ProfileViolation::Interpolation { actual: mode },
            ),
        ));
    }
    Ok(())
}

pub(super) fn resources(value: &Model<'_, Mdlx>) -> Result<(), Error> {
    use crate::model::{AddressMode, ColorSpace};
    for texture in value.sampled_images() {
        if texture.color_space != ColorSpace::Unspecified {
            return Err(Error::at(
                Location::for_owner(texture.resource).field("color_space"),
                ProfileError::new(
                    ProfileKind::Mdlx,
                    ProfileViolation::ColorSpace {
                        actual: texture.color_space,
                    },
                ),
            ));
        }
        if !matches!(texture.sampler.u, AddressMode::Repeat | AddressMode::Clamp)
            || !matches!(texture.sampler.v, AddressMode::Repeat | AddressMode::Clamp)
        {
            return Err(Error::at(
                Location::for_owner(texture.resource).field("sampler"),
                ProfileError::new(
                    ProfileKind::Mdlx,
                    ProfileViolation::AddressMode {
                        u: texture.sampler.u,
                        v: texture.sampler.v,
                    },
                ),
            ));
        }
    }
    Ok(())
}
