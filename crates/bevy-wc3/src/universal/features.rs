//! Authored scene features evaluated without taking ownership of application cameras.
use super::placements::{ChildModelPlacement, ModelPlacementSpace};
use super::runtime::{UniversalDraw, UniversalNode};
use super::world::UniversalWorldLiquidSurface;
use super::{UniversalInstance, UniversalModelAsset, UniversalModelInstance, UniversalPlayback};
use bevy::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use wc3::model::scene::world::LightKind as WorldLightKind;
use wc3::model::scene::{Camera as PortableCamera, LightKind};
use wc3::model::values::programmed::LightKind as ProgrammedLightKind;
use wc3::model::{
    AimedCamera, Attachment, AttachmentActivation, AttachmentIdentity, AttachmentPoint,
    Camera as ModelCamera, CameraConvention, ClipId, FeatureId, FeatureKind, IlluminationKind,
    Light as ModelLight, LightActivation, Model, NodeFeature, NodeId, PlacementAppearance,
    Resource as ModelResource, SampledAnimation,
};

/// Conversion controls on a model's animation root. Absence uses the defaults.
/// These are integration scales, not calibrated game-to-physical conversions.
#[derive(Component, Clone, Copy, Debug)]
pub struct UniversalLightSettings {
    /// Disable all imported lights without removing their entities.
    pub enabled: bool,
    /// Lumens per authored point-light intensity unit.
    pub point_intensity_scale: f32,
    /// Lux per authored directional-light intensity unit.
    pub directional_intensity_scale: f32,
    /// World units per authored attenuation-end unit. Node scale is not applied.
    pub range_scale: f32,
    /// World-space range when attenuation end is nonpositive or nonfinite.
    pub fallback_range: f32,
    /// Allow shadow maps for lights whose source has ShadowCasting enabled.
    pub shadows_enabled: bool,
}

impl Default for UniversalLightSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            point_intensity_scale: 1_000.0,
            directional_intensity_scale: 10_000.0,
            range_scale: 1.0,
            fallback_range: 1_000.0,
            shadows_enabled: true,
        }
    }
}

/// Semantic provenance of an application integration point.
#[derive(Component, Clone, Copy, Debug)]
pub struct UniversalFeature {
    /// Feature identity in the source graph.
    pub id: FeatureId,
    /// Hierarchy node carrying the feature.
    pub node: NodeId,
}

/// Complete authored external-placement appearance, independent of interpretation.
/// WMO colors describe illumination and are not blindly multiplied into albedo.
#[derive(Component, Clone, Debug)]
pub struct UniversalPlacementAppearance(pub PlacementAppearance);

#[derive(Clone, Copy, Debug)]
pub(crate) struct AttachmentClock {
    clip: Option<ClipId>,
    time: f64,
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
    clock: Option<AttachmentClock>,
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
    appearance: PlacementAppearance,
    model: Option<Handle<UniversalModelAsset>>,
    placement: ChildModelPlacement,
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
impl RuntimeFeatureInstances {
    pub(super) fn has_changed_light_input(&self, lights: &mut LightQuery<'_, '_>) -> bool {
        self.lights.iter().any(|entity| {
            lights
                .get_mut(*entity)
                .is_ok_and(|(state, ..)| state.is_changed())
        })
    }
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
    (
        Without<UniversalNode>,
        Without<UniversalDraw>,
        Without<UniversalWorldLiquidSurface>,
    ),
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
    pub(super) fn light_diagnostics(&self) -> impl Iterator<Item = (FeatureId, &'static str)> + '_ {
        self.lights.iter().filter_map(|(feature,light)|match light {
            ModelLight::World(_)=>Some((feature.id,"WMO common diffuse lights use Bevy intensity/range conversion; native cone calibration, Euler orientation, color blending, cookies, flicker, and per-region light-set activation remain uninterpreted")),
            ModelLight::Specular(_)=>Some((feature.id,"M3 diffuse light animation, range and shadow flags use Bevy lights; independent specular response, distance-decay law, LOD gates and native cone calibration remain uninterpreted")),
            ModelLight::Point(_)=>Some((feature.id,"Region point-light color, intensity and range use Bevy lights; native attenuation-start and unresolved controls remain uninterpreted")),
            _=>None,
        }).chain(self.attachments.iter().filter_map(|definition| match &definition.appearance {
            PlacementAppearance::World { .. } => Some((definition.feature.id, "WMO placement appearance is retained on UniversalPlacementAppearance; interior direct illumination, region-derived ambient lighting, linked lights and placement controls require application interpretation, without changing child albedo or scene lighting")),
            PlacementAppearance::Unmodified => None,
        }))
    }
    pub fn compile(
        model: &Model<'_>,
        children: &HashMap<FeatureId, Handle<UniversalModelAsset>>,
        placements: &HashMap<FeatureId, ModelPlacementSpace>,
    ) -> Self {
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
                        appearance: PlacementAppearance::Unmodified,
                        model: None,
                        placement: ChildModelPlacement::from_model(
                            model,
                            ModelPlacementSpace::Authored,
                        ),
                    })
                }
                NodeFeature::Attachment(Attachment::External(value)) => {
                    result.attachments.push(AttachmentDefinition {
                        feature,
                        point: value.point.as_ref().map(owned_point),
                        resource: Some(value.resource.clone().into_owned()),
                        appearance: value.appearance.clone(),
                        model: children.get(&id).cloned(),
                        placement: ChildModelPlacement::from_model(
                            model,
                            placements.get(&id).copied().unwrap_or_default(),
                        ),
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
        parents: &HashMap<FeatureId, Entity>,
    ) -> RuntimeFeatureInstances {
        let mut instances = RuntimeFeatureInstances::default();
        for (feature, definition) in &self.lights {
            let Some(&parent) = parents
                .get(&feature.id)
                .or_else(|| nodes.get(&feature.node))
            else {
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
                ModelLight::World(light) => match light.kind {
                    WorldLightKind::Point => LightKind::Point,
                    WorldLightKind::Spot => LightKind::Spot,
                    WorldLightKind::Directional => LightKind::Directional,
                    _ => LightKind::Ambient,
                },
                ModelLight::Point(_) => LightKind::Point,
                ModelLight::Specular(light) => match light.light_type {
                    ProgrammedLightKind::Point => LightKind::Point,
                    ProgrammedLightKind::Directional => LightKind::Directional,
                    ProgrammedLightKind::Spot => LightKind::Spot,
                    _ => LightKind::Ambient,
                },
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
            let Some(&parent) = parents
                .get(&definition.feature.id)
                .or_else(|| nodes.get(&definition.feature.node))
            else {
                continue;
            };
            let identity = definition.point.as_ref().map(|p| p.identity.clone());
            let mut entity = commands.spawn((
                definition.feature,
                UniversalAttachmentPoint {
                    identity,
                    resource: definition.resource.clone(),
                    activation: 1.,
                    clock: None,
                },
                UniversalPlacementAppearance(definition.appearance.clone()),
                Transform::default(),
                Visibility::default(),
                ChildOf(parent),
            ));
            if let Some(model) = &definition.model {
                entity.insert((UniversalModelInstance(model.clone()), definition.placement));
            }
            instances.attachments.push(entity.id());
        }
        instances
    }
    pub fn animate(
        &self,
        sample: &SampledAnimation<'_, '_>,
        instances: &RuntimeFeatureInstances,
        settings: &UniversalLightSettings,
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
                    ModelLight::World(light) => {
                        let color = sample.animated(&light.color);
                        let modern = light.modern.as_ref();
                        let multiplier = modern.map_or(1., |value| {
                            if value.intensity_multiplier == 0. {
                                1.
                            } else {
                                value.intensity_multiplier
                            }
                        });
                        let intensity = sample.animated(&light.intensity) * multiplier;
                        let ambient = light.kind == WorldLightKind::Ambient;
                        (
                            [color[0], color[1], color[2]],
                            if ambient { 0. } else { intensity },
                            if light.use_attenuation {
                                light.attenuation[1]
                            } else {
                                0.
                            },
                            1.,
                            modern.is_some_and(|value| value.shadows),
                            if ambient {
                                [color[0], color[1], color[2]]
                            } else {
                                [0.; 3]
                            },
                            if ambient { intensity } else { 0. },
                            light.cone,
                        )
                    }
                    ModelLight::Point(light) => (
                        [
                            light.color[0] as f32 / 255.,
                            light.color[1] as f32 / 255.,
                            light.color[2] as f32 / 255.,
                        ],
                        light.intensity,
                        light.attenuation[1],
                        1.,
                        false,
                        [0.; 3],
                        0.,
                        [0.; 2],
                    ),
                    ModelLight::Specular(light) => (
                        light
                            .diffuse_color
                            .binding
                            .as_ref()
                            .and_then(|property| sample.property(property))
                            .unwrap_or(light.diffuse_color.initial),
                        light
                            .intensity_multiplier
                            .binding
                            .as_ref()
                            .and_then(|property| sample.property(property))
                            .unwrap_or(light.intensity_multiplier.initial),
                        light.attenuation_end,
                        1.,
                        light.flags.shadows(),
                        [0.; 3],
                        0.,
                        [
                            light
                                .hot_spot
                                .binding
                                .as_ref()
                                .and_then(|property| sample.property(property))
                                .unwrap_or(light.hot_spot.initial),
                            light
                                .falloff
                                .binding
                                .as_ref()
                                .and_then(|property| sample.property(property))
                                .unwrap_or(light.falloff.initial),
                        ],
                    ),
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
                let outer = cone[1].clamp(0.001, FRAC_PI_2);
                set_if_changed!(light.inner_angle, cone[0].clamp(0., outer));
                set_if_changed!(light.outer_angle, outer);
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
        let features = RuntimeFeatures::compile(&model, &HashMap::new(), &HashMap::new());
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

#[cfg(test)]
mod imported_light_tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;
    use wc3::model::animation::programmed::Property;
    use wc3::model::scene::{programmed, world};
    use wc3::model::values::programmed::LightFlags;
    use wc3::model::{Animated, AnimationProgram, AnimationSample, Node};
    #[test]
    fn world_and_programmed_diffuse_lights_use_bevy_components_without_global_ambient() {
        let mut builder = Model::builder("imported lights");
        let node = builder.add_node(Node::named("anchor")).unwrap();
        builder
            .add_feature(
                node,
                NodeFeature::Light(ModelLight::World(Box::new(world::Light {
                    kind: WorldLightKind::Point,
                    color: Animated::constant([0.2, 0.4, 0.8, 0.]),
                    intensity: Animated::constant(2.),
                    use_attenuation: true,
                    attenuation: [1., 8.],
                    ..Default::default()
                }))),
            )
            .unwrap();
        builder
            .add_feature(
                node,
                NodeFeature::Light(ModelLight::Specular(Box::new(programmed::Light {
                    light_type: ProgrammedLightKind::Point,
                    flags: LightFlags::from(1),
                    lod_cut: 0,
                    shadow_lod_cut: 0,
                    diffuse_color: Property {
                        initial: [0.3, 0.5, 0.7],
                        ..Default::default()
                    },
                    intensity_multiplier: Property {
                        initial: 3.,
                        ..Default::default()
                    },
                    specular_color: Default::default(),
                    specular_multiplier: Default::default(),
                    decay: Default::default(),
                    attenuation_end: 12.,
                    attenuation_start: Default::default(),
                    hot_spot: Default::default(),
                    falloff: Default::default(),
                }))),
            )
            .unwrap();
        let model = builder.finish().unwrap();
        let features = RuntimeFeatures::compile(&model, &HashMap::new(), &HashMap::new());
        let animation = AnimationProgram::compile(&model);
        let mut app = App::new();
        let root = app
            .world_mut()
            .spawn((Transform::default(), Visibility::default()))
            .id();
        let mut queue = CommandQueue::default();
        let instances = features.spawn(
            &mut Commands::new(&mut queue, app.world()),
            &HashMap::from([(node, root)]),
            &HashMap::new(),
        );
        queue.apply(app.world_mut());
        app.add_systems(
            Update,
            move |mut lights: LightQuery, mut attachments: AttachmentQuery| {
                let sample = animation.sample(AnimationSample::default()).unwrap();
                features.animate(
                    &sample,
                    &instances,
                    &UniversalLightSettings::default(),
                    &mut lights,
                    &mut attachments,
                );
            },
        );
        app.update();
        let mut lights = app.world_mut().query::<(&UniversalLight, &PointLight)>();
        let values = lights.iter(app.world()).collect::<Vec<_>>();
        assert_eq!(values.len(), 2);
        assert!(values
            .iter()
            .any(|(_, light)| light.intensity == 2000. && light.range == 8.));
        assert!(values.iter().any(|(_, light)| light.intensity == 3000.
            && light.range == 12.
            && light.shadow_maps_enabled));
    }
}

type AttachmentParentData = (&'static UniversalInstance, &'static UniversalPlayback);
type AttachmentChildData = (
    &'static mut UniversalAttachmentPoint,
    &'static mut UniversalPlayback,
    &'static mut Visibility,
);

/// Sample parent activation before child pose evaluation, then apply restarts.
pub(crate) fn restart_universal_attachments(
    mut queries: ParamSet<(Query<AttachmentParentData>, Query<AttachmentChildData>)>,
    mut pending: Local<Vec<(Entity, f32, AttachmentClock)>>,
) {
    pending.clear();
    for (instance, playback) in &queries.p0() {
        let model = instance.model();
        if model.features.attachments.is_empty() {
            continue;
        }
        let Ok(selection) = playback.selection(model) else {
            continue;
        };
        let Ok(sample) = model.animation.sample(selection) else {
            continue;
        };
        for (definition, entity) in model
            .features
            .attachments
            .iter()
            .zip(&instance.feature_instances().attachments)
        {
            let activation = match definition.point.as_ref().map(|point| &point.activation) {
                Some(AttachmentActivation::Enabled(value)) => {
                    if sample.animated(value) {
                        1.
                    } else {
                        0.
                    }
                }
                Some(AttachmentActivation::Visibility(value)) => {
                    sample.animated(value).clamp(0., 1.)
                }
                _ => 1.,
            };
            pending.push((
                *entity,
                activation,
                AttachmentClock {
                    clip: playback.clip,
                    time: playback.clip_time,
                },
            ));
        }
    }
    let mut attachments = queries.p1();
    for &(entity, activation, clock) in pending.iter() {
        let Ok((mut point, mut playback, mut visibility)) = attachments.get_mut(entity) else {
            continue;
        };
        observe_attachment_clock(&mut point, &mut playback, activation, clock);
        let next_visibility = if activation > 0. {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next_visibility {
            *visibility = next_visibility;
        }
    }
}

fn observe_attachment_clock(
    point: &mut UniversalAttachmentPoint,
    playback: &mut UniversalPlayback,
    activation: f32,
    clock: AttachmentClock,
) {
    let changed_clock = point
        .clock
        .is_some_and(|previous| previous.clip != clock.clip || clock.time < previous.time);
    if activation > 0. && (point.activation <= 0. || changed_clock) {
        playback.restart();
    }
    point.clock = Some(clock);
    point.activation = activation;
}
#[cfg(test)]
mod attachment_lifecycle_tests {
    use super::*;
    #[test]
    fn inactive_mounts_restart_only_on_activation_or_parent_backwards_traversal() {
        let mut point = UniversalAttachmentPoint {
            identity: None,
            resource: None,
            activation: 0.,
            clock: None,
        };
        let mut playback = UniversalPlayback::new();
        playback.clip_time = 3.;
        let clock = AttachmentClock {
            clip: None,
            time: 2.,
        };
        observe_attachment_clock(&mut point, &mut playback, 0., clock);
        assert_eq!(playback.clip_time, 3.);
        observe_attachment_clock(&mut point, &mut playback, 1., clock);
        assert_eq!(playback.clip_time, 0.);
        playback.clip_time = 4.;
        observe_attachment_clock(
            &mut point,
            &mut playback,
            1.,
            AttachmentClock { time: 3., ..clock },
        );
        assert_eq!(playback.clip_time, 4.);
        observe_attachment_clock(
            &mut point,
            &mut playback,
            1.,
            AttachmentClock { time: 1., ..clock },
        );
        assert_eq!(playback.clip_time, 0.);
    }
}
