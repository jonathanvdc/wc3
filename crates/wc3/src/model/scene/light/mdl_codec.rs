//! Derived light fields and adapters for version-selected binary storage.
use super::{
    FalloffField, Light, LightFalloff, LightShadowRange, LightTrack, ShadowCastingField,
    ShadowIntensityField, ShadowRangeField,
};
use crate::model::mdl;
use crate::model::mdl::{MdlWriter, Parser, ReadErrorKind, Span};
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::{Color, ModelVersion, Node};
use std::io::Write as IoWrite;
use std::marker::PhantomData;
fn zero() -> f32 {
    0.0
}
fn white() -> Color {
    [1.0; 3]
}
fn quadratic() -> f32 {
    LightFalloff::default().quadratic
}
fn damping() -> f32 {
    LightFalloff::default().damping
}
fn is_zero(value: &f32) -> bool {
    value.to_bits() == 0
}
fn read_shadow_intensity<V: ModelVersion>(parser: &mut Parser<'_>) -> Result<f32, mdl::ReadError> {
    if V::ShadowIntensity::default().shadow_intensity().is_none() {
        return Err(parser.error(ReadErrorKind::UnsupportedField));
    }
    parser.read()
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(
    block = "Light",
    write_order(
        node,
        kind,
        attenuation_start,
        attenuation_end,
        color,
        intensity,
        ambient_color,
        ambient_intensity,
        shadow_intensity,
        casting,
        shadow_start,
        shadow_end,
        quadratic,
        linear,
        damping,
        visibility,
        tracks
    ),
    after_read = "Self::finish",
    validate_read = "Self::validate_read",
    validate_write = "Self::validate_write"
)]
struct LightMdl<V: ModelVersion> {
    #[mdl(flatten)]
    node: Node,
    #[mdl(flags(Omnidirectional = 1, Directional = 2, Ambient = 4))]
    kind: u32,
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdl(flag = "ShadowCasting", default)]
    casting: bool,
    #[mdl(
        static_property = "ShadowIntensity",
        default,
        skip_if = "is_zero",
        read_with = "read_shadow_intensity::<V>"
    )]
    shadow_intensity: f32,
    #[mdl(
        animatable = "AttenuationStart",
        track = "LightTrack::AttenuationStart",
        default = "zero"
    )]
    attenuation_start: f32,
    #[mdl(
        animatable = "AttenuationEnd",
        track = "LightTrack::AttenuationEnd",
        default = "zero"
    )]
    attenuation_end: f32,
    #[mdl(animatable = "Color", track = "LightTrack::Color", default = "white")]
    color: Color,
    #[mdl(
        animatable = "Intensity",
        track = "LightTrack::Intensity",
        default = "zero"
    )]
    intensity: f32,
    #[mdl(
        animatable = "AmbColor",
        track = "LightTrack::AmbientColor",
        default = "white"
    )]
    ambient_color: Color,
    #[mdl(
        animatable = "AmbIntensity",
        track = "LightTrack::AmbientIntensity",
        default = "zero"
    )]
    ambient_intensity: f32,
    #[mdl(
        animatable = "Visibility",
        track = "LightTrack::Visibility",
        default = "zero",
        animated_only
    )]
    visibility: f32,
    #[mdl(
        animatable = "ShadowCastingStart",
        track = "LightTrack::ShadowCastingStart",
        default = "zero",
        enabled_if = "Self::has_range",
        enable_with = "Self::mark_range"
    )]
    shadow_start: f32,
    #[mdl(
        animatable = "ShadowCastingEnd",
        track = "LightTrack::ShadowCastingEnd",
        default = "zero",
        enabled_if = "Self::has_range",
        enable_with = "Self::mark_range"
    )]
    shadow_end: f32,
    #[mdl(
        animatable = "QuadraticFalloff",
        track = "LightTrack::QuadraticFalloff",
        default = "quadratic",
        enabled_if = "Self::has_falloff",
        enable_with = "Self::mark_falloff"
    )]
    quadratic: f32,
    #[mdl(
        animatable = "LinearFalloff",
        track = "LightTrack::LinearFalloff",
        default = "zero",
        enabled_if = "Self::has_falloff",
        enable_with = "Self::mark_falloff"
    )]
    linear: f32,
    #[mdl(
        animatable = "Damping",
        track = "LightTrack::Damping",
        default = "damping",
        enabled_if = "Self::has_falloff",
        enable_with = "Self::mark_falloff"
    )]
    damping: f32,
    #[mdl(tracks)]
    tracks: Vec<LightTrack>,
    #[mdl(skip, default)]
    range_present: bool,
    #[mdl(skip, default)]
    falloff_present: bool,
}
impl<V: ModelVersion> LightMdl<V> {
    fn has_range(&self) -> bool {
        V::ShadowRange::default().shadow_casting_range().is_some()
    }
    fn has_falloff(&self) -> bool {
        V::Falloff::default().falloff().is_some()
    }
    fn mark_range(&mut self) {
        self.range_present = true;
    }
    fn mark_falloff(&mut self) {
        self.falloff_present = true;
    }
    fn finish(&mut self, _: Span) -> Result<(), mdl::ReadError> {
        set_node_kind(&mut self.node, 0x200);
        Ok(())
    }
    fn validate_read(&self, span: Span) -> Result<(), mdl::ReadError> {
        if !self.kind.is_power_of_two() {
            return Err(mdl::ReadError::new(
                span,
                ReadErrorKind::Expected("one light-type flag"),
            ));
        }
        if (self.casting && V::ShadowCasting::default().shadow_casting().is_none())
            || (self.range_present && !self.has_range())
            || (self.falloff_present && !self.has_falloff())
        {
            return Err(mdl::ReadError::new(span, ReadErrorKind::UnsupportedField));
        }
        Ok(())
    }
    fn validate_write(&self) -> Result<(), mdl::WriteError> {
        validate_node_kind(&self.node, 0x200)?;
        if !self.kind.is_power_of_two() {
            return Err(mdl::WriteError::Unsupported("light type"));
        }
        Ok(())
    }
}
impl<V: ModelVersion> mdl::Read for Light<V> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let value = parser.read::<LightMdl<V>>()?;
        let mut light = Self::new(value.node, value.kind.trailing_zeros());
        light.attenuation_start = value.attenuation_start;
        light.attenuation_end = value.attenuation_end;
        light.color = value.color;
        light.intensity = value.intensity;
        light.ambient_color = value.ambient_color;
        light.ambient_intensity = value.ambient_intensity;
        if let Some(casting) = light.shadow_casting.shadow_casting_mut() {
            *casting = u32::from(value.casting);
        }
        if let Some(intensity) = light.shadow_intensity.shadow_intensity_mut() {
            *intensity = value.shadow_intensity;
        }
        if let Some(range) = light.shadow_range.shadow_casting_range_mut() {
            *range = LightShadowRange {
                start: value.shadow_start,
                end: value.shadow_end,
            };
        }
        if let Some(falloff) = light.falloff.falloff_mut() {
            *falloff = LightFalloff {
                quadratic: value.quadratic,
                linear: value.linear,
                damping: value.damping,
            };
        }
        light.tracks = value.tracks;
        Ok(light)
    }
}
impl<V: ModelVersion> mdl::Write for Light<V> {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        if self.light_type > 2 {
            return Err(mdl::WriteError::Unsupported("light type"));
        }
        let casting = self.shadow_casting.shadow_casting().unwrap_or(0);
        if casting > 1 {
            return Err(mdl::WriteError::Unsupported("nonboolean shadow casting"));
        }
        let range = self.shadow_range.shadow_casting_range().unwrap_or_default();
        let falloff = self.falloff.falloff().unwrap_or_default();
        writer.write(&LightMdl::<V> {
            node: self.node.clone(),
            kind: 1 << self.light_type,
            version: PhantomData,
            casting: casting != 0,
            shadow_intensity: self.shadow_intensity.shadow_intensity().unwrap_or(0.0),
            attenuation_start: self.attenuation_start,
            attenuation_end: self.attenuation_end,
            color: self.color,
            intensity: self.intensity,
            ambient_color: self.ambient_color,
            ambient_intensity: self.ambient_intensity,
            visibility: 0.0,
            shadow_start: range.start,
            shadow_end: range.end,
            quadratic: falloff.quadratic,
            linear: falloff.linear,
            damping: falloff.damping,
            tracks: self.tracks.clone(),
            range_present: false,
            falloff_present: false,
        })
    }
}
