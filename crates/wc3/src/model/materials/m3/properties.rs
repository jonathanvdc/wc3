//! One structural schema for native binding construction and pass-field selection.
use super::{
    CompositeMaterialStep, CompositeSection, CreepMaterialStep, DataDrivenMaterialStep,
    DisplacementMaterialStep, HairMaterialStep, LensFlareMaterialStep, MaterialStep, Property,
    ReflectionMaterialStep, SplatTerrainBakeMaterialStep, StandardMaterialStep,
    TerrainMaterialStep, TextureLayer, VolumeMaterialStep, VolumeNoiseMaterialStep,
};
use crate::model::profiles::m3::properties::{Field, FieldPath, FieldSegment};

pub(crate) enum PropertyMut<'a> {
    Scalar(&'a mut Property<f32>),
    Vector2(&'a mut Property<[f32; 2]>),
    Vector3(&'a mut Property<[f32; 3]>),
    Vector4(&'a mut Property<[f32; 4]>),
    Unsigned(&'a mut Property<u32>),
    Unsigned16(&'a mut Property<u16>),
}
pub(crate) trait PropertyValue: Sized {
    fn as_mut(value: &mut Property<Self>) -> PropertyMut<'_>;
}
macro_rules! property_values {
    ($($ty:ty => $variant:ident),* $(,)?) => {$(
        impl PropertyValue for $ty {
            fn as_mut(value: &mut Property<Self>) -> PropertyMut<'_> { PropertyMut::$variant(value) }
        }
    )*};
}
property_values! { f32 => Scalar, [f32; 2] => Vector2, [f32; 3] => Vector3, [f32; 4] => Vector4, u32 => Unsigned, u16 => Unsigned16 }

pub(crate) trait MaterialProperties {
    fn supports_path(&self, segments: &[FieldSegment]) -> bool;
    fn visit_properties_mut(
        &mut self,
        path: &FieldPath,
        visitor: &mut dyn FnMut(PropertyMut<'_>, &FieldPath),
    );
}
impl<T: PropertyValue> MaterialProperties for Property<T> {
    fn supports_path(&self, segments: &[FieldSegment]) -> bool {
        segments.is_empty()
    }
    fn visit_properties_mut(
        &mut self,
        path: &FieldPath,
        visitor: &mut dyn FnMut(PropertyMut<'_>, &FieldPath),
    ) {
        visitor(T::as_mut(self), path);
    }
}
impl<T: MaterialProperties> MaterialProperties for Option<T> {
    fn supports_path(&self, segments: &[FieldSegment]) -> bool {
        self.as_ref()
            .is_some_and(|value| value.supports_path(segments))
    }
    fn visit_properties_mut(
        &mut self,
        path: &FieldPath,
        visitor: &mut dyn FnMut(PropertyMut<'_>, &FieldPath),
    ) {
        if let Some(value) = self {
            value.visit_properties_mut(path, visitor);
        }
    }
}
impl<T: MaterialProperties> MaterialProperties for Vec<T> {
    fn supports_path(&self, segments: &[FieldSegment]) -> bool {
        match segments.split_first() {
            Some((FieldSegment::Index(index), rest)) => self
                .get(*index)
                .is_some_and(|value| value.supports_path(rest)),
            _ => false,
        }
    }
    fn visit_properties_mut(
        &mut self,
        path: &FieldPath,
        visitor: &mut dyn FnMut(PropertyMut<'_>, &FieldPath),
    ) {
        for (index, value) in self.iter_mut().enumerate() {
            value.visit_properties_mut(&path.indexed(index), visitor);
        }
    }
}
macro_rules! properties {
    ($ty:ty { $($field:ident => $($member:ident).+),* $(,)? }) => {
        impl MaterialProperties for $ty {
            fn supports_path(&self, segments: &[FieldSegment]) -> bool {
                match segments.split_first() {
                    $(Some((FieldSegment::Field(Field::$field), rest)) => self.$($member).+.supports_path(rest),)*
                    _ => false,
                }
            }
            fn visit_properties_mut(&mut self, path: &FieldPath, visitor: &mut dyn FnMut(PropertyMut<'_>, &FieldPath)) {
                $(self.$($member).+.visit_properties_mut(&path.child(Field::$field), visitor);)*
                let _ = (path, visitor);
            }
        }
    };
}
properties! { ReflectionMaterialStep {
    ReflectionStrength => reflection.parameters, DisplacementStrength => displacement.parameters,
    ReflectionOffset => reflection_offset, BlurAngle => blur_angle, BlurDistanceMax => blur_distance_max,
    ReflectionMap => reflection.input, DisplacementMap => displacement.input, BlurMap => blur_map,
}}
properties! { StandardMaterialStep {
    DiffuseLayer => diffuse, DecalLayer => decal_layer, SpecularLayer => specular.input.color,
    GlossLayer => specular.input.gloss, EmissiveLayer1 => emissive.input.primary.input,
    EmissiveLayer2 => emissive.input.secondary.input, EnvironmentLayer => environment_layer,
    EnvironmentMaskLayer => environment_mask_layer, AlphaLayer1 => opacity.primary, AlphaLayer2 => opacity.secondary,
    NormalLayer => normals.base, HeightLayer => height_layer, LightMapLayer => light_map_layer,
    AmbientOcclusionLayer => ambient_occlusion_layer, NormalBlend1MaskLayer => normals.primary.mask,
    NormalBlend2MaskLayer => normals.secondary.mask, NormalBlend1Layer => normals.primary.input,
    NormalBlend2Layer => normals.secondary.input, ParallaxHeight => parallax_height,
    MotionBlurAmount => motion_blur_amount, NormalBlendFactors => normals.weights,
}}
properties! { DataDrivenMaterialStep<'_> {} }
properties! { LensFlareMaterialStep<'_> {
    FlareMap => flare_map, MaskMap => mask_map, Intensity => intensity, Color => color, Hdr => hdr, Size => size,
}}
properties! { DisplacementMaterialStep { Strength => displacement.parameters, NormalMap => displacement.input, StrengthMap => strength_map, } }
properties! { TerrainMaterialStep { TerrainMap => terrain_map, } }
properties! { CreepMaterialStep { MaskMap => mask_map, } }
properties! { SplatTerrainBakeMaterialStep { DiffuseMap => diffuse_map, NormalMap => normal_map, SpecularMap => specular_map, } }
properties! { HairMaterialStep {
    LayerBase => diffuse.input, LayerSpecShift => layer_spec_shift, LayerSpecNoise => layer_spec_noise,
    LayerAmbientOcclusion => layer_ambient_occlusion, ColorDiffuse => diffuse.parameters, ColorSpecular => color_specular,
}}
properties! { CompositeSection { MapMultiplier => map_multiplier, } }
properties! { CompositeMaterialStep { Sections => sections, } }
properties! { TextureLayer {
    Color => binding.contribution.color, RgbMultiply => binding.contribution.rgb_multiply,
    RgbAdd => binding.contribution.rgb_add, AviPlay => binding.sampling.avi_play,
    AviRestart => binding.sampling.avi_restart, CurrentFrame => binding.sampling.current_frame,
    UvOffset => binding.coordinates.transform.uv_offset, UvAngle => binding.coordinates.transform.uv_angle,
    UvTiling => binding.coordinates.transform.uv_tiling, WOffset => binding.coordinates.transform.w_offset,
    WTiling => binding.coordinates.transform.w_tiling, MapAlpha => binding.contribution.map_alpha,
    TriplanarOffset => binding.coordinates.transform.triplanar_offset, TriplanarScale => binding.coordinates.transform.triplanar_scale,
}}
properties! { VolumeMaterialStep { Density => density, ColorMap => color_map, NoiseMap1 => noise_map1, NoiseMap2 => noise_map2, } }
properties! { VolumeNoiseMaterialStep {
    Density => density, NearPlane => near_plane, Falloff => falloff, ColorMap => color_map,
    NoiseMap1 => noise_map1, NoiseMap2 => noise_map2, ScrollRate => scroll_rate, Position => position,
    Scale => scale, Rotation => rotation,
}}

macro_rules! material_variants {
    ($($variant:ident),* $(,)?) => {
        impl MaterialProperties for MaterialStep<'_> {
            fn supports_path(&self, segments: &[FieldSegment]) -> bool {
                match self { $(Self::$variant(value) => value.supports_path(segments),)* }
            }
            fn visit_properties_mut(&mut self, path: &FieldPath, visitor: &mut dyn FnMut(PropertyMut<'_>, &FieldPath)) {
                match self { $(Self::$variant(value) => value.visit_properties_mut(path, visitor),)* }
            }
        }
    };
}
material_variants! { Standard, Displacement, Composite, Terrain, Volume, VolumeNoise, Hair, Creep, SplatTerrainBake, Reflection, LensFlare, DataDriven }
