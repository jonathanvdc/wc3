//! Authored scene features evaluated without taking ownership of application cameras.
use super::runtime::{UniversalDraw, UniversalNode};
use crate::light::Wc3LightSettings;
use bevy::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use wc3::model::scene::{Camera as PortableCamera, LightKind};
use wc3::model::{
    AimedCamera, Attachment, AttachmentActivation, AttachmentIdentity, AttachmentPoint,
    Camera as ModelCamera, CameraConvention, FeatureId, FeatureKind, IlluminationKind,
    Light as ModelLight, LightActivation, Model, NodeFeature, NodeId, Resource as ModelResource,
    SampledAnimation,
};

/// Semantic provenance of an application integration point.
#[derive(Component, Clone, Copy, Debug)]
pub struct UniversalFeature {
    /// Feature identity in the source graph.
    pub id: FeatureId,
    /// Hierarchy node carrying the feature.
    pub node: NodeId,
}

/// Authored attachment identity and its current activation.
#[derive(Component, Clone, Debug)]
pub struct UniversalAttachmentPoint {
    /// Identity retains its namespace; numeric roles and indices are distinct.
    pub identity: Option<AttachmentIdentity<'static>>,
    /// Optional external model to resolve through the application's provider.
    pub resource: Option<ModelResource<'static>>,
    /// Current fractional visibility, or zero/one for boolean activation.
    pub activation: f32,
}

/// Authored illumination and application-controlled per-light suppression.
#[derive(Component, Clone, Debug)]
pub struct UniversalLight {
    /// Complete immutable authored definition, including unmapped controls.
    pub definition: ModelLight,
    /// Per-light enablement combined with animated activation.
    pub enabled: bool,
    /// Current ambient RGB contribution, retained for application integration.
    /// Bevy's global scene ambient resource is never modified by this renderer.
    pub ambient_color: [f32; 3],
    /// Current ambient multiplier after activation.
    pub ambient_intensity: f32,
}

/// A sampled authored camera expressed in model coordinates.
#[derive(Clone, Copy, Debug)]
pub struct SampledUniversalCamera {
    /// Eye position including authored local displacement.
    pub eye: Vec3,
    /// Authored aim point, or the node's forward axis for untargeted cameras.
    pub target: Vec3,
    /// Up vector including authored roll.
    pub up: Vec3,
    /// Vertical field of view in radians, converted using the requested aspect.
    pub vertical_field_of_view: f32,
    /// Authored near and far clipping planes in model units.
    pub clipping: [f32; 2],
    /// Authored visibility multiplier; applications choose whether to activate views.
    pub visibility: f32,
}

#[derive(Clone)]
enum CameraDefinition {
    Aimed(AimedCamera),
    Portable(PortableCamera),
}
#[derive(Clone)]
struct AttachmentDefinition {
    feature: UniversalFeature,
    point: Option<AttachmentPoint<'static>>,
    resource: Option<ModelResource<'static>>,
}
#[derive(Clone, Default)]
pub(super) struct RuntimeFeatures {
    lights: Vec<(UniversalFeature, ModelLight)>,
    cameras: Vec<(UniversalFeature, CameraDefinition)>,
    attachments: Vec<AttachmentDefinition>,
    hooks: Vec<(UniversalFeature, FeatureKind)>,
}
#[derive(Default)]
pub(super) struct RuntimeFeatureInstances {
    lights: Vec<Entity>,
    attachments: Vec<Entity>,
}

fn owned_point(point: &AttachmentPoint<'_>) -> AttachmentPoint<'static> {
    AttachmentPoint {
        identity: match &point.identity {
            AttachmentIdentity::Named(name) => {
                AttachmentIdentity::Named(Cow::Owned(name.to_string()))
            }
            AttachmentIdentity::Role(role) => AttachmentIdentity::Role(*role),
            AttachmentIdentity::Index(index) => AttachmentIdentity::Index(*index),
        },
        activation: point.activation.clone(),
        control: point.control,
    }
}
pub(super) type LightQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut UniversalLight,
        Option<&'static mut PointLight>,
        Option<&'static mut DirectionalLight>,
        Option<&'static mut SpotLight>,
    ),
>;
pub(super) type AttachmentQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut UniversalAttachmentPoint,
        &'static mut Visibility,
    ),
    (Without<UniversalNode>, Without<UniversalDraw>),
>;

macro_rules! set_if_changed {
    ($place:expr, $value:expr) => {{
        let value = $value;
        if $place != value {
            $place = value;
        }
    }};
}
impl RuntimeFeatures {
    pub fn compile(model: &Model<'_>) -> Self {
        let mut result = Self::default();
        for (id, value) in model.features() {
            let feature = UniversalFeature {
                id,
                node: value.node,
            };
            // Every feature has a provenance hook even when a dedicated visual
            // interpreter has not yet been implemented.
            result.hooks.push((feature, value.payload.kind()));
            match &value.payload {
                NodeFeature::Light(light) => result.lights.push((feature, light.clone())),
                NodeFeature::Camera(ModelCamera::Aimed(camera)) => result
                    .cameras
                    .push((feature, CameraDefinition::Aimed((**camera).clone()))),
                NodeFeature::Camera(ModelCamera::Portable(camera)) => result
                    .cameras
                    .push((feature, CameraDefinition::Portable((**camera).clone()))),
                NodeFeature::Attachment(Attachment::Point(point)) => {
                    result.attachments.push(AttachmentDefinition {
                        feature,
                        point: Some(owned_point(point)),
                        resource: None,
                    })
                }
                NodeFeature::Attachment(Attachment::External(value)) => {
                    result.attachments.push(AttachmentDefinition {
                        feature,
                        point: value.point.as_ref().map(owned_point),
                        resource: Some(value.resource.clone().into_owned()),
                    })
                }
                _ => {}
            }
        }
        result
    }
    /// All authored integration identities, including collision and controllers.
    pub fn hooks(&self) -> &[(UniversalFeature, FeatureKind)] {
        &self.hooks
    }
    pub fn camera_ids(&self) -> impl Iterator<Item = UniversalFeature> + '_ {
        self.cameras.iter().map(|(feature, _)| *feature)
    }
    pub fn spawn(
        &self,
        commands: &mut Commands,
        nodes: &HashMap<NodeId, Entity>,
    ) -> RuntimeFeatureInstances {
        let mut instances = RuntimeFeatureInstances::default();
        for (feature, definition) in &self.lights {
            let Some(&parent) = nodes.get(&feature.node) else {
                continue;
            };
            let mut entity = commands.spawn((
                *feature,
                UniversalLight {
                    definition: definition.clone(),
                    enabled: true,
                    ambient_color: [0.; 3],
                    ambient_intensity: 0.,
                },
                Transform::default(),
                Visibility::default(),
                ChildOf(parent),
            ));
            let kind = match definition {
                ModelLight::Portable(light) => light.kind,
                ModelLight::AmbientDiffuse(light) => match light.kind {
                    IlluminationKind::Point => LightKind::Point,
                    IlluminationKind::Directional => LightKind::Directional,
                    _ => LightKind::Ambient,
                },
                _ => LightKind::Ambient,
            };
            match kind {
                LightKind::Point => {
                    entity.insert(PointLight {
                        intensity: 0.,
                        ..Default::default()
                    });
                }
                LightKind::Directional => {
                    entity.insert(DirectionalLight {
                        illuminance: 0.,
                        ..Default::default()
                    });
                }
                LightKind::Spot => {
                    entity.insert(SpotLight {
                        intensity: 0.,
                        ..Default::default()
                    });
                }
                _ => {}
            }
            instances.lights.push(entity.id());
        }
        for definition in &self.attachments {
            let Some(&parent) = nodes.get(&definition.feature.node) else {
                continue;
            };
            let identity = definition.point.as_ref().map(|p| p.identity.clone());
            instances.attachments.push(
                commands
                    .spawn((
                        definition.feature,
                        UniversalAttachmentPoint {
                            identity,
                            resource: definition.resource.clone(),
                            activation: 1.,
                        },
                        Transform::default(),
                        Visibility::default(),
                        ChildOf(parent),
                    ))
                    .id(),
            );
        }
        instances
    }
    pub fn animate(
        &self,
        sample: &SampledAnimation<'_, '_>,
        instances: &RuntimeFeatureInstances,
        settings: &Wc3LightSettings,
        lights: &mut LightQuery<'_, '_>,
        attachments: &mut AttachmentQuery<'_, '_>,
    ) {
        for &entity in &instances.lights {
            let Ok((mut state, point, directional, spot)) = lights.get_mut(entity) else {
                continue;
            };
            let (color, intensity, range, activation, shadows, ambient, ambient_intensity, cone) =
                match &state.definition {
                    ModelLight::Portable(light) => (
                        sample.animated(&light.color),
                        sample.animated(&light.intensity),
                        sample.animated(&light.attenuation[1]),
                        1.,
                        light.shadow_casting.unwrap_or(false),
                        if light.kind == LightKind::Ambient {
                            sample.animated(&light.color)
                        } else {
                            [0.; 3]
                        },
                        if light.kind == LightKind::Ambient {
                            sample.animated(&light.intensity)
                        } else {
                            0.
                        },
                        light.cone.each_ref().map(|v| sample.animated(v)),
                    ),
                    ModelLight::AmbientDiffuse(light) => {
                        let (activation, shadows) = match &light.activation {
                            LightActivation::Enabled(enabled) => {
                                (if sample.animated(enabled) { 1. } else { 0. }, false)
                            }
                            LightActivation::Visibility(visible) => (
                                sample.animated(&visible.visibility),
                                visible.shadow_casting.unwrap_or(false),
                            ),
                        };
                        (
                            sample.animated(&light.diffuse.color),
                            sample.animated(&light.diffuse.intensity),
                            sample.animated(&light.attenuation[1]),
                            activation,
                            shadows,
                            sample.animated(&light.ambient.color),
                            sample.animated(&light.ambient.intensity),
                            [0., 0.],
                        )
                    }
                    _ => continue,
                };
            let activation = if state.enabled && settings.enabled {
                activation.clamp(0., 1.)
            } else {
                0.
            };
            let color = Color::linear_rgb(color[0].max(0.), color[1].max(0.), color[2].max(0.));
            let range = if range.is_finite() && range > 0. {
                range * settings.range_scale
            } else {
                settings.fallback_range
            };
            if state.ambient_color != ambient {
                state.ambient_color = ambient;
            }
            let ambient_intensity = ambient_intensity.max(0.) * activation;
            if state.ambient_intensity != ambient_intensity {
                state.ambient_intensity = ambient_intensity;
            }
            let intensity = intensity.max(0.) * activation;
            if let Some(mut light) = point {
                set_if_changed!(light.color, color);
                set_if_changed!(light.intensity, intensity * settings.point_intensity_scale);
                set_if_changed!(light.range, range);
                set_if_changed!(
                    light.shadow_maps_enabled,
                    shadows && settings.shadows_enabled
                );
            }
            if let Some(mut light) = directional {
                set_if_changed!(light.color, color);
                set_if_changed!(
                    light.illuminance,
                    intensity * settings.directional_intensity_scale
                );
                set_if_changed!(
                    light.shadow_maps_enabled,
                    shadows && settings.shadows_enabled
                );
            }
            if let Some(mut light) = spot {
                set_if_changed!(light.color, color);
                set_if_changed!(light.intensity, intensity * settings.point_intensity_scale);
                set_if_changed!(light.range, range);
                set_if_changed!(
                    light.shadow_maps_enabled,
                    shadows && settings.shadows_enabled
                );
                set_if_changed!(light.inner_angle, cone[0].max(0.).min(cone[1]));
                set_if_changed!(light.outer_angle, cone[1].clamp(0.001, FRAC_PI_2));
            }
        }
        for (definition, &entity) in self.attachments.iter().zip(&instances.attachments) {
            if let Ok((mut point, mut visibility)) = attachments.get_mut(entity) {
                let activation = match definition.point.as_ref().map(|p| &p.activation) {
                    Some(AttachmentActivation::Enabled(v)) => {
                        if sample.animated(v) {
                            1.
                        } else {
                            0.
                        }
                    }
                    Some(AttachmentActivation::Visibility(v)) => sample.animated(v).clamp(0., 1.),
                    _ => 1.,
                };
                set_if_changed!(point.activation, activation);
                set_if_changed!(
                    *visibility,
                    if activation > 0. {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    }
                );
            }
        }
    }
    pub fn sample_camera(
        &self,
        feature: FeatureId,
        sample: &SampledAnimation<'_, '_>,
        node: Mat4,
        model_up: Vec3,
        aspect: f32,
    ) -> Option<SampledUniversalCamera> {
        if !aspect.is_finite() || aspect <= 0. {
            return None;
        }
        let (_, definition) = self.cameras.iter().find(|(id, _)| id.id == feature)?;
        let (eye, target, fov, clipping, roll, visibility, diagonal) = match definition {
            CameraDefinition::Aimed(camera) => {
                let offset = Vec3::from(sample.animated(&camera.eye_offset));
                let target =
                    Vec3::from(camera.target) + Vec3::from(sample.animated(&camera.target_offset));
                let (visibility, diagonal) = match &camera.convention {
                    CameraConvention::Vertical(controls) => {
                        (sample.animated(&controls.visibility), false)
                    }
                    CameraConvention::Diagonal(_) => (1., true),
                };
                (
                    node.transform_point3(offset),
                    target,
                    sample.animated(&camera.field_of_view),
                    camera.clipping,
                    sample.animated(&camera.roll),
                    visibility,
                    diagonal,
                )
            }
            CameraDefinition::Portable(camera) => {
                let eye = node.transform_point3(Vec3::ZERO);
                let target = camera
                    .target
                    .as_ref()
                    .map(|v| Vec3::from(sample.animated(v)))
                    .unwrap_or_else(|| eye + node.transform_vector3(-Vec3::Z));
                (
                    eye,
                    target,
                    sample.animated(&camera.field_of_view),
                    camera.clipping,
                    0.,
                    1.,
                    false,
                )
            }
        };
        let fov = if diagonal {
            2. * ((fov * 0.5).tan() / (aspect * aspect + 1.).sqrt()).atan()
        } else {
            fov
        };
        let direction = (target - eye).try_normalize()?;
        Some(SampledUniversalCamera {
            eye,
            target,
            up: Quat::from_axis_angle(direction, roll) * model_up,
            vertical_field_of_view: fov,
            clipping,
            visibility,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::{Animated, AnimationProgram, AnimationSample, Node, PhotographicControls};
    #[test]
    fn aimed_camera_samples_offsets_and_roll_without_spawning_a_view() {
        let mut builder = Model::builder("authored camera");
        let node = builder.add_node(Node::named("camera anchor")).unwrap();
        let feature = builder
            .add_feature(
                node,
                NodeFeature::Camera(ModelCamera::Aimed(Box::new(AimedCamera {
                    eye_offset: Animated::constant([1., 0., 0.]),
                    target: [0., 10., 0.],
                    target_offset: Animated::constant([0., 0., 2.]),
                    field_of_view: Animated::constant(1.),
                    clipping: [0.1, 100.],
                    roll: Animated::constant(0.),
                    convention: CameraConvention::Vertical(Box::new(PhotographicControls {
                        visibility: Animated::constant(1.),
                        ..Default::default()
                    })),
                }))),
            )
            .unwrap();
        let model = builder.finish().unwrap();
        let features = RuntimeFeatures::compile(&model);
        let program = AnimationProgram::compile(&model);
        let sample = program.sample(AnimationSample::default()).unwrap();
        let camera = features
            .sample_camera(
                feature,
                &sample,
                Mat4::from_translation(Vec3::new(3., 0., 0.)),
                Vec3::Z,
                1.5,
            )
            .unwrap();
        assert_eq!(camera.eye, Vec3::new(4., 0., 0.));
        assert_eq!(camera.target, Vec3::new(0., 10., 2.));
        assert_eq!(camera.vertical_field_of_view, 1.);
        assert_eq!(camera.clipping, [0.1, 100.]);
    }
    #[test]
    fn resource_and_attachment_identity_detach_from_borrowed_storage() {
        let resource = ModelResource::Named {
            namespace: Cow::Borrowed("casc"),
            name: Cow::Borrowed("asset"),
        }
        .into_owned();
        let point = owned_point(&AttachmentPoint {
            identity: AttachmentIdentity::Named(Cow::Borrowed("hand")),
            activation: AttachmentActivation::Uncontrolled,
            control: None,
        });
        assert!(matches!(
            resource,
            ModelResource::Named {
                namespace: Cow::Owned(_),
                name: Cow::Owned(_)
            }
        ));
        assert!(matches!(
            point.identity,
            AttachmentIdentity::Named(Cow::Owned(_))
        ));
    }
}
