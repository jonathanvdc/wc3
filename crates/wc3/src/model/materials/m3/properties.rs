//! Typed traversal of material animation slots during native import.
use super::{
    CompositeMaterialStep, CompositeSection, CreepMaterialStep, DataDrivenMaterialStep,
    DisplacementMaterialStep, HairMaterialStep, LensFlareMaterialStep, MaterialStep,
    PropertyBinding, ReflectionMaterialStep, SplatTerrainBakeMaterialStep, StandardMaterialStep,
    TerrainMaterialStep, TextureLayer, VolumeMaterialStep, VolumeNoiseMaterialStep,
};
use crate::model::PropertyValue;

pub(crate) enum PropertyMut<'a> {
    Scalar(&'a mut PropertyBinding<f32>),
    Vector2(&'a mut PropertyBinding<[f32; 2]>),
    Vector3(&'a mut PropertyBinding<[f32; 3]>),
    Vector4(&'a mut PropertyBinding<[f32; 4]>),
    Unsigned(&'a mut PropertyBinding<u32>),
    Unsigned16(&'a mut PropertyBinding<u16>),
}
pub(crate) trait M3PropertyValue: PropertyValue + Sized {
    fn as_mut(value: &mut PropertyBinding<Self>) -> PropertyMut<'_>;
}
macro_rules! property_values {
    ($($ty:ty => $variant:ident),* $(,)?) => {$(
        impl M3PropertyValue for $ty {
            fn as_mut(value: &mut PropertyBinding<Self>) -> PropertyMut<'_> { PropertyMut::$variant(value) }
        }
    )*};
}
property_values! { f32 => Scalar, [f32; 2] => Vector2, [f32; 3] => Vector3, [f32; 4] => Vector4, u32 => Unsigned, u16 => Unsigned16 }

pub(crate) trait MaterialProperties {
    fn visit_properties_mut(&mut self, visitor: &mut dyn FnMut(PropertyMut<'_>));
}
impl<T: M3PropertyValue> MaterialProperties for PropertyBinding<T> {
    fn visit_properties_mut(&mut self, visitor: &mut dyn FnMut(PropertyMut<'_>)) {
        visitor(T::as_mut(self));
    }
}
impl<T: MaterialProperties> MaterialProperties for Option<T> {
    fn visit_properties_mut(&mut self, visitor: &mut dyn FnMut(PropertyMut<'_>)) {
        if let Some(value) = self {
            value.visit_properties_mut(visitor);
        }
    }
}
impl<T: MaterialProperties> MaterialProperties for Vec<T> {
    fn visit_properties_mut(&mut self, visitor: &mut dyn FnMut(PropertyMut<'_>)) {
        for value in self.iter_mut() {
            value.visit_properties_mut(visitor);
        }
    }
}
macro_rules! properties {
    ($ty:ty { $($($member:ident).+),* $(,)? }) => {
        impl MaterialProperties for $ty {
            fn visit_properties_mut(&mut self, visitor: &mut dyn FnMut(PropertyMut<'_>)) {
                $(self.$($member).+.visit_properties_mut(visitor);)*
                let _ = visitor;
            }
        }
    };
}
properties! { ReflectionMaterialStep {
    reflection.parameters, displacement.parameters,
    reflection_offset, blur_angle, blur_distance_max,
    reflection.input, displacement.input, blur_map,
}}
properties! { StandardMaterialStep {
    diffuse, decal_layer, specular.input.color,
    specular.input.gloss, emissive.input.primary.input,
    emissive.input.secondary.input, environment_layer,
    environment_mask_layer, opacity.primary, opacity.secondary,
    normals.base, height_layer, light_map_layer,
    ambient_occlusion_layer, normals.primary.mask,
    normals.secondary.mask, normals.primary.input,
    normals.secondary.input, parallax_height,
    motion_blur_amount, normals.weights,
}}
properties! { DataDrivenMaterialStep<'_> {} }
properties! { LensFlareMaterialStep<'_> {
    flare_map, mask_map, intensity, color, hdr, size,
}}
properties! { DisplacementMaterialStep { displacement.parameters, displacement.input, strength_map, } }
properties! { TerrainMaterialStep { terrain_map, } }
properties! { CreepMaterialStep { mask_map, } }
properties! { SplatTerrainBakeMaterialStep { diffuse_map, normal_map, specular_map, } }
properties! { HairMaterialStep {
    diffuse.input, layer_spec_shift, layer_spec_noise,
    layer_ambient_occlusion, diffuse.parameters, color_specular,
}}
properties! { CompositeSection { map_multiplier, } }
properties! { CompositeMaterialStep { sections, } }
properties! { TextureLayer {
    binding.contribution.color, binding.contribution.rgb_multiply,
    binding.contribution.rgb_add, binding.sampling.avi_play,
    binding.sampling.avi_restart, binding.sampling.current_frame,
    binding.coordinates.transform.uv_offset, binding.coordinates.transform.uv_angle,
    binding.coordinates.transform.uv_tiling, binding.coordinates.transform.w_offset,
    binding.coordinates.transform.w_tiling, binding.contribution.map_alpha,
    binding.coordinates.transform.triplanar_offset, binding.coordinates.transform.triplanar_scale,
}}
properties! { VolumeMaterialStep { density, color_map, noise_map1, noise_map2, } }
properties! { VolumeNoiseMaterialStep {
    density, near_plane, falloff, color_map,
    noise_map1, noise_map2, scroll_rate, position,
    scale, rotation,
}}

macro_rules! material_variants {
    ($($variant:ident),* $(,)?) => {
        impl MaterialProperties for MaterialStep<'_> {
            fn visit_properties_mut(&mut self, visitor: &mut dyn FnMut(PropertyMut<'_>)) {
                match self { $(Self::$variant(value) => value.visit_properties_mut(visitor),)* }
            }
        }
    };
}
material_variants! { Standard, Displacement, Composite, Terrain, Volume, VolumeNoise, Hair, Creep, SplatTerrainBake, Reflection, LensFlare, DataDriven }
