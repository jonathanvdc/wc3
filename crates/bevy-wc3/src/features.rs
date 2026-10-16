//! Authored scene features evaluated without taking ownership of application cameras.
use super::child_coordinates::{ChildCoordinates, CoordinatePolicy};
use super::runtime::{ModelDraw, ModelNode};
use super::world::WorldLiquidSurface;
use super::{ModelAsset, ModelInstance, ModelRig, Playback};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use wc3::model::profiles::Universal;
use wc3::model::scene::world::LightKind as WorldLightKind;
use wc3::model::scene::{Camera as PortableCamera, LightKind};
use wc3::model::values::programmed::LightKind as ProgrammedLightKind;
use wc3::model::{
    AimedCamera, Attachment, Camera as ModelCamera, CameraConvention, ClipId, FeatureId,
    FeatureKind, IlluminationKind, Light as SemanticLight, LightActivation, Model, ModelMount,
    MountTarget, NodeFeature, NodeId, PlacementAppearance as SemanticPlacementAppearance,
    ResourceLocator, SampledAnimation,
};

/// Conversion controls on a model's animation root. Absence uses the defaults.
/// These are integration scales, not calibrated game-to-physical conversions.
#[derive(Component, Clone, Copy, Debug)]
pub struct ModelLightSettings {
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

impl Default for ModelLightSettings {
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
pub struct ModelFeature {
    /// Feature identity in the source graph.
    pub id: FeatureId,
    /// Hierarchy node carrying the feature.
    pub node: NodeId,
}

/// Complete semantic attachment definition, including authored activation and appearance.
#[derive(Component, Clone, Debug)]
pub struct ModelAttachment {
    /// Authored attachment with its source graph identities.
    pub definition: Attachment<'static>,
    /// Owned child declaration, retained after the semantic model is released.
    pub resource: Option<ResourceLocator<'static>>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AttachmentClock {
    clip: Option<ClipId>,
    time: f64,
}

/// Sampled attachment state, separate from the immutable authored definition.
#[derive(Component, Clone, Debug)]
pub struct AttachmentState {
    /// Current fractional visibility, or zero/one for boolean activation.
    pub visibility: f32,
    clock: Option<AttachmentClock>,
}

/// Authored illumination and application-controlled per-light suppression.
#[derive(Component, Clone, Debug)]
pub struct ModelLight {
    /// Complete immutable authored definition, including unmapped controls.
    pub definition: SemanticLight,
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
pub struct SampledCamera {
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
    feature: ModelFeature,
    definition: Attachment<'static>,
    resource: Option<ResourceLocator<'static>>,
    model: Option<Handle<ModelAsset>>,
    placement: Option<ModelMount>,
    parent_basis: Transform,
}
impl AttachmentDefinition {
    fn visibility(&self, sample: &SampledAnimation<'_, '_, Universal>) -> f32 {
        self.definition.visibility(sample)
    }
}
#[derive(Clone, Default)]
pub(super) struct RuntimeFeatures {
    lights: Vec<(ModelFeature, SemanticLight)>,
    cameras: Vec<(ModelFeature, CameraDefinition)>,
    attachments: Vec<AttachmentDefinition>,
    hooks: Vec<(ModelFeature, FeatureKind)>,
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

pub(super) type LightQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut ModelLight,
        Option<&'static mut PointLight>,
        Option<&'static mut DirectionalLight>,
        Option<&'static mut SpotLight>,
    ),
>;
pub(super) type AttachmentQuery<'world, 'state> = Query<
    'world,
    'state,
    (&'static mut AttachmentState, &'static mut Visibility),
    (
        Without<ModelNode>,
        Without<ModelDraw>,
        Without<WorldLiquidSurface>,
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
            SemanticLight::World(_)=>Some((feature.id,"WMO common diffuse lights use Bevy intensity/range conversion; native cone calibration, Euler orientation, color blending, cookies, flicker, and per-region light-set activation remain uninterpreted")),
            SemanticLight::Specular(_)=>Some((feature.id,"M3 diffuse light animation, range and shadow flags use Bevy lights; independent specular response, distance-decay law, LOD gates and native cone calibration remain uninterpreted")),
            SemanticLight::Point(_)=>Some((feature.id,"Region point-light color, intensity and range use Bevy lights; native attenuation-start and unresolved controls remain uninterpreted")),
            _=>None,
        }).chain(self.attachments.iter().filter_map(|definition| match definition.definition.child().map(|value| &value.appearance) {
            Some(SemanticPlacementAppearance::World { .. }) => Some((definition.feature.id, "WMO placement appearance is retained on ModelAttachment; interior direct illumination, region-derived ambient lighting, linked lights and placement controls require application interpretation, without changing child albedo or scene lighting")),
            _ => None,
        }))
    }
    pub fn compile(
        model: &Model<'_>,
        children: &HashMap<FeatureId, Handle<ModelAsset>>,
        placements: &HashMap<FeatureId, CoordinatePolicy>,
    ) -> Self {
        let mut result = Self::default();
        for (id, value) in model.features() {
            let feature = ModelFeature {
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
                NodeFeature::Attachment(value) => result.attachments.push(AttachmentDefinition {
                    feature,
                    definition: value.clone().into_owned(),
                    resource: value.child().map(|child| {
                        model
                            .model_resource(child.resource)
                            .expect("validated attachment resource")
                            .clone()
                            .into_owned()
                    }),
                    model: value.child().and_then(|_| children.get(&id).cloned()),
                    placement: value.mount(id, feature.node).map(|mut placement| {
                        placement.coordinates = placements.get(&id).copied().unwrap_or_default();
                        placement
                    }),
                    parent_basis: ChildCoordinates::from_model(model, CoordinatePolicy::Host)
                        .parent_basis,
                }),
                _ => {}
            }
        }
        result
    }
    pub(crate) fn append_remapped(&mut self, source: &Self, nodes: &HashMap<NodeId, NodeId>) {
        let remap = |mut feature: ModelFeature| {
            feature.node = nodes[&feature.node];
            feature
        };
        self.lights.extend(
            source
                .lights
                .iter()
                .map(|(feature, light)| (remap(*feature), light.clone())),
        );
        self.cameras.extend(
            source
                .cameras
                .iter()
                .map(|(feature, camera)| (remap(*feature), camera.clone())),
        );
        self.attachments
            .extend(source.attachments.iter().cloned().map(|mut attachment| {
                attachment.feature = remap(attachment.feature);
                if let Some(placement) = &mut attachment.placement {
                    if let MountTarget::Node(node) = &mut placement.target {
                        *node = nodes[node];
                    }
                }
                attachment
            }));
        self.hooks.extend(
            source
                .hooks
                .iter()
                .map(|(feature, kind)| (remap(*feature), *kind)),
        );
    }
    pub(crate) fn attachment_node(&self, id: FeatureId) -> Option<NodeId> {
        self.attachments
            .iter()
            .find(|definition| {
                definition.feature.id == id && definition.definition.socket().is_some()
            })
            .map(|definition| definition.feature.node)
    }
    pub(crate) fn attachment_entity(
        &self,
        id: FeatureId,
        instances: &RuntimeFeatureInstances,
    ) -> Option<Entity> {
        self.attachments
            .iter()
            .position(|definition| {
                definition.feature.id == id && definition.definition.socket().is_some()
            })
            .and_then(|index| instances.attachments.get(index).copied())
    }
    pub(crate) fn attachment_active(
        &self,
        id: FeatureId,
        sample: &SampledAnimation<'_, '_, Universal>,
    ) -> bool {
        self.attachments
            .iter()
            .find(|definition| {
                definition.feature.id == id && definition.definition.socket().is_some()
            })
            .is_some_and(|definition| definition.visibility(sample) > 0.)
    }
    /// All authored integration identities, including collision and controllers.
    pub fn hooks(&self) -> &[(ModelFeature, FeatureKind)] {
        &self.hooks
    }
    pub fn camera_ids(&self) -> impl Iterator<Item = ModelFeature> + '_ {
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
                ModelLight {
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
                SemanticLight::Portable(light) => light.kind,
                SemanticLight::AmbientDiffuse(light) => match light.kind {
                    IlluminationKind::Point => LightKind::Point,
                    IlluminationKind::Directional => LightKind::Directional,
                    _ => LightKind::Ambient,
                },
                SemanticLight::World(light) => match light.kind {
                    WorldLightKind::Point => LightKind::Point,
                    WorldLightKind::Spot => LightKind::Spot,
                    WorldLightKind::Directional => LightKind::Directional,
                    _ => LightKind::Ambient,
                },
                SemanticLight::Point(_) => LightKind::Point,
                SemanticLight::Specular(light) => match light.light_type {
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
            let mut entity = commands.spawn((
                definition.feature,
                AttachmentState {
                    visibility: 1.,
                    clock: None,
                },
                ModelAttachment {
                    definition: definition.definition.clone(),
                    resource: definition.resource.clone(),
                },
                Transform::default(),
                Visibility::default(),
                ChildOf(parent),
            ));
            if let (Some(model), Some(placement)) = (&definition.model, definition.placement) {
                entity.insert((
                    ModelInstance(model.clone()),
                    ChildCoordinates {
                        coordinates: placement.coordinates,
                        parent_basis: definition.parent_basis,
                    },
                ));
            }
            instances.attachments.push(entity.id());
        }
        instances
    }
    pub fn animate(
        &self,
        sample: &SampledAnimation<'_, '_>,
        instances: &RuntimeFeatureInstances,
        settings: &ModelLightSettings,
        lights: &mut LightQuery<'_, '_>,
        attachments: &mut AttachmentQuery<'_, '_>,
    ) {
        for &entity in &instances.lights {
            let Ok((mut state, point, directional, spot)) = lights.get_mut(entity) else {
                continue;
            };
            let (color, intensity, range, activation, shadows, ambient, ambient_intensity, cone) =
                match &state.definition {
                    SemanticLight::Portable(light) => (
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
                    SemanticLight::AmbientDiffuse(light) => {
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
                    SemanticLight::World(light) => {
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
                    SemanticLight::Point(light) => (
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
                    SemanticLight::Specular(light) => (
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
                let activation = definition.visibility(sample);
                set_if_changed!(point.visibility, activation);
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
    ) -> Option<SampledCamera> {
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
        Some(SampledCamera {
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
    fn compiled_attachments_own_their_resolved_child_declaration() {
        let compiled = {
            let path = b"child.mdx".to_vec();
            let mut draft = Model::<Universal>::builder("child declaration");
            let resource =
                draft.add_model_resource(ResourceLocator::Path(std::borrow::Cow::Borrowed(&path)));
            let node = draft.add_node(Node::named("child")).unwrap();
            draft
                .add_feature(
                    node,
                    NodeFeature::Attachment(Attachment::Child(wc3::model::ChildModel {
                        resource,
                        socket: None,
                        appearance: wc3::model::PlacementAppearance::Unmodified,
                    })),
                )
                .unwrap();
            let model = draft.finish().unwrap();
            RuntimeFeatures::compile(&model, &HashMap::new(), &HashMap::new())
        };
        assert!(
            matches!(compiled.attachments[0].resource.as_ref(), Some(ResourceLocator::Path(std::borrow::Cow::Owned(path))) if path == b"child.mdx")
        );
    }
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
                NodeFeature::Light(SemanticLight::World(Box::new(world::Light {
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
                NodeFeature::Light(SemanticLight::Specular(Box::new(programmed::Light {
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
                    &ModelLightSettings::default(),
                    &mut lights,
                    &mut attachments,
                );
            },
        );
        app.update();
        let mut lights = app.world_mut().query::<(&ModelLight, &PointLight)>();
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

type AttachmentParentData = (&'static ModelRig, &'static Playback);
type AttachmentChildData = (
    &'static mut AttachmentState,
    &'static mut Playback,
    &'static mut Visibility,
);

/// Sample parent activation before child pose evaluation, then apply restarts.
pub(crate) fn restart_attachments(
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
            let activation = definition.visibility(&sample);
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
    point: &mut AttachmentState,
    playback: &mut Playback,
    activation: f32,
    clock: AttachmentClock,
) {
    let changed_clock = point
        .clock
        .is_some_and(|previous| previous.clip != clock.clip || clock.time < previous.time);
    if activation > 0. && (point.visibility <= 0. || changed_clock) {
        playback.restart();
    }
    point.clock = Some(clock);
    point.visibility = activation;
}
#[cfg(test)]
mod attachment_lifecycle_tests {
    use super::*;
    #[test]
    fn inactive_mounts_restart_only_on_activation_or_parent_backwards_traversal() {
        let mut point = AttachmentState {
            visibility: 0.,
            clock: None,
        };
        let mut playback = Playback::new();
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
