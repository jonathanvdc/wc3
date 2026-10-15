//! Full light and camera properties translated to native records.
use super::encode_animation::Encoder;
use crate::model::conversion::Error;
use crate::model::mdlx::{self as native, ModelVersion};
use crate::model::semantic::profiles::mdlx as profile;
use crate::model::semantic::NodeId;
use native::scene::Node;
use std::collections::BTreeMap;

pub(super) fn light<'a, V: ModelVersion>(
    source: &profile::Light,
    node: Node<'a>,
    encoder: &mut Encoder<'_, '_, '_>,
) -> Result<native::Light<'a, V>, Error> {
    let kind = match source.kind {
        profile::LightKind::Omni => native::LightType::Omnidirectional,
        profile::LightKind::Directional => native::LightType::Directional,
        profile::LightKind::Ambient => native::LightType::Ambient,
        profile::LightKind::Unknown(value) => native::LightType::Unknown(value),
    };
    let mut record = native::Light::<V>::new(node, kind);
    record.attenuation_start =
        encoder.value(&source.attenuation[0], Ok, "light.attenuation_start")?;
    record.attenuation_end = encoder.value(&source.attenuation[1], Ok, "light.attenuation_end")?;
    record.color = encoder.value(&source.diffuse.color, Ok, "light.color")?;
    record.intensity = encoder.value(&source.diffuse.intensity, Ok, "light.intensity")?;
    record.ambient_color = encoder.value(&source.ambient.color, Ok, "light.ambient_color")?;
    record.ambient_intensity =
        encoder.value(&source.ambient.intensity, Ok, "light.ambient_intensity")?;
    record.visibility = super::encode_helpers::visibility(
        &source.activation.visibility,
        encoder,
        "light.visibility",
    )?;
    if let Some(value) = source.activation.shadow_casting {
        if let Err(error) = record.try_set_shadow_casting(value) {
            encoder
                .context
                .unsupported("light.shadow_casting", error.to_string())?;
        }
    }
    if let Some(value) = source.activation.shadow_intensity {
        if let Err(error) = record.try_set_shadow_intensity(value) {
            encoder
                .context
                .unsupported("light.shadow_intensity", error.to_string())?;
        }
    }
    if let Some(value) = &source.activation.shadow_range {
        let range = native::scene::LightShadowRange {
            start: encoder.value(&value[0], Ok, "light.shadow_start")?,
            end: encoder.value(&value[1], Ok, "light.shadow_end")?,
        };
        if let Err(error) = record.try_set_shadow_casting_range(range) {
            encoder
                .context
                .unsupported("light.shadow_range", error.to_string())?;
        }
    }
    if let Some(value) = &source.activation.falloff {
        let falloff = native::scene::LightFalloff {
            quadratic: encoder.value(&value[0], Ok, "light.quadratic")?,
            linear: encoder.value(&value[1], Ok, "light.linear")?,
            damping: encoder.value(&value[2], Ok, "light.damping")?,
        };
        if let Err(error) = record.try_set_falloff(falloff) {
            encoder
                .context
                .unsupported("light.falloff", error.to_string())?;
        }
    }
    Ok(record)
}
pub(super) fn camera<V: ModelVersion>(
    source: &profile::Camera,
    id: NodeId,
    nodes: &BTreeMap<NodeId, Node<'static>>,
    encoder: &mut Encoder<'_, '_, '_>,
) -> Result<native::Camera<'static, V>, Error> {
    let graph_node = encoder
        .model
        .node(id)
        .ok_or_else(|| Error::new("camera.node", "camera anchor does not exist"))?;
    if graph_node.parent.is_some() {
        encoder.context.unsupported("camera.parent", "native Warcraft III cameras have model-space eye and target positions without a hierarchy parent")?;
    }
    let mut record = native::Camera::<V>::new(&graph_node.name)
        .map_err(|error| Error::new("camera.name", error.to_string()))?;
    record.position = graph_node.transform.translation;
    record.target_position = source.target;
    record.field_of_view = source.field_of_view;
    record.near_clip = source.clipping[0];
    record.far_clip = source.clipping[1];
    record.translation = source
        .eye_offset
        .property
        .as_ref()
        .map(|property| encoder.track(property, Ok, "camera.eye_offset"))
        .transpose()?
        .flatten()
        .or_else(|| {
            (source.eye_offset.default != [0.; 3])
                .then(|| native::animation::Track::constant(source.eye_offset.default))
        });
    record.target_translation = source
        .target_offset
        .property
        .as_ref()
        .map(|property| encoder.track(property, Ok, "camera.target_offset"))
        .transpose()?
        .flatten()
        .or_else(|| {
            (source.target_offset.default != [0.; 3])
                .then(|| native::animation::Track::constant(source.target_offset.default))
        });
    record.rotation = source
        .roll
        .property
        .as_ref()
        .map(|property| encoder.track(property, Ok, "camera.roll"))
        .transpose()?
        .flatten()
        .or_else(|| {
            (source.roll.default != 0.)
                .then(|| native::animation::Track::constant(source.roll.default))
        });
    record.visibility = super::encode_helpers::visibility(
        &source.convention.visibility,
        encoder,
        "camera.visibility",
    )?;
    macro_rules! lens {
        ($field:ident) => {
            if let Some(value) = &source.convention.$field {
                record.$field = value
                    .property
                    .as_ref()
                    .map(|property| {
                        encoder.track(property, Ok, concat!("camera.", stringify!($field)))
                    })
                    .transpose()?
                    .flatten()
                    .or_else(|| Some(native::animation::Track::constant(value.default)));
            }
        };
    }
    lens!(focus_distance);
    lens!(focal_length);
    lens!(f_stop);
    let _ = nodes;
    Ok(record)
}
