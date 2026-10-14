//! Resource alternatives and common scene parameters retain their own authoring meaning.
use super::super::text;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::actors::AssetAddress;
use crate::model::wem::{records, RawDocument};

pub(in crate::model::conversion::wem) fn read_asset<'a>(
    document: &'a RawDocument<'_>,
    value: &records::AssetKey,
) -> Result<AssetAddress<'a>, Error> {
    Ok(AssetAddress {
        namespace: value.group,
        numeric: (value.id != u32::MAX).then_some(value.id),
        path: text::read_bytes(document, value.path)?,
    })
}
pub(in crate::model::conversion::wem) fn write_asset(
    document: &mut RawDocument<'_>,
    value: &AssetAddress<'_>,
) -> Result<records::AssetKey, Error> {
    Ok(records::AssetKey {
        group: value.namespace,
        id: value.numeric.unwrap_or(u32::MAX),
        path: text::write_bytes(document, &value.path)?,
    })
}

use crate::model::semantic::profiles::wem::actors::{
    Camera, DepthOfField, Light, LightKind, LightVolume,
};
use crate::model::semantic::Animated;
use crate::model::wem::Bool;

pub(in crate::model::conversion::wem) trait Field<T>:
    Sized
{
    fn read(&self) -> Option<T>;
    fn write(value: Option<T>) -> Result<Self, Error>;
}
impl<T> Field<T> for () {
    fn read(&self) -> Option<T> {
        None
    }
    fn write(value: Option<T>) -> Result<Self, Error> {
        if value.is_some() {
            Err(Error::new(
                "wem.actor.capability",
                "selected node vocabulary omits an authored parameter",
            ))
        } else {
            Ok(())
        }
    }
}
macro_rules! direct_field {
    ($type:ty) => {
        impl Field<$type> for $type {
            fn read(&self) -> Option<$type> {
                Some(*self)
            }
            fn write(value: Option<$type>) -> Result<Self, Error> {
                value.ok_or_else(|| {
                    Error::new(
                        "wem.actor.capability",
                        "selected node vocabulary requires an undeclared parameter",
                    )
                })
            }
        }
    };
}
direct_field!(f32);
direct_field!([f32; 3]);
impl Field<bool> for Bool {
    fn read(&self) -> Option<bool> {
        Some(self.get())
    }
    fn write(value: Option<bool>) -> Result<Self, Error> {
        value.map(Self::from).ok_or_else(|| {
            Error::new(
                "wem.actor.capability",
                "selected node vocabulary requires an undeclared switch",
            )
        })
    }
}

pub(in crate::model::conversion::wem) fn read_light<D: records::LightPayloadDialect>(
    source: &records::LightPayload<D>,
) -> Result<Light, Error>
where
    D::ShadowCasting: Field<bool>,
    D::ShadowCastingStart: Field<f32>,
    D::ShadowCastingEnd: Field<f32>,
    D::QuadraticFalloff: Field<f32>,
    D::LinearFalloff: Field<f32>,
    D::Damping: Field<f32>,
{
    let volume = if let Some(shadow_casting) = source.shadow_casting.read() {
        Some(LightVolume {
            shadow_casting,
            shadow_range: [
                source
                    .shadow_casting_start
                    .read()
                    .ok_or_else(|| Error::new("wem.light", "incomplete shadow controls"))?,
                source
                    .shadow_casting_end
                    .read()
                    .ok_or_else(|| Error::new("wem.light", "incomplete shadow controls"))?,
            ],
            attenuation_coefficients: [
                source
                    .quadratic_falloff
                    .read()
                    .ok_or_else(|| Error::new("wem.light", "incomplete falloff controls"))?,
                source
                    .linear_falloff
                    .read()
                    .ok_or_else(|| Error::new("wem.light", "incomplete falloff controls"))?,
            ],
            damping: source
                .damping
                .read()
                .ok_or_else(|| Error::new("wem.light", "incomplete damping control"))?,
        })
    } else {
        None
    };
    Ok(Light {
        kind: match source.kind {
            records::LightKind::Omni => LightKind::Omnidirectional,
            records::LightKind::Directional => LightKind::Directional,
            records::LightKind::Spot => LightKind::Spot,
            records::LightKind::Ambient => LightKind::Ambient,
            records::LightKind::Unknown(code) => LightKind::Unknown(code),
        },
        color: Animated::constant(source.color),
        intensity: Animated::constant(source.intensity),
        attenuation: [
            Animated::constant(source.attenuation_start),
            Animated::constant(source.attenuation_end),
        ],
        cone: [source.hot_spot, source.falloff],
        volume,
    })
}
pub(in crate::model::conversion::wem) fn write_light<D: records::LightPayloadDialect>(
    source: &Light,
) -> Result<records::LightPayload<D>, Error>
where
    D::ShadowCasting: Field<bool>,
    D::ShadowCastingStart: Field<f32>,
    D::ShadowCastingEnd: Field<f32>,
    D::QuadraticFalloff: Field<f32>,
    D::LinearFalloff: Field<f32>,
    D::Damping: Field<f32>,
{
    Ok(records::LightPayload {
        kind: match source.kind {
            LightKind::Omnidirectional => records::LightKind::Omni,
            LightKind::Directional => records::LightKind::Directional,
            LightKind::Spot => records::LightKind::Spot,
            LightKind::Ambient => records::LightKind::Ambient,
            LightKind::Unknown(code) => records::LightKind::Unknown(code),
        },
        color: source.color.default,
        intensity: source.intensity.default,
        attenuation_start: source.attenuation[0].default,
        attenuation_end: source.attenuation[1].default,
        hot_spot: source.cone[0],
        falloff: source.cone[1],
        shadow_casting: D::ShadowCasting::write(source.volume.as_ref().map(|v| v.shadow_casting))?,
        shadow_casting_start: D::ShadowCastingStart::write(
            source.volume.as_ref().map(|v| v.shadow_range[0]),
        )?,
        shadow_casting_end: D::ShadowCastingEnd::write(
            source.volume.as_ref().map(|v| v.shadow_range[1]),
        )?,
        quadratic_falloff: D::QuadraticFalloff::write(
            source
                .volume
                .as_ref()
                .map(|v| v.attenuation_coefficients[0]),
        )?,
        linear_falloff: D::LinearFalloff::write(
            source
                .volume
                .as_ref()
                .map(|v| v.attenuation_coefficients[1]),
        )?,
        damping: D::Damping::write(source.volume.as_ref().map(|v| v.damping))?,
    })
}
pub(in crate::model::conversion::wem) fn read_camera<D: records::CameraPayloadDialect>(
    source: &records::CameraPayload<D>,
) -> Result<Camera, Error>
where
    D::Target: Field<[f32; 3]>,
    D::FocusDistance: Field<f32>,
    D::FocalLength: Field<f32>,
    D::FStop: Field<f32>,
{
    let depth_of_field = if let Some(focus_distance) = source.focus_distance.read() {
        Some(DepthOfField {
            focus_distance,
            focal_length: source
                .focal_length
                .read()
                .ok_or_else(|| Error::new("wem.camera", "incomplete lens controls"))?,
            aperture: source
                .f_stop
                .read()
                .ok_or_else(|| Error::new("wem.camera", "incomplete aperture control"))?,
        })
    } else {
        None
    };
    Ok(Camera {
        field_of_view: Animated::constant(source.fov),
        clipping: [
            Animated::constant(source.near_clip),
            Animated::constant(source.far_clip),
        ],
        target: source.target.read().map(Animated::constant),
        depth_of_field,
    })
}
pub(in crate::model::conversion::wem) fn write_camera<D: records::CameraPayloadDialect>(
    source: &Camera,
) -> Result<records::CameraPayload<D>, Error>
where
    D::Target: Field<[f32; 3]>,
    D::FocusDistance: Field<f32>,
    D::FocalLength: Field<f32>,
    D::FStop: Field<f32>,
{
    Ok(records::CameraPayload {
        fov: source.field_of_view.default,
        near_clip: source.clipping[0].default,
        far_clip: source.clipping[1].default,
        target: D::Target::write(source.target.as_ref().map(|v| v.default))?,
        focus_distance: D::FocusDistance::write(
            source.depth_of_field.as_ref().map(|v| v.focus_distance),
        )?,
        focal_length: D::FocalLength::write(
            source.depth_of_field.as_ref().map(|v| v.focal_length),
        )?,
        f_stop: D::FStop::write(source.depth_of_field.as_ref().map(|v| v.aperture))?,
    })
}
