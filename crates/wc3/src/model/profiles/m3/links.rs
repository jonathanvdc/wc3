//! Traversal of all graph links held by M3 profile values.
use super::{
    AnimationControl, AttachmentVolume, AuxiliaryGeometry, BillboardBehavior, Body, BodyParameters,
    BoneControls, Camera, CameraBokeh, CameraClipping, CameraFalloff, Cloth, ClothCollider,
    ClothProxy, Composite, CompositeSection, Creep, DataDriven, DataProperties, Displacement,
    DynamicState, ExternalModel, Feature, Force, Hair, HitTestShape, IkCcd, IkJoint, IkTwoJoint,
    Joint, LensFlare, Light, Material, ModelMetadata, NodeRole, OneBoneSolver, Parameters,
    Projector, Property, PropertyControl, Reflection, ShadowBox, Shape, ShapeGeometry,
    SplatTerrainBake, SplinePoint, Standard, SubFlare, Terrain, TextureLayer, TurretBehavior,
    VariationControl, ViewVolume, Volume, VolumeNoise, Warp,
};
use crate::model::{CurveValue, FeatureKind, GeometryKind, Link, Quaternion, ValidateLinks};

impl ValidateLinks for Projector {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Material(self.material));
        self.offset.visit_links(visitor);
        self.pitch.visit_links(visitor);
        self.yaw.visit_links(visitor);
        self.roll.visit_links(visitor);
        self.field_of_view.visit_links(visitor);
        self.aspect_ratio.visit_links(visitor);
        self.near.visit_links(visitor);
        self.far.visit_links(visitor);
        self.box_offset_z_bottom.visit_links(visitor);
        self.box_offset_z_top.visit_links(visitor);
        self.box_offset_x_left.visit_links(visitor);
        self.box_offset_x_right.visit_links(visitor);
        self.box_offset_y_front.visit_links(visitor);
        self.box_offset_y_back.visit_links(visitor);
        if !self.falloff.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.alpha_init.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.alpha_mid.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.alpha_end.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.lifetime_attack.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.lifetime_attack_to.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.lifetime_hold.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.lifetime_hold_to.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.lifetime_decay.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.lifetime_decay_to.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.attenuation_distance.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.active.visit_links(visitor);
    }
}
impl ValidateLinks for HitTestShape {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.transform.iter().flatten().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if let Some(mesh) = self.mesh {
            visitor(Link::TypedMesh {
                id: mesh,
                kind: GeometryKind::Collision,
            });
        }
        if !self.size_x.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.size_y.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.size_z.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for AttachmentVolume {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Node(self.bone1));
        visitor(Link::Node(self.bone2));
        if self.transform.iter().flatten().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if let Some(mesh) = self.mesh {
            visitor(Link::TypedMesh {
                id: mesh,
                kind: GeometryKind::Collision,
            });
        }
        if !self.size_x.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.size_y.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.size_z.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for TurretBehavior {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.transform.iter().flatten().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.unknown1.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.unknown2.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if !self.yaw_min.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.yaw_max.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.yaw_weight.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.pitch_min.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.pitch_max.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.pitch_weight.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.unknown3.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.unknown4.is_finite() {
            visitor(Link::InvalidValue);
        }
        if self.main_bone_offset.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for BillboardBehavior {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for id in self.dependents.iter() {
            visitor(Link::Node(*id));
        }
        if self.up.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.forward.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for OneBoneSolver {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for id in self.dependents.iter() {
            visitor(Link::Node(*id));
        }
        if let Some(id) = self.bone_fallback {
            visitor(Link::Node(id));
        }
        if !self.max_angle.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Force {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.strength.visit_links(visitor);
        self.width.visit_links(visitor);
        self.height.visit_links(visitor);
        self.length.visit_links(visitor);
    }
}
impl ValidateLinks for Warp {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.radius.visit_links(visitor);
        self.height.visit_links(visitor);
        self.strength.visit_links(visitor);
        self.angular.visit_links(visitor);
        self.axial.visit_links(visitor);
        self.radial.visit_links(visitor);
    }
}
impl ValidateLinks for Light {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.diffuse_color.visit_links(visitor);
        self.intensity_multiplier.visit_links(visitor);
        self.specular_color.visit_links(visitor);
        self.specular_multiplier.visit_links(visitor);
        self.decay.visit_links(visitor);
        if !self.attenuation_end.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.attenuation_start.visit_links(visitor);
        self.hot_spot.visit_links(visitor);
        self.falloff.visit_links(visitor);
    }
}
impl<'a> ValidateLinks for Camera<'a> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.field_of_view.visit_links(visitor);
        self.shadow_clip_distance.visit_links(visitor);
        self.focus_distance.visit_links(visitor);
        self.far_focus_range.visit_links(visitor);
        self.near_focus_range.visit_links(visitor);
        self.dof_amount.visit_links(visitor);
        self.optics.visit_links(visitor);
    }
}
impl ValidateLinks for CameraClipping {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.far.visit_links(visitor);
        self.near.visit_links(visitor);
    }
}
impl ValidateLinks for CameraFalloff {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.start.visit_links(visitor);
        self.end.visit_links(visitor);
    }
}
impl ValidateLinks for CameraBokeh {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.f_stop.visit_links(visitor);
        self.maximum_circle.visit_links(visitor);
    }
}
impl ValidateLinks for IkJoint {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for id in self.dependents.iter() {
            visitor(Link::Node(*id));
        }
        visitor(Link::Node(self.second));
        if !self.raycast_up.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.raycast_down.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.max_speed.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.goal_threshold.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for IkTwoJoint {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for id in self.dependents.iter() {
            visitor(Link::Node(*id));
        }
        visitor(Link::Node(self.target));
        visitor(Link::Node(self.end));
        if self.hinge_axis.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if !self.max_angle_inner.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.max_angle_outer.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.search_up.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.search_down.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for IkCcd {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for id in self.dependents.iter() {
            visitor(Link::Node(*id));
        }
        visitor(Link::Node(self.target));
        if !self.search_up.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.search_down.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for ShadowBox {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.length.visit_links(visitor);
        self.width.visit_links(visitor);
        self.height.visit_links(visitor);
    }
}
impl ValidateLinks for ViewVolume {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.size.visit_links(visitor);
    }
}
impl ValidateLinks for ModelMetadata {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for BoneControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.position.visit_links(visitor);
        self.rotation.visit_links(visitor);
        self.scale.visit_links(visitor);
        self.batching.visit_links(visitor);
    }
}
impl<'a> ValidateLinks for ExternalModel<'a> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for VariationControl {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.amplitude.visit_links(visitor);
        self.frequency.visit_links(visitor);
    }
}
impl ValidateLinks for SplinePoint {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Node(self.node));
        if self.emission_offset.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.emission_vector.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        self.velocity.visit_links(visitor);
        self.velocity_base_factor.visit_links(visitor);
        self.velocity_end_factor.visit_links(visitor);
        self.yaw_variation.visit_links(visitor);
        self.pitch_variation.visit_links(visitor);
        self.velocity_variation.visit_links(visitor);
        self.yaw.visit_links(visitor);
        self.pitch.visit_links(visitor);
        if !self.emission_vector_norm_factor.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.velocity_norm_factor.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for ShapeGeometry<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        fn scalar(value: f32, visitor: &mut dyn FnMut(Link)) {
            if !value.is_finite() {
                visitor(Link::InvalidValue);
            }
        }
        fn size(value: f32, visitor: &mut dyn FnMut(Link)) {
            if !value.is_finite() || value < 0.0 {
                visitor(Link::InvalidValue);
            }
        }
        match self {
            Self::Box { half_extents } => {
                for value in half_extents {
                    size(*value, visitor);
                }
            }
            Self::Sphere {
                radius,
                axis_controls,
            } => {
                size(*radius, visitor);
                for value in axis_controls {
                    scalar(*value, visitor);
                }
            }
            Self::Capsule {
                radius,
                length,
                axis_control,
            }
            | Self::Cylinder {
                radius,
                length,
                axis_control,
            } => {
                size(*radius, visitor);
                size(*length, visitor);
                scalar(*axis_control, visitor);
            }
            Self::ConvexHull {
                mesh,
                planes,
                controls,
            }
            | Self::TriangleMesh {
                mesh,
                planes,
                controls,
            } => {
                for value in controls {
                    scalar(*value, visitor);
                }
                visitor(Link::TypedMesh {
                    id: *mesh,
                    kind: GeometryKind::Collision,
                });
                for plane in planes.iter() {
                    for value in plane {
                        scalar(value, visitor);
                    }
                }
            }
            Self::Unknown {
                dimensions,
                mesh,
                planes,
                ..
            } => {
                for value in dimensions {
                    scalar(*value, visitor);
                }
                if let Some(id) = mesh {
                    visitor(Link::TypedMesh {
                        id: *id,
                        kind: GeometryKind::Collision,
                    });
                }
                for plane in planes.iter() {
                    for value in plane {
                        scalar(value, visitor);
                    }
                }
            }
        }
    }
}
impl ValidateLinks for AuxiliaryGeometry<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::TypedMesh {
            id: self.mesh,
            kind: GeometryKind::Collision,
        });
        if self
            .planes
            .iter()
            .any(|plane| plane.iter().any(|value| !value.is_finite()))
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Shape<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self
            .transform
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
            || self
                .margin
                .is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            visitor(Link::InvalidValue);
        }
        self.geometry.visit_links(visitor);
        if let Some(value) = &self.auxiliary_geometry {
            value.visit_links(visitor);
        }
    }
}
impl ValidateLinks for BodyParameters {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if [
            self.mass,
            self.friction,
            self.restitution,
            self.linear_damping,
            self.angular_damping,
            self.gravity_factor,
        ]
        .iter()
        .any(|value| !value.is_finite())
            || self.mass < 0.0
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for DynamicState {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.state.visit_links(visitor);
        if !self.blend_out.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Body<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Node(self.node));
        if let Some(value) = &self.parameters {
            value.visit_links(visitor);
        }
        if let Some(value) = &self.dynamic {
            value.visit_links(visitor);
        }
        self.shapes.visit_links(visitor);
    }
}
impl ValidateLinks for Joint {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Node(self.node_a));
        visitor(Link::Node(self.node_b));
        if self.frame_a.iter().flatten().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.frame_b.iter().flatten().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if !self.limit_min.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.limit_max.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.cone_angle.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.friction.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.damping_ratio.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.angular_frequency.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.break_threshold.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for ClothCollider {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.transform.iter().flatten().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if !self.radius.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.height.is_finite() {
            visitor(Link::InvalidValue);
        }
        if let Some(bone) = self.bone {
            visitor(Link::Node(bone));
        }
    }
}
impl<'a> ValidateLinks for ClothProxy<'a> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Mesh(self.mesh));
        for row in self.influences.iter() {
            if row
                .iter()
                .any(|influence| !influence.weight.is_finite() || influence.weight < 0.)
            {
                visitor(Link::InvalidValue);
            }
        }
    }
}
impl<'a> ValidateLinks for Cloth<'a> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Mesh(self.cage));
        for row in self.anchors.iter() {
            for influence in row.into_iter().flatten() {
                if !influence.weight.is_finite() || influence.weight < 0. {
                    visitor(Link::InvalidValue);
                } else if influence.weight != 0. {
                    visitor(Link::Node(influence.node));
                }
            }
        }
        for child in self.colliders.iter() {
            child.visit_links(visitor);
        }
        for child in self.proxies.iter() {
            child.visit_links(visitor);
        }
        if !self.density.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.tracking.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.stretch_stiffness.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.horizontal_stiffness.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.bending_stiffness.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.damping.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.friction.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.gravity.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.explosion_scale.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.wind_scale.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.shear_stiffness.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.drag_factor.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.lift_factor.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.sphere_stiffness.is_finite() {
            visitor(Link::InvalidValue);
        }
        if let Some(active) = &self.active {
            active.visit_links(visitor);
        }
        if !self.skin_offset.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.skin_exponent.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.skin_stiffness.is_finite() {
            visitor(Link::InvalidValue);
        }
        if self.local_wind.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Reflection {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.reflection.parameters.visit_links(visitor);
        self.displacement.parameters.visit_links(visitor);
        self.reflection_offset.visit_links(visitor);
        self.blur_angle.visit_links(visitor);
        self.blur_distance_max.visit_links(visitor);
        if let Some(child) = &self.reflection.input {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.displacement.input {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.blur_map {
            child.visit_links(visitor);
        }
    }
}
impl ValidateLinks for Standard {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.state.visit_links(visitor);
        self.diffuse.visit_links(visitor);
        self.specular.visit_links(visitor);
        self.emissive.visit_links(visitor);
        self.opacity.visit_links(visitor);
        self.normals.visit_links(visitor);
        self.hdr_environment_constant.visit_links(visitor);
        self.hdr_environment_diffuse.visit_links(visitor);
        self.hdr_environment_specular.visit_links(visitor);
        self.decal_layer.visit_links(visitor);
        self.environment_layer.visit_links(visitor);
        self.environment_mask_layer.visit_links(visitor);
        self.height_layer.visit_links(visitor);
        self.light_map_layer.visit_links(visitor);
        self.ambient_occlusion_layer.visit_links(visitor);
        self.parallax_height.visit_links(visitor);
        self.motion_blur_amount.visit_links(visitor);
    }
}

impl ValidateLinks for AnimationControl {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl<'a> ValidateLinks for DataDriven<'a> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for id in self.textures.iter() {
            visitor(Link::Texture(*id));
        }
        if !self.unknown108.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.unknown112.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.unknown116.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl<'a> ValidateLinks for DataProperties<'a> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for child in self.groups.iter() {
            child.visit_links(visitor);
        }
    }
}
impl ValidateLinks for SubFlare {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.position.is_finite() {
            visitor(Link::InvalidValue);
        }
        if self.size_xy.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.scale_xy.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.fade_in.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.fade_out.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.color_alpha.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.offset.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
    }
}
impl<'a> ValidateLinks for LensFlare<'a> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some(child) = &self.flare_map {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.mask_map {
            child.visit_links(visitor);
        }
        for child in self.sub_flares.iter() {
            child.visit_links(visitor);
        }
        if !self.distance_fade.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.intensity.visit_links(visitor);
        self.color.visit_links(visitor);
        self.hdr.visit_links(visitor);
        self.size.visit_links(visitor);
    }
}
impl ValidateLinks for Displacement {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.displacement.parameters.visit_links(visitor);
        if let Some(child) = &self.displacement.input {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.strength_map {
            child.visit_links(visitor);
        }
    }
}
impl ValidateLinks for Terrain {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some(child) = &self.terrain_map {
            child.visit_links(visitor);
        }
    }
}
impl ValidateLinks for Creep {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some(child) = &self.mask_map {
            child.visit_links(visitor);
        }
    }
}
impl ValidateLinks for SplatTerrainBake {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some(child) = &self.diffuse_map {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.normal_map {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.specular_map {
            child.visit_links(visitor);
        }
    }
}
impl ValidateLinks for Hair {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some(child) = &self.diffuse.input {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.layer_spec_shift {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.layer_spec_noise {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.layer_ambient_occlusion {
            child.visit_links(visitor);
        }
        if !self.shift_primary.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.shift_secondary.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.diffuse.parameters.visit_links(visitor);
        self.color_specular.visit_links(visitor);
        if !self.specular_exponent_primary.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.specular_exponent_secondary.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for CompositeSection {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Material(self.material));
        self.map_multiplier.visit_links(visitor);
    }
}
impl ValidateLinks for Composite {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for child in self.sections.iter() {
            child.visit_links(visitor);
        }
    }
}
impl ValidateLinks for TextureLayer {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.binding.visit_links(visitor);
        if !self.fresnel_exponent.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.fresnel_min.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.fresnel_max.is_finite() {
            visitor(Link::InvalidValue);
        }
        if self.fresnel_translation.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.fresnel_mask.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.fresnel_rotation.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Volume {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.density.visit_links(visitor);
        if let Some(child) = &self.color_map {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.noise_map1 {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.noise_map2 {
            child.visit_links(visitor);
        }
    }
}
impl ValidateLinks for VolumeNoise {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.density.visit_links(visitor);
        self.near_plane.visit_links(visitor);
        self.falloff.visit_links(visitor);
        if let Some(child) = &self.color_map {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.noise_map1 {
            child.visit_links(visitor);
        }
        if let Some(child) = &self.noise_map2 {
            child.visit_links(visitor);
        }
        self.scroll_rate.visit_links(visitor);
        self.position.visit_links(visitor);
        self.scale.visit_links(visitor);
        self.rotation.visit_links(visitor);
    }
}

impl<T: CurveValue + 'static> ValidateLinks for Property<T> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.initial.is_valid() || !reset_valid(&self.reset, self.control.flags.bits()) {
            visitor(Link::InvalidValue);
        }
        self.binding.visit_links(visitor);
    }
}
impl<T: CurveValue + 'static> ValidateLinks for PropertyControl<T> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !reset_valid(&self.reset, self.control.flags.bits()) {
            visitor(Link::InvalidValue);
        }
        self.binding.visit_links(visitor);
    }
}
fn reset_valid<T: CurveValue + 'static>(value: &T, flags: u16) -> bool {
    if value.is_valid() {
        return true;
    }
    if flags != 0 {
        return false;
    }
    let any = value as &dyn std::any::Any;
    any.downcast_ref::<Quaternion>()
        .is_some_and(|q| q.components().iter().all(|v| v.is_finite() && *v == 0.))
}
impl ValidateLinks for Material<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Standard(value) => value.visit_links(visitor),
            Self::Displacement(value) => value.visit_links(visitor),
            Self::Composite(value) => value.visit_links(visitor),
            Self::Terrain(value) => value.visit_links(visitor),
            Self::Volume(value) => value.visit_links(visitor),
            Self::VolumeNoise(value) => value.visit_links(visitor),
            Self::Creep(value) => value.visit_links(visitor),
            Self::Hair(value) => value.visit_links(visitor),
            Self::SplatTerrainBake(value) => value.visit_links(visitor),
            Self::Reflection(value) => value.visit_links(visitor),
            Self::LensFlare(value) => value.visit_links(visitor),
            Self::DataDriven(value) => value.visit_links(visitor),
        }
    }
}
impl Feature<'_> {
    pub(crate) fn visit_profile_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Event(value) => value.visit_links(visitor),
            Self::Extent(value) => value.visit_links(visitor),
            Self::Particle(value) => value.visit_links(visitor),
            Self::ParticleCopy(value) => value.visit_links(visitor),
            Self::Ribbon(value) => value.visit_links(visitor),
            Self::Light(value) => value.visit_links(visitor),
            Self::Camera(value) => value.visit_links(visitor),
            Self::Projector(value) => value.visit_links(visitor),
            Self::Force(value) => value.visit_links(visitor),
            Self::Warp(value) => value.visit_links(visitor),
            Self::Attachment(value) => value.visit_links(visitor),
            Self::HitTest(value) => value.visit_links(visitor),
            Self::AttachmentVolume(value) => value.visit_links(visitor),
            Self::Billboard(value) => value.visit_links(visitor),
            Self::Turret(value) => value.visit_links(visitor),
            Self::IkJoint(value) => value.visit_links(visitor),
            Self::IkTwoJoint(value) => value.visit_links(visitor),
            Self::IkCcd(value) => value.visit_links(visitor),
            Self::OneBoneSolver(value) => value.visit_links(visitor),
            Self::ShadowBox(value) => value.visit_links(visitor),
            Self::ViewVolume(value) => value.visit_links(visitor),

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
impl ValidateLinks for Parameters<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Named(value) = self {
            value.visit_links(visitor)
        }
    }
}

impl ValidateLinks for super::CameraOptics {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Legacy { controls } => {
                if controls.iter().any(|v| !v.is_finite()) {
                    visitor(Link::InvalidValue);
                }
            }
            Self::Clipping(clipping) => clipping.visit_links(visitor),
            Self::Falloff { clipping, falloff } => {
                clipping.visit_links(visitor);
                falloff.visit_links(visitor);
            }
            Self::Bokeh {
                clipping,
                falloff,
                bokeh,
            } => {
                clipping.visit_links(visitor);
                falloff.visit_links(visitor);
                bokeh.visit_links(visitor);
            }
        }
    }
}

impl ValidateLinks for super::ParticleCopy {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::TypedFeature {
            id: self.source,
            kind: FeatureKind::M3Particle,
        });
        self.emission_rate.visit_links(visitor);
        self.squirt_amount.visit_links(visitor);
    }
}

impl ValidateLinks for NodeRole {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Bone(value) = self {
            value.visit_links(visitor);
        }
    }
}
