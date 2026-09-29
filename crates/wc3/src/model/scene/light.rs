//! Direct and ambient lighting with version-dependent shadow settings.
use crate::model::conversion::ConversionContext;
use crate::model::Color;
use crate::model::ConversionError;
use crate::model::KnownChunk;
use crate::model::LightsChunk;
use crate::model::ValueError;
use crate::model::{mdl, mdx};
use crate::model::{Animatable, Track};
use crate::model::{Model, Node};
use crate::model::{
    ModelVersion, SupportsLightFalloff, SupportsLightShadowCasting, SupportsLightShadowIntensity,
};
use mdl_codec::{damping, is_zero, quadratic, white, zero};
use std::fmt::Debug;
use std::marker::PhantomData;

/// How a light illuminates the model. Unknown values round-trip through MDX.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    mdx::Read,
    mdx::Write,
    mdx::Value,
    mdl::Read,
    mdl::Write,
)]
#[mdx(value = u32)]
#[mdl(choice)]
pub enum LightType {
    #[default]
    #[mdx(value = 0)]
    Omnidirectional,
    #[mdx(value = 1)]
    Directional,
    #[mdx(value = 2)]
    Ambient,
    #[mdx(unknown)]
    #[mdl(unknown)]
    Unknown(u32),
}

/// Light features available in a particular model version.
pub trait ShadowCastingField: Default + mdx::Read + mdx::Write + Clone + Debug + PartialEq {
    fn shadow_casting(&self) -> Option<u32> {
        None
    }
    fn shadow_casting_mut(&mut self) -> Option<&mut u32> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoShadowCasting;
impl ShadowCastingField for NoShadowCasting {}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct ShadowCasting(u32);
impl ShadowCastingField for ShadowCasting {
    fn shadow_casting(&self) -> Option<u32> {
        Some(self.0)
    }
    fn shadow_casting_mut(&mut self) -> Option<&mut u32> {
        Some(&mut self.0)
    }
}

pub trait ShadowIntensityField:
    Default + mdx::Read + mdx::Write + Clone + Debug + PartialEq
{
    fn shadow_intensity(&self) -> Option<f32> {
        None
    }
    fn shadow_intensity_mut(&mut self) -> Option<&mut f32> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoShadowIntensity;
impl ShadowIntensityField for NoShadowIntensity {}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct LightShadowIntensity {
    pub shadow_intensity: f32,
}
impl ShadowIntensityField for LightShadowIntensity {
    fn shadow_intensity(&self) -> Option<f32> {
        Some(self.shadow_intensity)
    }
    fn shadow_intensity_mut(&mut self) -> Option<&mut f32> {
        Some(&mut self.shadow_intensity)
    }
}

pub trait ShadowRangeField:
    Default + mdx::Read + mdx::Write + mdx::ReadTracks + mdx::WriteTracks + Clone + Debug + PartialEq
{
    fn shadow_casting_range(&self) -> Option<&LightShadowRange> {
        None
    }
    fn shadow_casting_range_mut(&mut self) -> Option<&mut LightShadowRange> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoShadowRange;
impl ShadowRangeField for NoShadowRange {}

/// Start and end distances for a light's shadow-casting range.
#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct LightShadowRange {
    #[mdx(tag = *b"KLSS")]
    pub start: Animatable<f32>,
    #[mdx(tag = *b"KLSE")]
    pub end: Animatable<f32>,
}
impl ShadowRangeField for LightShadowRange {
    fn shadow_casting_range(&self) -> Option<&LightShadowRange> {
        Some(self)
    }
    fn shadow_casting_range_mut(&mut self) -> Option<&mut LightShadowRange> {
        Some(self)
    }
}

pub trait FalloffField:
    Default + mdx::Read + mdx::Write + mdx::ReadTracks + mdx::WriteTracks + Clone + Debug + PartialEq
{
    fn falloff(&self) -> Option<&LightFalloff> {
        None
    }
    fn falloff_mut(&mut self) -> Option<&mut LightFalloff> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct NoFalloff;
impl FalloffField for NoFalloff {}

/// Distance falloff coefficients, editable in version 1600 and newer.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
pub struct LightFalloff {
    #[mdx(tag = *b"KLQF")]
    pub quadratic: Animatable<f32>,
    #[mdx(tag = *b"KLLF")]
    pub linear: Animatable<f32>,
    #[mdx(tag = *b"KLDA")]
    pub damping: Animatable<f32>,
}
impl Default for LightFalloff {
    fn default() -> Self {
        Self {
            quadratic: Animatable::Static(0.0005),
            linear: Animatable::Static(0.0),
            damping: Animatable::Static(0.00001),
        }
    }
}
impl FalloffField for LightFalloff {
    fn falloff(&self) -> Option<&LightFalloff> {
        Some(self)
    }
    fn falloff_mut(&mut self) -> Option<&mut LightFalloff> {
        Some(self)
    }
}

pub trait LightLayout {
    type ShadowCasting: ShadowCastingField;
    type ShadowIntensity: ShadowIntensityField;
    type ShadowRange: ShadowRangeField;
    type Falloff: FalloffField;
}
use crate::model::{V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};
macro_rules! light_layout {
    ($version:ty, $casting:ty, $intensity:ty, $range:ty, $falloff:ty) => {
        impl LightLayout for $version {
            type ShadowCasting = $casting;
            type ShadowIntensity = $intensity;
            type ShadowRange = $range;
            type Falloff = $falloff;
        }
    };
}
light_layout!(
    V800,
    NoShadowCasting,
    NoShadowIntensity,
    NoShadowRange,
    NoFalloff
);
light_layout!(
    V900,
    NoShadowCasting,
    NoShadowIntensity,
    NoShadowRange,
    NoFalloff
);
light_layout!(
    V1000,
    NoShadowCasting,
    NoShadowIntensity,
    NoShadowRange,
    NoFalloff
);
light_layout!(
    V1100,
    NoShadowCasting,
    NoShadowIntensity,
    NoShadowRange,
    NoFalloff
);
light_layout!(
    V1200,
    NoShadowCasting,
    LightShadowIntensity,
    NoShadowRange,
    NoFalloff
);
light_layout!(
    V1300,
    ShadowCasting,
    LightShadowIntensity,
    LightShadowRange,
    NoFalloff
);
light_layout!(
    V1400,
    ShadowCasting,
    LightShadowIntensity,
    LightShadowRange,
    NoFalloff
);
light_layout!(
    V1600,
    ShadowCasting,
    LightShadowIntensity,
    LightShadowRange,
    LightFalloff
);
light_layout!(
    V1800,
    ShadowCasting,
    LightShadowIntensity,
    LightShadowRange,
    LightFalloff
);

/// A node that supplies direct and ambient lighting.
///
/// Shadow settings depend on the model version; checked accessors report
/// unavailable features. MDL supports static shadow intensity, not an animated
/// shadow-intensity channel.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = LightsChunk::<V>::TAG))]
#[mdl(block = "Light", after_read = "Self::finish_mdl", validate_write = "Self::validate_mdl",
    write_order(node, light_type, attenuation_start, attenuation_end, color, intensity,
        ambient_color, ambient_intensity, shadow_value, casting, shadow_start,
        shadow_end, quadratic, linear, damping, visibility),
    virtual_fields(
        #[mdl(flag = "ShadowCasting", default)]
        #[mdl(get = "Self::mdl_casting", set = "Self::set_mdl_casting")]
        casting: bool,
        #[mdl(
            static_property = "ShadowIntensity",
            default,
            skip_if = "is_zero"
        )]
        #[mdl(get = "Self::mdl_shadow_intensity", slot = "Self::mdl_shadow_intensity_mut")]
        shadow_value: f32,
        #[mdl(
            property = "ShadowCastingStart",
            default = "zero"
        )]
        #[mdl(get = "Self::mdl_shadow_start", slot = "Self::mdl_shadow_start_mut")]
        shadow_start: Animatable<f32>,
        #[mdl(
            property = "ShadowCastingEnd",
            default = "zero"
        )]
        #[mdl(get = "Self::mdl_shadow_end", slot = "Self::mdl_shadow_end_mut")]
        shadow_end: Animatable<f32>,
        #[mdl(
            property = "QuadraticFalloff",
            default = "quadratic"
        )]
        #[mdl(get = "Self::mdl_quadratic", slot = "Self::mdl_quadratic_mut")]
        quadratic: Animatable<f32>,
        #[mdl(
            property = "LinearFalloff",
            default = "zero"
        )]
        #[mdl(get = "Self::mdl_linear", slot = "Self::mdl_linear_mut")]
        linear: Animatable<f32>,
        #[mdl(
            property = "Damping",
            default = "damping"
        )]
        #[mdl(get = "Self::mdl_damping", slot = "Self::mdl_damping_mut")]
        damping: Animatable<f32>
    )
)]
pub struct Light<V: ModelVersion> {
    #[mdl(flatten)]
    /// Shared node.
    pub node: Node,
    #[mdl(flatten)]
    /// The kind of illumination.
    pub light_type: LightType,
    #[mdl(skip, default)]
    shadow_casting: V::ShadowCasting,
    #[mdx(tag = *b"KLAS")]
    #[mdl(property = "AttenuationStart", default = "zero")]
    /// Attenuation start distance.
    pub attenuation_start: Animatable<f32>,
    #[mdx(tag = *b"KLAE")]
    #[mdl(property = "AttenuationEnd", default = "zero")]
    /// Attenuation end distance.
    pub attenuation_end: Animatable<f32>,
    #[mdx(tag = *b"KLAC")]
    #[mdl(property = "Color", default = "white")]
    /// RGB light color.
    pub color: Animatable<Color>,
    #[mdx(tag = *b"KLAI")]
    #[mdl(property = "Intensity", default = "zero")]
    /// Light intensity.
    pub intensity: Animatable<f32>,
    #[mdx(tag = *b"KLBC")]
    #[mdl(property = "AmbColor", default = "white")]
    /// Ambient RGB color.
    pub ambient_color: Animatable<Color>,
    #[mdx(tag = *b"KLBI")]
    #[mdl(property = "AmbIntensity", default = "zero")]
    /// Ambient intensity.
    pub ambient_intensity: Animatable<f32>,
    #[mdl(skip, default)]
    shadow_intensity: V::ShadowIntensity,
    #[mdl(skip, default)]
    #[mdx(flatten)]
    shadow_range: V::ShadowRange,
    #[mdl(skip, default)]
    #[mdx(flatten)]
    falloff: V::Falloff,
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdx(tag = *b"KLAV")]
    #[mdl(property = "Visibility")]
    /// Optional visibility animation.
    pub visibility: Option<Track<f32>>,
}

impl<V: ModelVersion> Light<V> {
    /// Creates a light with white colors and the format defaults for versioned fields.
    pub fn new(node: Node, light_type: LightType) -> Self {
        Self {
            node,
            light_type,
            shadow_casting: V::ShadowCasting::default(),
            attenuation_start: Animatable::Static(0.0),
            attenuation_end: Animatable::Static(0.0),
            color: Animatable::Static([1.0; 3]),
            intensity: Animatable::Static(0.0),
            ambient_color: Animatable::Static([1.0; 3]),
            ambient_intensity: Animatable::Static(0.0),
            shadow_intensity: V::ShadowIntensity::default(),
            shadow_range: V::ShadowRange::default(),
            falloff: V::Falloff::default(),
            visibility: None,
            version: PhantomData,
        }
    }

    /// Returns the shadow-casting flag available from version 1300.
    pub fn try_shadow_casting(&self) -> Result<bool, ValueError> {
        self.shadow_casting
            .shadow_casting()
            .map(|value| value != 0)
            .ok_or(ValueError::UnsupportedVersion {
                tag: LightsChunk::<V>::TAG,
                minimum: 1300,
                actual: V::NUMBER,
            })
    }
    /// Sets the shadow-casting flag available from version 1300.
    pub fn try_set_shadow_casting(&mut self, enabled: bool) -> Result<(), ValueError> {
        *self
            .shadow_casting
            .shadow_casting_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LightsChunk::<V>::TAG,
                minimum: 1300,
                actual: V::NUMBER,
            })? = u32::from(enabled);
        Ok(())
    }
    /// Returns shadow intensity, available from version 1200.
    pub fn try_shadow_intensity(&self) -> Result<f32, ValueError> {
        self.shadow_intensity
            .shadow_intensity()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LightsChunk::<V>::TAG,
                minimum: 1200,
                actual: V::NUMBER,
            })
    }
    /// Sets shadow intensity, available from version 1200.
    pub fn try_set_shadow_intensity(&mut self, value: f32) -> Result<(), ValueError> {
        *self
            .shadow_intensity
            .shadow_intensity_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LightsChunk::<V>::TAG,
                minimum: 1200,
                actual: V::NUMBER,
            })? = value;
        Ok(())
    }
    /// Returns shadow-casting start and end, available from version 1300.
    pub fn try_shadow_casting_range(&self) -> Result<LightShadowRange, ValueError> {
        self.shadow_range
            .shadow_casting_range()
            .cloned()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LightsChunk::<V>::TAG,
                minimum: 1300,
                actual: V::NUMBER,
            })
    }
    /// Sets shadow-casting start and end, available from version 1300.
    pub fn try_set_shadow_casting_range(
        &mut self,
        range: LightShadowRange,
    ) -> Result<(), ValueError> {
        *self
            .shadow_range
            .shadow_casting_range_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LightsChunk::<V>::TAG,
                minimum: 1300,
                actual: V::NUMBER,
            })? = range;
        Ok(())
    }
    /// Returns the falloff values used by the game, including older-version defaults.
    pub fn falloff(&self) -> LightFalloff {
        self.falloff.falloff().cloned().unwrap_or_default()
    }
    /// Sets the three serialized falloff fields from version 1600 onward.
    pub fn try_set_falloff(&mut self, falloff: LightFalloff) -> Result<(), ValueError> {
        *self
            .falloff
            .falloff_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: LightsChunk::<V>::TAG,
                minimum: 1600,
                actual: V::NUMBER,
            })? = falloff;
        Ok(())
    }
}

impl<V: SupportsLightShadowIntensity> Light<V> {
    /// Returns shadow intensity.
    pub fn shadow_intensity(&self) -> f32 {
        self.try_shadow_intensity().expect("supported version")
    }

    /// Sets shadow intensity.
    pub fn set_shadow_intensity(&mut self, value: f32) {
        self.try_set_shadow_intensity(value)
            .expect("supported version");
    }
}

impl<V: SupportsLightShadowCasting> Light<V> {
    /// Returns whether shadow casting is enabled.
    pub fn shadow_casting(&self) -> bool {
        self.try_shadow_casting().expect("supported version")
    }

    /// Enables or disables shadow casting.
    pub fn set_shadow_casting(&mut self, enabled: bool) {
        self.try_set_shadow_casting(enabled)
            .expect("supported version");
    }

    /// Returns the shadow-casting range.
    pub fn shadow_casting_range(&self) -> LightShadowRange {
        self.try_shadow_casting_range().expect("supported version")
    }

    /// Sets the shadow-casting range.
    pub fn set_shadow_casting_range(&mut self, range: LightShadowRange) {
        self.try_set_shadow_casting_range(range)
            .expect("supported version");
    }
}

impl<V: SupportsLightFalloff> Light<V> {
    /// Sets the serialized falloff coefficients.
    pub fn set_falloff(&mut self, falloff: LightFalloff) {
        self.try_set_falloff(falloff).expect("supported version");
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of `LITE` records in file order.
    pub fn lights(&self) -> Vec<Light<V>> {
        self.collect_chunk_records::<LightsChunk<V>>()
    }

    /// Replaces lights in the first `LITE` chunk.
    pub fn set_lights(&mut self, lights: &[Light<V>]) {
        self.replace_chunk(LightsChunk::new(lights.to_vec()));
    }
}

impl<V: ModelVersion> Light<V> {
    pub(crate) fn convert_with<T: ModelVersion>(
        &self,
        context: &mut ConversionContext<'_>,
        path: &str,
    ) -> Result<Light<T>, ConversionError> {
        let mut target = Light::<T>::new(self.node.clone(), self.light_type);
        target.attenuation_start = self.attenuation_start.clone();
        target.attenuation_end = self.attenuation_end.clone();
        target.color = self.color.clone();
        target.intensity = self.intensity.clone();
        target.ambient_color = self.ambient_color.clone();
        target.ambient_intensity = self.ambient_intensity.clone();

        context.field(
            self.shadow_casting.shadow_casting(),
            target.shadow_casting.shadow_casting_mut(),
            0,
            &format!("{path}.shadow_casting"),
        )?;
        context.field(
            self.shadow_intensity.shadow_intensity(),
            target.shadow_intensity.shadow_intensity_mut(),
            0.0,
            &format!("{path}.shadow_intensity"),
        )?;
        context.field(
            self.shadow_range.shadow_casting_range().cloned(),
            target.shadow_range.shadow_casting_range_mut(),
            LightShadowRange::default(),
            &format!("{path}.shadow_range"),
        )?;
        context.field(
            self.falloff.falloff().cloned(),
            target.falloff.falloff_mut(),
            LightFalloff::default(),
            &format!("{path}.falloff"),
        )?;
        target.visibility = self.visibility.clone();
        Ok(target)
    }
}

mod mdl_codec;
