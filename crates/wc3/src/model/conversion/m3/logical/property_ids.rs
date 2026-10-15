//! Read-only discovery of native property IDs already authored in graph controls.
use super::{
    bindings::{Bindings, Value},
    Graph,
};
use crate::model::semantic::profiles::m3 as surface;
trait Collect {
    fn collect(&self, bindings: &mut Bindings);
}
impl<T: Value + Clone> Collect for surface::Property<T> {
    fn collect(&self, bindings: &mut Bindings) {
        bindings.entries.entry(self.control.source_id).or_default();
        if let Some(target) = &self.binding {
            bindings.register(self.control.source_id, target.clone());
            bindings.register_mode(target, self.control.interpolation);
        }
    }
}
impl<T: Value + Clone> Collect for surface::PropertyControl<T> {
    fn collect(&self, bindings: &mut Bindings) {
        bindings.entries.entry(self.control.source_id).or_default();
        if let Some(target) = &self.binding {
            bindings.register(self.control.source_id, target.clone());
            bindings.register_mode(target, self.control.interpolation);
        }
    }
}
impl Collect for surface::Projector {
    fn collect(&self, bindings: &mut Bindings) {
        self.offset.collect(bindings);
        self.pitch.collect(bindings);
        self.yaw.collect(bindings);
        self.roll.collect(bindings);
        self.field_of_view.collect(bindings);
        self.aspect_ratio.collect(bindings);
        self.near.collect(bindings);
        self.far.collect(bindings);
        self.box_offset_z_bottom.collect(bindings);
        self.box_offset_z_top.collect(bindings);
        self.box_offset_x_left.collect(bindings);
        self.box_offset_x_right.collect(bindings);
        self.box_offset_y_front.collect(bindings);
        self.box_offset_y_back.collect(bindings);
        self.active.collect(bindings);
    }
}
impl Collect for surface::Force {
    fn collect(&self, bindings: &mut Bindings) {
        self.strength.collect(bindings);
        self.width.collect(bindings);
        self.height.collect(bindings);
        self.length.collect(bindings);
    }
}
impl Collect for surface::Warp {
    fn collect(&self, bindings: &mut Bindings) {
        self.radius.collect(bindings);
        self.height.collect(bindings);
        self.strength.collect(bindings);
        self.angular.collect(bindings);
        self.axial.collect(bindings);
        self.radial.collect(bindings);
    }
}
impl Collect for surface::Light {
    fn collect(&self, bindings: &mut Bindings) {
        self.diffuse_color.collect(bindings);
        self.intensity_multiplier.collect(bindings);
        self.specular_color.collect(bindings);
        self.specular_multiplier.collect(bindings);
        self.decay.collect(bindings);
        self.attenuation_start.collect(bindings);
        self.hot_spot.collect(bindings);
        self.falloff.collect(bindings);
    }
}
impl<'a> Collect for surface::Camera<'a> {
    fn collect(&self, bindings: &mut Bindings) {
        self.optics.collect(bindings);
        self.field_of_view.collect(bindings);
        self.shadow_clip_distance.collect(bindings);
        self.focus_distance.collect(bindings);
        self.far_focus_range.collect(bindings);
        self.near_focus_range.collect(bindings);
        self.dof_amount.collect(bindings);
    }
}
impl Collect for surface::CameraClipping {
    fn collect(&self, bindings: &mut Bindings) {
        self.far.collect(bindings);
        self.near.collect(bindings);
    }
}
impl Collect for surface::CameraFalloff {
    fn collect(&self, bindings: &mut Bindings) {
        self.start.collect(bindings);
        self.end.collect(bindings);
    }
}
impl Collect for surface::CameraBokeh {
    fn collect(&self, bindings: &mut Bindings) {
        self.f_stop.collect(bindings);
        self.maximum_circle.collect(bindings);
    }
}
impl Collect for surface::ShadowBox {
    fn collect(&self, bindings: &mut Bindings) {
        self.length.collect(bindings);
        self.width.collect(bindings);
        self.height.collect(bindings);
    }
}
impl Collect for surface::ViewVolume {
    fn collect(&self, bindings: &mut Bindings) {
        self.size.collect(bindings);
    }
}
impl Collect for surface::BoneControls {
    fn collect(&self, bindings: &mut Bindings) {
        self.position.collect(bindings);
        self.rotation.collect(bindings);
        self.scale.collect(bindings);
        self.batching.collect(bindings);
    }
}
impl Collect for surface::VariationControl {
    fn collect(&self, bindings: &mut Bindings) {
        self.amplitude.collect(bindings);
        self.frequency.collect(bindings);
    }
}
impl<'a> Collect for surface::Particle<'a> {
    fn collect(&self, bindings: &mut Bindings) {
        self.motion.speed.collect(bindings);
        self.motion.variation.collect(bindings);
        self.details.direction.initial_yaw.collect(bindings);
        self.details.direction.initial_pitch.collect(bindings);
        self.details.direction.initial_horizontal.collect(bindings);
        self.details.direction.initial_vertical.collect(bindings);
        self.lifetime.duration.collect(bindings);
        self.lifetime.variation.collect(bindings);
        self.appearance.size_animation.collect(bindings);
        self.appearance.rotation_animation.collect(bindings);
        self.appearance.color_start.collect(bindings);
        self.appearance.color_mid.collect(bindings);
        self.appearance.color_end.collect(bindings);
        self.emission.rate.collect(bindings);
        self.details.shape_outer.collect(bindings);
        self.details.shape_inner.collect(bindings);
        self.details.outer_radius.collect(bindings);
        self.details.inner_radius.collect(bindings);
        self.details.size_random_animation.collect(bindings);
        self.details.rotation_random_animation.collect(bindings);
        self.details.color_start_random.collect(bindings);
        self.details.color_mid_random.collect(bindings);
        self.details.color_end_random.collect(bindings);
        self.emission.trigger.collect(bindings);
        self.details.pitch.collect(bindings);
        self.details.yaw.collect(bindings);
        self.details.speed.collect(bindings);
        self.details.size.collect(bindings);
        self.details.alpha.collect(bindings);
        self.details.color.collect(bindings);
        self.details.rotation.collect(bindings);
        self.details.horizontal.collect(bindings);
        self.details.vertical.collect(bindings);
        self.details.particle_velocity.collect(bindings);
        self.details.phase_shift.collect(bindings);
        self.details.alpha_threshold.collect(bindings);
        self.details.uv_offset.collect(bindings);
        self.details.uv_angle.collect(bindings);
        self.details.uv_tiling.collect(bindings);
        for value in self.details.spline_points.iter() {
            value.collect(bindings);
        }
        self.details.lower_bound.collect(bindings);
        self.details.upper_bound.collect(bindings);
        self.details.trail_emission_rate.collect(bindings);
    }
}
impl Collect for surface::SplinePoint {
    fn collect(&self, bindings: &mut Bindings) {
        self.velocity.collect(bindings);
        self.velocity_base_factor.collect(bindings);
        self.velocity_end_factor.collect(bindings);
        self.yaw_variation.collect(bindings);
        self.pitch_variation.collect(bindings);
        self.velocity_variation.collect(bindings);
        self.yaw.collect(bindings);
        self.pitch.collect(bindings);
    }
}
impl<'a> Collect for surface::Ribbon<'a> {
    fn collect(&self, bindings: &mut Bindings) {
        self.motion.speed.collect(bindings);
        self.motion.variation.collect(bindings);
        self.direction.initial_yaw.collect(bindings);
        self.direction.initial_pitch.collect(bindings);
        self.direction.initial_horizontal.collect(bindings);
        self.direction.initial_vertical.collect(bindings);
        self.lifetime.duration.collect(bindings);
        self.lifetime.variation.collect(bindings);
        self.appearance.size_animation.collect(bindings);
        self.appearance.rotation_animation.collect(bindings);
        self.appearance.color_start.collect(bindings);
        self.appearance.color_mid.collect(bindings);
        self.appearance.color_end.collect(bindings);
        self.max_length.collect(bindings);
        for value in self.spline_points.iter() {
            value.collect(bindings);
        }
        self.active.collect(bindings);
        self.yaw.collect(bindings);
        self.pitch.collect(bindings);
        self.speed.collect(bindings);
        self.size.collect(bindings);
        self.alpha.collect(bindings);
        self.particle_velocity.collect(bindings);
        self.overlay.collect(bindings);
    }
}
impl<'a> Collect for surface::Body<'a> {
    fn collect(&self, bindings: &mut Bindings) {
        if let Some(value) = &self.dynamic {
            value.state.collect(bindings);
        }
    }
}
impl<'a> Collect for surface::Cloth<'a> {
    fn collect(&self, bindings: &mut Bindings) {
        if let Some(active) = &self.active {
            active.collect(bindings);
        }
    }
}
impl Collect for surface::Reflection {
    fn collect(&self, bindings: &mut Bindings) {
        self.reflection_strength.collect(bindings);
        self.displacement_strength.collect(bindings);
        self.reflection_offset.collect(bindings);
        self.blur_angle.collect(bindings);
        self.blur_distance_max.collect(bindings);
        if let Some(value) = &self.reflection_map {
            value.collect(bindings);
        }
        if let Some(value) = &self.displacement_map {
            value.collect(bindings);
        }
        if let Some(value) = &self.blur_map {
            value.collect(bindings);
        }
    }
}
impl Collect for surface::Standard {
    fn collect(&self, bindings: &mut Bindings) {
        if let Some(value) = &self.diffuse_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.decal_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.specular_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.gloss_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.emissive_layer1 {
            value.collect(bindings);
        }
        if let Some(value) = &self.emissive_layer2 {
            value.collect(bindings);
        }
        if let Some(value) = &self.environment_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.environment_mask_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.alpha_layer1 {
            value.collect(bindings);
        }
        if let Some(value) = &self.alpha_layer2 {
            value.collect(bindings);
        }
        if let Some(value) = &self.normal_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.height_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.light_map_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.ambient_occlusion_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.normal_blend1_mask_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.normal_blend2_mask_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.normal_blend1_layer {
            value.collect(bindings);
        }
        if let Some(value) = &self.normal_blend2_layer {
            value.collect(bindings);
        }
        self.parallax_height.collect(bindings);
        self.motion_blur_amount.collect(bindings);
        for value in self.normal_blend_factors.iter() {
            value.collect(bindings);
        }
    }
}
impl<'a> Collect for surface::LensFlare<'a> {
    fn collect(&self, bindings: &mut Bindings) {
        if let Some(value) = &self.flare_map {
            value.collect(bindings);
        }
        if let Some(value) = &self.mask_map {
            value.collect(bindings);
        }
        self.intensity.collect(bindings);
        self.color.collect(bindings);
        self.hdr.collect(bindings);
        self.size.collect(bindings);
    }
}
impl Collect for surface::Displacement {
    fn collect(&self, bindings: &mut Bindings) {
        self.strength.collect(bindings);
        if let Some(value) = &self.normal_map {
            value.collect(bindings);
        }
        if let Some(value) = &self.strength_map {
            value.collect(bindings);
        }
    }
}
impl Collect for surface::Terrain {
    fn collect(&self, bindings: &mut Bindings) {
        if let Some(value) = &self.terrain_map {
            value.collect(bindings);
        }
    }
}
impl Collect for surface::Creep {
    fn collect(&self, bindings: &mut Bindings) {
        if let Some(value) = &self.mask_map {
            value.collect(bindings);
        }
    }
}
impl Collect for surface::SplatTerrainBake {
    fn collect(&self, bindings: &mut Bindings) {
        if let Some(value) = &self.diffuse_map {
            value.collect(bindings);
        }
        if let Some(value) = &self.normal_map {
            value.collect(bindings);
        }
        if let Some(value) = &self.specular_map {
            value.collect(bindings);
        }
    }
}
impl Collect for surface::Hair {
    fn collect(&self, bindings: &mut Bindings) {
        if let Some(value) = &self.layer_base {
            value.collect(bindings);
        }
        if let Some(value) = &self.layer_spec_shift {
            value.collect(bindings);
        }
        if let Some(value) = &self.layer_spec_noise {
            value.collect(bindings);
        }
        if let Some(value) = &self.layer_ambient_occlusion {
            value.collect(bindings);
        }
        self.color_diffuse.collect(bindings);
        self.color_specular.collect(bindings);
    }
}
impl Collect for surface::CompositeSection {
    fn collect(&self, bindings: &mut Bindings) {
        self.map_multiplier.collect(bindings);
    }
}
impl Collect for surface::Composite {
    fn collect(&self, bindings: &mut Bindings) {
        for value in self.sections.iter() {
            value.collect(bindings);
        }
    }
}
impl Collect for surface::TextureLayer {
    fn collect(&self, bindings: &mut Bindings) {
        self.color.collect(bindings);
        self.rgb_multiply.collect(bindings);
        self.rgb_add.collect(bindings);
        self.avi_play.collect(bindings);
        self.avi_restart.collect(bindings);
        self.current_frame.collect(bindings);
        self.uv_offset.collect(bindings);
        self.uv_angle.collect(bindings);
        self.uv_tiling.collect(bindings);
        self.w_offset.collect(bindings);
        self.w_tiling.collect(bindings);
        self.map_alpha.collect(bindings);
        self.triplanar_offset.collect(bindings);
        self.triplanar_scale.collect(bindings);
    }
}
impl Collect for surface::Volume {
    fn collect(&self, bindings: &mut Bindings) {
        self.density.collect(bindings);
        if let Some(value) = &self.color_map {
            value.collect(bindings);
        }
        if let Some(value) = &self.noise_map1 {
            value.collect(bindings);
        }
        if let Some(value) = &self.noise_map2 {
            value.collect(bindings);
        }
    }
}
impl Collect for surface::VolumeNoise {
    fn collect(&self, bindings: &mut Bindings) {
        self.density.collect(bindings);
        self.near_plane.collect(bindings);
        self.falloff.collect(bindings);
        if let Some(value) = &self.color_map {
            value.collect(bindings);
        }
        if let Some(value) = &self.noise_map1 {
            value.collect(bindings);
        }
        if let Some(value) = &self.noise_map2 {
            value.collect(bindings);
        }
        self.scroll_rate.collect(bindings);
        self.position.collect(bindings);
        self.scale.collect(bindings);
        self.rotation.collect(bindings);
    }
}
impl Collect for surface::Material<'_> {
    fn collect(&self, bindings: &mut Bindings) {
        match self {
            Self::Standard(value) => value.collect(bindings),
            Self::Displacement(value) => value.collect(bindings),
            Self::Composite(value) => value.collect(bindings),
            Self::Terrain(value) => value.collect(bindings),
            Self::Volume(value) => value.collect(bindings),
            Self::VolumeNoise(value) => value.collect(bindings),
            Self::Creep(value) => value.collect(bindings),
            Self::Hair(value) => value.collect(bindings),
            Self::SplatTerrainBake(value) => value.collect(bindings),
            Self::Reflection(value) => value.collect(bindings),
            Self::LensFlare(value) => value.collect(bindings),
            Self::DataDriven(_) => {}
        }
    }
}
impl Collect for surface::Feature<'_> {
    fn collect(&self, bindings: &mut Bindings) {
        match self {
            Self::Particle(value) => value.collect(bindings),
            Self::ParticleCopy(value) => {
                value.emission_rate.collect(bindings);
                value.squirt_amount.collect(bindings);
            }
            Self::Ribbon(value) => value.collect(bindings),
            Self::Light(value) => value.collect(bindings),
            Self::Camera(value) => value.collect(bindings),
            Self::Projector(value) => value.collect(bindings),
            Self::Force(value) => value.collect(bindings),
            Self::Warp(value) => value.collect(bindings),
            Self::Attachment(_) => {}
            Self::HitTest(_) => {}
            Self::AttachmentVolume(_) => {}
            Self::Billboard(_) => {}
            Self::Turret(_) => {}
            Self::IkJoint(_) => {}
            Self::IkTwoJoint(_) => {}
            Self::IkCcd(_) => {}
            Self::OneBoneSolver(_) => {}
            Self::ShadowBox(value) => value.collect(bindings),
            Self::ViewVolume(value) => value.collect(bindings),
            Self::Event(_) => {}
            Self::Extent(value) => {
                bindings.entries.entry(value.control.source_id).or_default();
                bindings.register(value.control.source_id, value.minimum.clone());
                bindings.register(value.control.source_id, value.maximum.clone());
                bindings.register(value.control.source_id, value.radius.clone());
                for target in [
                    value.minimum.target(),
                    value.maximum.target(),
                    value.radius.target(),
                ] {
                    bindings
                        .modes
                        .insert(target.clone(), value.control.interpolation);
                }
            }

            Self::GeometryAnimation(unavailable)
            | Self::CollisionShape(unavailable)
            | Self::ParticleProgram(unavailable)
            | Self::FaceActor(unavailable)
            | Self::LightProgram(unavailable)
            | Self::AtlasParticles(unavailable)
            | Self::PointLight(unavailable) => match *unavailable {},
        }
    }
}
pub(super) fn collect(model: &Graph<'_>) -> Bindings {
    let mut result = Bindings::default();
    for (_, material) in model.materials() {
        material.surface.collect(&mut result);
    }
    for (_, feature) in model.features() {
        feature.payload.collect(&mut result);
    }
    for (_, node) in model.nodes() {
        if let surface::NodeRole::Bone(controls) = &node.controls {
            controls.collect(&mut result);
        }
    }
    for (_, system) in model.systems() {
        match system {
            surface::System::Physics(system) => {
                for body in &system.bodies {
                    body.collect(&mut result);
                }
            }
            surface::System::Cloth(system) => {
                for cloth in &system.cloths {
                    cloth.collect(&mut result);
                }
            }
            surface::System::AnimationPartitions(_)
            | surface::System::Collision(_)
            | surface::System::TurretGroups(_)
            | surface::System::ControllerSelections(_) => {}
        }
    }
    result
}

impl Collect for surface::CameraOptics {
    fn collect(&self, bindings: &mut Bindings) {
        match self {
            Self::Legacy { .. } => {}
            Self::Clipping(clipping) => clipping.collect(bindings),
            Self::Falloff { clipping, falloff } => {
                clipping.collect(bindings);
                falloff.collect(bindings);
            }
            Self::Bokeh {
                clipping,
                falloff,
                bokeh,
            } => {
                clipping.collect(bindings);
                falloff.collect(bindings);
                bokeh.collect(bindings);
            }
        }
    }
}
