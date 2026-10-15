//! Logical material properties bind to typed graph destinations, never section addresses.
use crate::model::semantic::profiles::m3::properties::Target as PropertyField;
use crate::model::semantic::profiles::m3::properties::{Field, FieldPath};
use crate::model::semantic::profiles::UniversalProperty;
use crate::model::semantic::{
    profiles::m3 as surface, OwnerId, Property as Destination, PropertyTarget, Quaternion,
};
use std::collections::{BTreeMap, HashMap};
#[derive(Clone, Debug)]
pub(super) enum Target {
    Scalar(Destination<f32>),
    Vector2(Destination<[f32; 2]>),
    Vector3(Destination<[f32; 3]>),
    Vector4(Destination<[f32; 4]>),
    Unsigned(Destination<u32>),
    Unsigned16(Destination<u16>),
    Rotation(Destination<Quaternion>),
    Signed(Destination<i32>),
    Byte(Destination<u8>),
    Bytes4(Destination<[u8; 4]>),
    Unsigned4(Destination<[u32; 4]>),
}
#[derive(Default)]
pub(super) struct Bindings {
    pub entries: BTreeMap<u32, Vec<Target>>,
    pub modes: HashMap<PropertyTarget, surface::values::Interpolation>,
}
pub(super) trait Value {
    fn target(value: Destination<Self>) -> Target
    where
        Self: Sized;
}
impl Value for f32 {
    fn target(value: Destination<Self>) -> Target {
        Target::Scalar(value)
    }
}
impl Value for [f32; 2] {
    fn target(value: Destination<Self>) -> Target {
        Target::Vector2(value)
    }
}
impl Value for [f32; 3] {
    fn target(value: Destination<Self>) -> Target {
        Target::Vector3(value)
    }
}
impl Value for [f32; 4] {
    fn target(value: Destination<Self>) -> Target {
        Target::Vector4(value)
    }
}
impl Value for u32 {
    fn target(value: Destination<Self>) -> Target {
        Target::Unsigned(value)
    }
}
impl Value for u16 {
    fn target(value: Destination<Self>) -> Target {
        Target::Unsigned16(value)
    }
}
impl Value for Quaternion {
    fn target(value: Destination<Self>) -> Target {
        Target::Rotation(value)
    }
}
impl Value for i32 {
    fn target(value: Destination<Self>) -> Target {
        Target::Signed(value)
    }
}
impl Value for u8 {
    fn target(value: Destination<Self>) -> Target {
        Target::Byte(value)
    }
}
impl Value for [u8; 4] {
    fn target(value: Destination<Self>) -> Target {
        Target::Bytes4(value)
    }
}
impl Value for [u32; 4] {
    fn target(value: Destination<Self>) -> Target {
        Target::Unsigned4(value)
    }
}
impl Bindings {
    pub fn register_mode<T>(
        &mut self,
        destination: &Destination<T>,
        interpolation: surface::values::Interpolation,
    ) {
        self.modes
            .insert(destination.target().clone(), interpolation);
    }
    pub fn register<T: Value>(&mut self, source_id: u32, destination: Destination<T>) {
        self.entries
            .entry(source_id)
            .or_default()
            .push(T::target(destination));
    }
}
trait Bind {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings);
}
impl<T: Value> Bind for surface::Property<T> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        let destination = Destination::new(PropertyTarget::Profile {
            owner,
            key: UniversalProperty::M3(PropertyField::Field(path.clone())),
        });
        self.binding = Some(destination.clone());
        bindings.register_mode(&destination, self.control.interpolation);
        bindings
            .entries
            .entry(self.control.source_id)
            .or_default()
            .push(T::target(destination));
    }
}
impl Bind for surface::Reflection {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        self.reflection_strength
            .bind(owner, &path.child(Field::ReflectionStrength), bindings);
        self.displacement_strength
            .bind(owner, &path.child(Field::DisplacementStrength), bindings);
        self.reflection_offset
            .bind(owner, &path.child(Field::ReflectionOffset), bindings);
        self.blur_angle
            .bind(owner, &path.child(Field::BlurAngle), bindings);
        self.blur_distance_max
            .bind(owner, &path.child(Field::BlurDistanceMax), bindings);
        if let Some(value) = &mut self.reflection_map {
            value.bind(owner, &path.child(Field::ReflectionMap), bindings);
        }
        if let Some(value) = &mut self.displacement_map {
            value.bind(owner, &path.child(Field::DisplacementMap), bindings);
        }
        if let Some(value) = &mut self.blur_map {
            value.bind(owner, &path.child(Field::BlurMap), bindings);
        }
    }
}
impl Bind for surface::Standard {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(value) = &mut self.diffuse_layer {
            value.bind(owner, &path.child(Field::DiffuseLayer), bindings);
        }
        if let Some(value) = &mut self.decal_layer {
            value.bind(owner, &path.child(Field::DecalLayer), bindings);
        }
        if let Some(value) = &mut self.specular_layer {
            value.bind(owner, &path.child(Field::SpecularLayer), bindings);
        }
        if let Some(value) = &mut self.gloss_layer {
            value.bind(owner, &path.child(Field::GlossLayer), bindings);
        }
        if let Some(value) = &mut self.emissive_layer1 {
            value.bind(owner, &path.child(Field::EmissiveLayer1), bindings);
        }
        if let Some(value) = &mut self.emissive_layer2 {
            value.bind(owner, &path.child(Field::EmissiveLayer2), bindings);
        }
        if let Some(value) = &mut self.environment_layer {
            value.bind(owner, &path.child(Field::EnvironmentLayer), bindings);
        }
        if let Some(value) = &mut self.environment_mask_layer {
            value.bind(owner, &path.child(Field::EnvironmentMaskLayer), bindings);
        }
        if let Some(value) = &mut self.alpha_layer1 {
            value.bind(owner, &path.child(Field::AlphaLayer1), bindings);
        }
        if let Some(value) = &mut self.alpha_layer2 {
            value.bind(owner, &path.child(Field::AlphaLayer2), bindings);
        }
        if let Some(value) = &mut self.normal_layer {
            value.bind(owner, &path.child(Field::NormalLayer), bindings);
        }
        if let Some(value) = &mut self.height_layer {
            value.bind(owner, &path.child(Field::HeightLayer), bindings);
        }
        if let Some(value) = &mut self.light_map_layer {
            value.bind(owner, &path.child(Field::LightMapLayer), bindings);
        }
        if let Some(value) = &mut self.ambient_occlusion_layer {
            value.bind(owner, &path.child(Field::AmbientOcclusionLayer), bindings);
        }
        if let Some(value) = &mut self.normal_blend1_mask_layer {
            value.bind(owner, &path.child(Field::NormalBlend1MaskLayer), bindings);
        }
        if let Some(value) = &mut self.normal_blend2_mask_layer {
            value.bind(owner, &path.child(Field::NormalBlend2MaskLayer), bindings);
        }
        if let Some(value) = &mut self.normal_blend1_layer {
            value.bind(owner, &path.child(Field::NormalBlend1Layer), bindings);
        }
        if let Some(value) = &mut self.normal_blend2_layer {
            value.bind(owner, &path.child(Field::NormalBlend2Layer), bindings);
        }
        self.parallax_height
            .bind(owner, &path.child(Field::ParallaxHeight), bindings);
        self.motion_blur_amount
            .bind(owner, &path.child(Field::MotionBlurAmount), bindings);
        for (index, value) in self.normal_blend_factors.iter_mut().enumerate() {
            value.bind(
                owner,
                &path.child(Field::NormalBlendFactors).indexed(index),
                bindings,
            );
        }
    }
}
impl<'a> Bind for surface::DataDriven<'a> {
    fn bind(&mut self, _owner: OwnerId, _path: &FieldPath, _bindings: &mut Bindings) {}
}
impl<'a> Bind for surface::DataProperty<'a> {
    fn bind(&mut self, _owner: OwnerId, _path: &FieldPath, _bindings: &mut Bindings) {}
}
impl<'a> Bind for surface::DataGroup<'a> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        for (index, value) in self.properties.iter_mut().enumerate() {
            value.bind(
                owner,
                &path.child(Field::Properties).indexed(index),
                bindings,
            );
        }
    }
}
impl<'a> Bind for surface::DataProperties<'a> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        for (index, value) in self.groups.iter_mut().enumerate() {
            value.bind(owner, &path.child(Field::Groups).indexed(index), bindings);
        }
    }
}
impl Bind for surface::SubFlare {
    fn bind(&mut self, _owner: OwnerId, _path: &FieldPath, _bindings: &mut Bindings) {}
}
impl<'a> Bind for surface::LensFlare<'a> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(value) = &mut self.flare_map {
            value.bind(owner, &path.child(Field::FlareMap), bindings);
        }
        if let Some(value) = &mut self.mask_map {
            value.bind(owner, &path.child(Field::MaskMap), bindings);
        }
        for (index, value) in self.sub_flares.iter_mut().enumerate() {
            value.bind(
                owner,
                &path.child(Field::SubFlares).indexed(index),
                bindings,
            );
        }
        self.intensity
            .bind(owner, &path.child(Field::Intensity), bindings);
        self.color.bind(owner, &path.child(Field::Color), bindings);
        self.hdr.bind(owner, &path.child(Field::Hdr), bindings);
        self.size.bind(owner, &path.child(Field::Size), bindings);
    }
}
impl Bind for surface::Displacement {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        self.strength
            .bind(owner, &path.child(Field::Strength), bindings);
        if let Some(value) = &mut self.normal_map {
            value.bind(owner, &path.child(Field::NormalMap), bindings);
        }
        if let Some(value) = &mut self.strength_map {
            value.bind(owner, &path.child(Field::StrengthMap), bindings);
        }
    }
}
impl Bind for surface::Terrain {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(value) = &mut self.terrain_map {
            value.bind(owner, &path.child(Field::TerrainMap), bindings);
        }
    }
}
impl Bind for surface::Creep {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(value) = &mut self.mask_map {
            value.bind(owner, &path.child(Field::MaskMap), bindings);
        }
    }
}
impl Bind for surface::SplatTerrainBake {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(value) = &mut self.diffuse_map {
            value.bind(owner, &path.child(Field::DiffuseMap), bindings);
        }
        if let Some(value) = &mut self.normal_map {
            value.bind(owner, &path.child(Field::NormalMap), bindings);
        }
        if let Some(value) = &mut self.specular_map {
            value.bind(owner, &path.child(Field::SpecularMap), bindings);
        }
    }
}
impl Bind for surface::Hair {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(value) = &mut self.layer_base {
            value.bind(owner, &path.child(Field::LayerBase), bindings);
        }
        if let Some(value) = &mut self.layer_spec_shift {
            value.bind(owner, &path.child(Field::LayerSpecShift), bindings);
        }
        if let Some(value) = &mut self.layer_spec_noise {
            value.bind(owner, &path.child(Field::LayerSpecNoise), bindings);
        }
        if let Some(value) = &mut self.layer_ambient_occlusion {
            value.bind(owner, &path.child(Field::LayerAmbientOcclusion), bindings);
        }
        self.color_diffuse
            .bind(owner, &path.child(Field::ColorDiffuse), bindings);
        self.color_specular
            .bind(owner, &path.child(Field::ColorSpecular), bindings);
    }
}
impl Bind for surface::CompositeSection {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        self.map_multiplier
            .bind(owner, &path.child(Field::MapMultiplier), bindings);
    }
}
impl Bind for surface::Composite {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        for (index, value) in self.sections.iter_mut().enumerate() {
            value.bind(owner, &path.child(Field::Sections).indexed(index), bindings);
        }
    }
}
impl Bind for surface::TextureLayer {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        self.color.bind(owner, &path.child(Field::Color), bindings);
        self.rgb_multiply
            .bind(owner, &path.child(Field::RgbMultiply), bindings);
        self.rgb_add
            .bind(owner, &path.child(Field::RgbAdd), bindings);
        self.avi_play
            .bind(owner, &path.child(Field::AviPlay), bindings);
        self.avi_restart
            .bind(owner, &path.child(Field::AviRestart), bindings);
        self.current_frame
            .bind(owner, &path.child(Field::CurrentFrame), bindings);
        self.uv_offset
            .bind(owner, &path.child(Field::UvOffset), bindings);
        self.uv_angle
            .bind(owner, &path.child(Field::UvAngle), bindings);
        self.uv_tiling
            .bind(owner, &path.child(Field::UvTiling), bindings);
        self.w_offset
            .bind(owner, &path.child(Field::WOffset), bindings);
        self.w_tiling
            .bind(owner, &path.child(Field::WTiling), bindings);
        self.map_alpha
            .bind(owner, &path.child(Field::MapAlpha), bindings);
        self.triplanar_offset
            .bind(owner, &path.child(Field::TriplanarOffset), bindings);
        self.triplanar_scale
            .bind(owner, &path.child(Field::TriplanarScale), bindings);
    }
}
impl Bind for surface::Volume {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        self.density
            .bind(owner, &path.child(Field::Density), bindings);
        if let Some(value) = &mut self.color_map {
            value.bind(owner, &path.child(Field::ColorMap), bindings);
        }
        if let Some(value) = &mut self.noise_map1 {
            value.bind(owner, &path.child(Field::NoiseMap1), bindings);
        }
        if let Some(value) = &mut self.noise_map2 {
            value.bind(owner, &path.child(Field::NoiseMap2), bindings);
        }
    }
}
impl Bind for surface::VolumeNoise {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        self.density
            .bind(owner, &path.child(Field::Density), bindings);
        self.near_plane
            .bind(owner, &path.child(Field::NearPlane), bindings);
        self.falloff
            .bind(owner, &path.child(Field::Falloff), bindings);
        if let Some(value) = &mut self.color_map {
            value.bind(owner, &path.child(Field::ColorMap), bindings);
        }
        if let Some(value) = &mut self.noise_map1 {
            value.bind(owner, &path.child(Field::NoiseMap1), bindings);
        }
        if let Some(value) = &mut self.noise_map2 {
            value.bind(owner, &path.child(Field::NoiseMap2), bindings);
        }
        self.scroll_rate
            .bind(owner, &path.child(Field::ScrollRate), bindings);
        self.position
            .bind(owner, &path.child(Field::Position), bindings);
        self.scale.bind(owner, &path.child(Field::Scale), bindings);
        self.rotation
            .bind(owner, &path.child(Field::Rotation), bindings);
    }
}
pub(super) fn material(value: &mut surface::Material<'_>, owner: OwnerId, bindings: &mut Bindings) {
    match value {
        surface::Material::Standard(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::Displacement(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::Composite(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::Terrain(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::Volume(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::VolumeNoise(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::Creep(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::Hair(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::SplatTerrainBake(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::Reflection(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::LensFlare(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
        surface::Material::DataDriven(value) => {
            value.bind(owner, &FieldPath::root(Field::Material), bindings)
        }
    }
}
pub(super) fn field<T: Value>(
    value: &mut surface::Property<T>,
    owner: OwnerId,
    path: &FieldPath,
    bindings: &mut Bindings,
) {
    value.bind(owner, path, bindings)
}

pub(super) use super::property_ids::collect;

impl Bind for surface::Body<'_> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(dynamic) = &mut self.dynamic {
            dynamic
                .state
                .bind(owner, &path.child(Field::State), bindings);
        }
    }
}
impl Bind for surface::Cloth<'_> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(active) = &mut self.active {
            active.bind(owner, &path.child(Field::Active), bindings);
        }
    }
}
impl Bind for surface::System<'_> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        match self {
            surface::System::AnimationPartitions(_)
            | surface::System::Collision(_)
            | surface::System::TurretGroups(_)
            | surface::System::ControllerSelections(_) => {}
            surface::System::Physics(system) => {
                for (index, body) in system.bodies.iter_mut().enumerate() {
                    body.bind(owner, &path.child(Field::Body).indexed(index), bindings);
                }
            }
            surface::System::Cloth(system) => {
                for (index, cloth) in system.cloths.iter_mut().enumerate() {
                    cloth.bind(owner, &path.child(Field::Cloth).indexed(index), bindings);
                }
            }
        }
    }
}

pub(super) fn system(value: &mut surface::System<'_>, owner: OwnerId, bindings: &mut Bindings) {
    value.bind(owner, &FieldPath::root(Field::Physics), bindings);
}
