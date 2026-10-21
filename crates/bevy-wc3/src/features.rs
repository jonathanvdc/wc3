//! Authored scene features evaluated without taking ownership of application cameras.
use super::child_coordinates::{ChildCoordinates, CoordinatePolicy};
use super::runtime::{ModelDraw, ModelNode};
use super::world::WorldLiquidSurface;
use super::{ModelAsset, ModelInstance, ModelRig, Playback};
use crate::child_models::{ChildModelBinding, ChildModelKey};
use crate::effects::model_budget::ActorFrame;
use crate::effects::model_children::ParticleModelSubtree;
use crate::effects::model_ownership::queue_model_birth;
use crate::layers::ModelOwnedRenderEntity;
use bevy::ecs::query::QueryFilter;
use bevy::ecs::system::SystemParam;
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use wc3::model::scene::world::LightKind as WorldLightKind;
use wc3::model::{EmissionKind, NodeCamera};

#[cfg(test)]
use wc3::model::ModelMetadata;
use wc3::model::{
    Attachment, Camera as ModelCamera, ClipId, FeatureId, FeatureKind, IlluminationKind,
    Light as SemanticLight, LightActivation, LookAtCamera, Model, ModelMount, MountTarget,
    NodeFeature, NodeId, PlacementAppearance as SemanticPlacementAppearance, ResourceLocator,
    SampledAnimation,
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
    pub ambient_color: Vec3,
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
    LookAt(LookAtCamera),
    Node(NodeCamera),
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
    fn visibility(&self, sample: &SampledAnimation<'_, '_>) -> f32 {
        self.definition.visibility(sample)
    }
}
#[derive(Clone, Default)]
pub(super) struct RuntimeFeatures {
    lights: Vec<(ModelFeature, SemanticLight, Vec3)>,
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
        self.lights.iter().filter_map(|(feature,light,_)|match light {
            SemanticLight::World(_)=>Some((feature.id,"WMO common diffuse lights use Bevy intensity/range conversion; native cone calibration, Euler orientation, color blending, cookies, flicker, and per-region light-set activation remain uninterpreted")),

            SemanticLight::Point(_)=>Some((feature.id,"Region point-light color, intensity and range use Bevy lights; native attenuation-start and unresolved controls remain uninterpreted")),
            _=>None,
        }).chain(self.attachments.iter().filter_map(|definition| match definition.definition.child().map(|value| &value.appearance) {
            Some(SemanticPlacementAppearance::World(_)) => Some((definition.feature.id, "WMO placement appearance is retained on ModelAttachment; interior direct illumination, region-derived ambient lighting, linked lights and placement flags require application interpretation, without changing child albedo or scene lighting")),
            _ => None,
        }))
    }
    pub fn compile(
        model: &Model<'_>,
        children: &HashMap<ChildModelKey, ChildModelBinding>,
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
                NodeFeature::Light(light) => result.lights.push((
                    feature,
                    light.clone(),
                    model
                        .node(value.node)
                        .expect("validated feature node")
                        .transform
                        .pivot,
                )),
                NodeFeature::Camera(ModelCamera::LookAt(camera)) => result
                    .cameras
                    .push((feature, CameraDefinition::LookAt((**camera).clone()))),
                NodeFeature::Camera(ModelCamera::Node(camera)) => result
                    .cameras
                    .push((feature, CameraDefinition::Node((**camera).clone()))),
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
                    model: value.child().and_then(|_| {
                        children
                            .get(&ChildModelKey::attachment(id))
                            .map(|binding| binding.model.clone())
                    }),
                    placement: value.mount(id, feature.node).map(|mut placement| {
                        placement.coordinates = children
                            .get(&ChildModelKey::attachment(id))
                            .map_or(CoordinatePolicy::default(), |binding| binding.coordinates);
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
                .map(|(feature, light, origin)| (remap(*feature), light.clone(), *origin)),
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
        sample: &SampledAnimation<'_, '_>,
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
        owned: bool,
        actor: Option<&ActorFrame>,
    ) -> RuntimeFeatureInstances {
        let mut instances = RuntimeFeatureInstances::default();
        for (feature, definition, origin) in &self.lights {
            let Some(&parent) = parents
                .get(&feature.id)
                .or_else(|| nodes.get(&feature.node))
            else {
                continue;
            };
            let mut entity = commands.spawn((
                *feature,
                ModelOwnedRenderEntity,
                ModelLight {
                    definition: definition.clone(),
                    enabled: true,
                    ambient_color: Vec3::ZERO,
                    ambient_intensity: 0.,
                },
                // The carrying node maps model coordinates; features originate at
                // its authored pivot, as in PoseContext::sample_feature_node.
                Transform::from_translation(*origin),
                Visibility::default(),
                ChildOf(parent),
            ));
            let kind = match definition {
                SemanticLight::SingleEmission(light) => light.kind,
                SemanticLight::AmbientDiffuse(light) => match light.kind {
                    IlluminationKind::Point => EmissionKind::Point,
                    IlluminationKind::Directional => EmissionKind::Directional,
                    _ => EmissionKind::Ambient,
                },
                SemanticLight::World(light) => match light.kind {
                    WorldLightKind::Point => EmissionKind::Point,
                    WorldLightKind::Spot => EmissionKind::Spot,
                    WorldLightKind::Directional => EmissionKind::Directional,
                    _ => EmissionKind::Ambient,
                },
                SemanticLight::Point(_) => EmissionKind::Point,
            };
            match kind {
                EmissionKind::Point => {
                    entity.insert(PointLight {
                        intensity: 0.,
                        ..Default::default()
                    });
                }
                EmissionKind::Directional => {
                    entity.insert(DirectionalLight {
                        illuminance: 0.,
                        ..Default::default()
                    });
                }
                EmissionKind::Spot => {
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
            let mut admitted = false;
            if let (Some(model), Some(placement)) = (&definition.model, definition.placement) {
                if let Ok(lease) = actor.map(ActorFrame::reserve).transpose() {
                    if owned {
                        entity.insert(ParticleModelSubtree);
                    }
                    if let Some(lease) = lease {
                        entity.insert(lease);
                    }
                    entity.insert((
                        ModelInstance(model.clone()),
                        ChildCoordinates {
                            coordinates: placement.coordinates,
                            parent_basis: definition.parent_basis,
                        },
                    ));
                    admitted = true;
                }
            }
            let entity = entity.id();
            if owned && admitted {
                queue_model_birth(commands, entity);
            }
            instances.attachments.push(entity);
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
                    SemanticLight::SingleEmission(light) => (
                        sample.animated(&light.emission.color),
                        sample.animated(&light.emission.intensity),
                        sample.animated(&light.attenuation[1]),
                        1.,
                        light.shadows.casting.unwrap_or(false),
                        if light.kind == EmissionKind::Ambient {
                            sample.animated(&light.emission.color)
                        } else {
                            Vec3::from_array([0.; 3])
                        },
                        if light.kind == EmissionKind::Ambient {
                            sample.animated(&light.emission.intensity)
                        } else {
                            0.
                        },
                        light.cone.each_ref().map(|v| sample.animated(v)),
                    ),
                    SemanticLight::AmbientDiffuse(light) => {
                        let activation = match &light.activation {
                            LightActivation::Enabled(enabled) => {
                                if sample.animated(enabled) {
                                    1.
                                } else {
                                    0.
                                }
                            }
                            LightActivation::Visibility(visibility) => sample.animated(visibility),
                        };
                        (
                            sample.animated(&light.diffuse.color),
                            sample.animated(&light.diffuse.intensity),
                            sample.animated(&light.attenuation[1]),
                            activation,
                            light.shadows.casting.unwrap_or(false),
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
                            Vec3::new(color[0], color[1], color[2]),
                            if ambient { 0. } else { intensity },
                            if light.use_attenuation {
                                light.attenuation[1]
                            } else {
                                0.
                            },
                            1.,
                            modern.is_some_and(|value| value.shadows),
                            if ambient {
                                Vec3::new(color[0], color[1], color[2])
                            } else {
                                Vec3::ZERO
                            },
                            if ambient { intensity } else { 0. },
                            light.cone.to_array(),
                        )
                    }
                    SemanticLight::Point(light) => (
                        Vec3::new(
                            light.color[0] as f32,
                            light.color[1] as f32,
                            light.color[2] as f32,
                        ) / 255.,
                        light.intensity,
                        light.attenuation[1],
                        1.,
                        false,
                        Vec3::ZERO,
                        0.,
                        [0.; 2],
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
        let (eye, target, projection, roll, visibility) = match definition {
            CameraDefinition::LookAt(camera) => {
                let offset = sample.animated(&camera.eye_offset);
                let target = camera.target + sample.animated(&camera.target_offset);
                let visibility = camera
                    .visibility
                    .as_ref()
                    .map_or(1., |value| sample.animated(value));
                (
                    node.transform_point3(offset),
                    target,
                    camera
                        .projection
                        .map_field_of_view(|angle| sample.animated(angle)),
                    sample.animated(&camera.roll),
                    visibility,
                )
            }
            CameraDefinition::Node(camera) => {
                let eye = node.transform_point3(Vec3::ZERO);
                let target = camera
                    .target
                    .as_ref()
                    .map(|v| sample.animated(v))
                    .unwrap_or_else(|| eye + node.transform_vector3(-Vec3::Z));
                (
                    eye,
                    target,
                    camera
                        .projection
                        .map_field_of_view(|angle| sample.animated(angle)),
                    0.,
                    1.,
                )
            }
        };
        let fov = projection.vertical_field_of_view(aspect);
        let direction = (target - eye).try_normalize()?;
        Some(SampledCamera {
            eye,
            target,
            up: Quat::from_axis_angle(direction, roll) * model_up,
            vertical_field_of_view: fov,
            clipping: projection.clipping.to_array(),
            visibility,
        })
    }
}

#[cfg(test)]
mod tests {
    use wc3::model::PropertyValues as SlotValues;

    use super::*;

    #[test]
    fn compiled_attachments_own_their_resolved_child_declaration() {
        let slot_values = SlotValues::default();
        let compiled = {
            let path = b"child.mdx".to_vec();
            let mut draft = Model::<wc3::model::profiles::Universal>::builder(ModelMetadata::Mdlx(
                Default::default(),
            ));
            let resource =
                draft.add_model_resource(ResourceLocator::Path(std::borrow::Cow::Borrowed(&path)));
            let node = draft.add_named_node("child").unwrap();
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
            let model = {
                draft.import_values(&slot_values).unwrap();
                draft.finish()
            }
            .unwrap();
            RuntimeFeatures::compile(&model, &HashMap::new())
        };
        assert!(
            matches!(compiled.attachments[0].resource.as_ref(), Some(ResourceLocator::Path(std::borrow::Cow::Owned(path))) if path == b"child.mdx")
        );
    }
}

#[cfg(test)]
mod imported_light_tests {}

type AttachmentParentData = (Entity, &'static ModelRig, &'static Playback);
type AttachmentChildData = (
    &'static mut AttachmentState,
    Option<&'static mut Playback>,
    &'static mut Visibility,
);

/// Shared activation/restart sampling for ordinary and owner-ordered model roots.
#[derive(SystemParam)]
pub(crate) struct AttachmentClocks<'w, 's, F: QueryFilter + 'static = ()> {
    queries: ParamSet<
        'w,
        's,
        (
            Query<'static, 'static, AttachmentParentData, F>,
            Query<'static, 'static, AttachmentChildData>,
        ),
    >,
    pending: Local<'s, Vec<(Entity, f32, AttachmentClock)>>,
}
pub(crate) type OwnedAttachmentClocks<'w, 's> =
    AttachmentClocks<'w, 's, With<ParticleModelSubtree>>;
impl<F: QueryFilter + 'static> AttachmentClocks<'_, '_, F> {
    /// Sample the current parent clock before its pose evaluation overwrites activation state.
    pub(crate) fn restart(&mut self, root: Entity) {
        self.pending.clear();
        {
            let parents = self.queries.p0();
            let Ok((_, instance, playback)) = parents.get(root) else {
                return;
            };
            let model = instance.model();
            if model.features.attachments.is_empty() {
                return;
            }
            let Ok(selection) = playback.selection() else {
                return;
            };
            let Ok(sample) = model.animation.sample(selection) else {
                return;
            };
            for (definition, entity) in model
                .features
                .attachments
                .iter()
                .zip(&instance.feature_instances().attachments)
            {
                self.pending.push((
                    *entity,
                    definition.visibility(&sample),
                    AttachmentClock {
                        clip: playback.clip,
                        time: playback.clip_time,
                    },
                ));
            }
        }
        let mut attachments = self.queries.p1();
        for &(entity, activation, clock) in self.pending.iter() {
            let Ok((mut point, playback, mut visibility)) = attachments.get_mut(entity) else {
                continue;
            };
            if let Some(mut playback) = playback {
                observe_attachment_clock(&mut point, &mut playback, activation, clock);
            } else {
                // A queued model request has no clock yet; retain activation now
                // so its first ready population cannot flash a hidden attachment.
                point.clock = Some(clock);
                point.visibility = activation;
            }
            let next = if activation > 0. {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *visibility != next {
                *visibility = next;
            }
        }
    }
}

/// Ordinary roots sample activation before child pose evaluation.
/// Particle-owned roots use the same operation after their current clocks are projected.
pub(crate) fn restart_attachments(
    mut clocks: AttachmentClocks<'_, '_, Without<ParticleModelSubtree>>,
    mut roots: Local<Vec<Entity>>,
) {
    roots.clear();
    roots.extend(clocks.queries.p0().iter().map(|(entity, _, _)| entity));
    for &root in roots.iter() {
        clocks.restart(root);
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
