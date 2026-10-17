//! Checked logical graphs and transactional construction.
use crate::model::ids::Scope;
use crate::model::profiles::{Profile, Universal};
use crate::model::DrawId;
use crate::model::{Animatable, Animated, ValueSlot, Values};
use crate::model::{
    AppearanceSchema as _, AttachedFeature, Basis, Bounds, Channel, Clip, ClipId, ClockId,
    Controller, ControllerId, Error, Event, EventId, FeatureId, InstanceId, LengthUnit, Look,
    LookId, Material, MaterialId, Mesh, MeshId, MeshInstance, ModelId, Node, NodeId, Skin, SkinId,
    SystemId, Time,
};
use crate::model::{
    BoneOverrideResourceId, EffectResourceId, ImageResourceId, ModelResourceId, PhysicsResourceId,
};
use crate::model::{
    ClipObject, ClockObject, ControllerObject, FeatureObject, InstanceObject, MaterialObject,
    MeshObject, NodeObject, SkinObject, SystemObject,
};
use crate::model::{Dependencies, Link, Table, Texture, ValidateLinks as _};
use std::borrow::Cow;
use std::collections::HashSet;
/// A repeating or frozen clock shared by animation channels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SharedClock {
    /// Samples time zero and has no traversed event interval.
    Frozen,
    /// Repeats over a strictly positive period.
    Repeating {
        /// Positive duration of one clock cycle.
        period: Time,
    },
}
impl SharedClock {
    /// Native duration; a frozen clock returns zero without inventing a cycle.
    pub fn period(self) -> Time {
        match self {
            Self::Frozen => Time::default(),
            Self::Repeating { period } => period,
        }
    }
}
/// An encoding-independent model. All public access is immutable; edits are checked before publication.
#[derive(Clone, Debug)]
pub struct Model<'a, P: Profile = Universal> {
    values: Values,
    dependencies: Dependencies<'a, P>,
    scope: Scope,
    id: ModelId,
    name: Cow<'a, str>,
    basis: Basis,
    units: LengthUnit,
    bounds: Option<Bounds>,
    metadata: P::ModelMetadata<'a>,
    systems: Table<SystemObject, P::System<'a>>,
    timeline_channels: Vec<Channel<'a>>,
    timeline_events: Vec<Event<'a>>,
    shared_channels: Vec<Channel<'a>>,
    shared_events: Vec<Event<'a>>,
    features: Table<FeatureObject, AttachedFeature<'a, P>>,
    appearance: P::Appearance<'a>,
    nodes: Table<NodeObject, Node<'a, P>>,
    meshes: Table<MeshObject, Mesh<'a, P>>,
    materials: Table<MaterialObject, Material<'a, P>>,
    clips: Table<ClipObject, Clip<'a, P>>,
    skins: Table<SkinObject, Skin<'a, P>>,
    instances: Table<InstanceObject, MeshInstance<'a, P>>,
    clocks: Table<ClockObject, SharedClock>,
    controllers: Table<ControllerObject, Controller<'a, P>>,
}
impl<'a, P: Profile> Model<'a, P> {
    /// Authored constants for every typed animation slot.
    pub fn values(&self) -> &Values {
        &self.values
    }
    /// Whether any authored program writes this slot.
    pub fn is_animated<T: Animatable>(&self, slot: Animated<T>) -> bool {
        self.channels()
            .any(|channel| channel.target() == slot.slot())
            || match slot.slot() {
                ValueSlot::Scalar(id) => self
                    .values
                    .boolean_source(id)
                    .is_some_and(|source| self.is_animated(source)),
                _ => false,
            }
    }
    /// Channels in every authored program, independent of playback selection.
    pub fn channels(&self) -> impl Iterator<Item = &Channel<'a>> {
        self.timeline_channels
            .iter()
            .chain(&self.shared_channels)
            .chain(self.clips.values().flat_map(|clip| &clip.channels))
            .chain(
                self.controllers
                    .values()
                    .flat_map(|controller| &controller.channels),
            )
    }
    /// External dependency declarations owned by this model.
    pub fn dependencies(&self) -> &Dependencies<'a, P> {
        &self.dependencies
    }
    /// Resolves a model-owned image dependency declaration.
    pub fn image_resource(&self, id: ImageResourceId) -> Option<&P::ImageResource<'a>> {
        self.dependencies.images.get(id)
    }
    /// Resolves a model-owned model dependency declaration.
    pub fn model_resource(&self, id: ModelResourceId) -> Option<&P::ModelResource<'a>> {
        self.dependencies.models.get(id)
    }
    /// Resolves a model-owned effect dependency declaration.
    pub fn effect_resource(&self, id: EffectResourceId) -> Option<&P::EffectResource<'a>> {
        self.dependencies.effects.get(id)
    }
    /// Resolves a model-owned physics dependency declaration.
    pub fn physics_resource(&self, id: PhysicsResourceId) -> Option<&P::PhysicsResource<'a>> {
        self.dependencies.physics.get(id)
    }
    /// Resolves a model-owned boneoverride dependency declaration.
    pub fn bone_override_resource(
        &self,
        id: BoneOverrideResourceId,
    ) -> Option<&P::BoneOverrideResource<'a>> {
        self.dependencies.bone_overrides.get(id)
    }
    /// Complete sampled image values in material/effect inputs, allocated slots, and curves.
    /// Repeated values are returned once; no shared editable texture objects are created.
    pub fn sampled_images(&self) -> Vec<Texture> {
        let mut images = Vec::new();
        let mut seen = HashSet::new();
        let mut visit = |link| {
            if let Link::SampledImage(image) = link {
                if seen.insert(image) {
                    images.push(image);
                }
            }
        };
        for material in self.materials.values() {
            material.surface.visit_links(&mut visit);
        }
        for feature in self.features.values() {
            feature.payload.visit_links(&mut visit);
        }
        self.metadata.visit_links(&mut visit);
        self.appearance.visit_links(&mut visit);
        for node in self.nodes.values() {
            node.controls.visit_links(&mut visit);
        }
        for skin in self.skins.values() {
            skin.binding.visit_links(&mut visit);
        }
        for mesh in self.meshes.values() {
            mesh.controls.visit_links(&mut visit);
            mesh.geometry.visit_links(&mut visit);
        }
        for instance in self.instances.values() {
            instance.controls.visit_links(&mut visit);
            for draw in &instance.draws {
                draw.controls.visit_links(&mut visit);
            }
        }
        for system in self.systems.values() {
            system.visit_links(&mut visit);
        }
        self.values.visit_values(&mut visit);
        for channel in self.timeline_channels.iter().chain(&self.shared_channels) {
            channel.visit_links(&mut visit);
        }
        for clip in self.clips.values() {
            clip.controls.visit_links(&mut visit);
            for channel in clip.channels() {
                channel.visit_links(&mut visit);
            }
        }
        for controller in self.controllers.values() {
            controller.controls.visit_links(&mut visit);
            for channel in &controller.channels {
                channel.visit_links(&mut visit);
            }
        }
        images
    }
    /// Edits a draft and publishes it only after the complete model validates.
    pub fn edit<R>(
        &mut self,
        edit: impl FnOnce(&mut ModelBuilder<'a, P>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut draft = ModelBuilder::from_model(self.clone());
        let result = edit(&mut draft)?;
        *self = draft.finish()?;
        Ok(result)
    }
    /// Stable identity of this model and its model-level property targets.
    pub fn id(&self) -> ModelId {
        self.id
    }
    /// Begins a draft whose payloads are constrained by `P`.
    pub fn builder(name: impl Into<Cow<'a, str>>) -> ModelBuilder<'a, P> {
        ModelBuilder::with_profile(name)
    }
    /// Creates an empty model, checking the profile's default authoring vocabulary.
    pub fn empty(name: impl Into<Cow<'a, str>>) -> Result<Model<'a, P>, Error> {
        ModelBuilder::<P>::with_profile(name).finish()
    }
    /// Display name of the model.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Declared axis directions and handedness of model coordinates.
    pub fn basis(&self) -> Basis {
        self.basis
    }
    /// Declared model length unit, or unspecified when the source does not declare one.
    pub fn units(&self) -> LengthUnit {
        self.units
    }
    /// Authored model bounds, when present.
    pub fn bounds(&self) -> Option<Bounds> {
        self.bounds
    }
    /// Resolves a programme's event occurrence by its actual graph identity.
    pub fn event(&self, id: EventId) -> Option<&Event<'a>> {
        self.controllers
            .values()
            .find_map(|controller| controller.events.get(&id))
    }
    /// Channels authored on the absolute model timeline, independent of playback intervals.
    pub fn timeline_channels(&self) -> &[Channel<'a>] {
        &self.timeline_channels
    }
    /// Event occurrences authored on the absolute model timeline.
    pub fn timeline_events(&self) -> &[Event<'a>] {
        &self.timeline_events
    }
    /// Channels governed by shared clocks rather than an individual playback clip.
    pub fn shared_channels(&self) -> &[Channel<'a>] {
        &self.shared_channels
    }
    /// Event occurrences governed by shared clocks.
    pub fn shared_events(&self) -> &[Event<'a>] {
        &self.shared_events
    }
    /// Model-wide authoring metadata selected by this vocabulary.
    pub fn metadata(&self) -> &P::ModelMetadata<'a> {
        &self.metadata
    }
    /// Independent scene system definitions in stable identity order.
    pub fn systems(&self) -> &Table<SystemObject, P::System<'a>> {
        &self.systems
    }
    /// Resolves an independent scene system belonging to this graph.
    pub fn system(&self, id: SystemId) -> Option<&P::System<'a>> {
        self.systems.get(id)
    }
    /// Animation controller definitions in stable identity order.
    pub fn controllers(&self) -> &Table<ControllerObject, Controller<'a, P>> {
        &self.controllers
    }
    /// Resolves a animation controller; returns `None` when the identity is absent from this graph.
    pub fn controller(&self, id: ControllerId) -> Option<&Controller<'a, P>> {
        self.controllers.get(id)
    }
    /// Owner-local playback behavior of a clip belonging to this graph.
    pub fn clip_controls(&self, id: ClipId) -> Option<&P::Clip<'a>> {
        Some(&self.clip(id)?.controls)
    }
    /// Owner-local behavior of a controller belonging to this graph.
    pub fn controller_controls(&self, id: ControllerId) -> Option<&P::Controller<'a>> {
        Some(&self.controller(id)?.controls)
    }
    /// Attached behavior definitions in stable identity order.
    pub fn features(&self) -> &Table<FeatureObject, AttachedFeature<'a, P>> {
        &self.features
    }
    /// Resolves a attached behavior; returns `None` when the identity is absent from this graph.
    pub fn feature(&self, id: FeatureId) -> Option<&AttachedFeature<'a, P>> {
        self.features.get(id)
    }
    /// Iterates the independently attached behaviors anchored to a hierarchy node.
    pub fn node_features(
        &self,
        node: NodeId,
    ) -> impl Iterator<Item = (FeatureId, &P::Feature<'a>)> {
        self.features
            .iter()
            .filter(move |(_, value)| value.node == node)
            .map(|(id, value)| (id, &value.payload))
    }
    /// Complete material assignment vocabulary selected by the profile.
    pub fn appearance(&self) -> &P::Appearance<'a> {
        &self.appearance
    }
    /// Iterates material look definitions in stable graph identity order.
    pub fn looks(&self) -> impl ExactSizeIterator<Item = (LookId, &Look<'a>)> {
        self.appearance.looks()
    }
    /// Resolves a material look; returns `None` when the identity is absent from this graph.
    pub fn look(&self, id: LookId) -> Option<&Look<'a>> {
        self.appearance.look(id)
    }
    /// Authored material look used when a caller does not select another look.
    pub fn default_look(&self) -> Option<LookId> {
        self.appearance.default_look()
    }
    /// Resolves the material of one draw under an authored look or the default look.
    pub fn material_for_draw(
        &self,
        instance: InstanceId,
        draw: DrawId,
        look: Option<LookId>,
    ) -> Result<MaterialId, Error> {
        self.instance(instance)
            .and_then(|instance| instance.draw(draw))
            .ok_or_else(|| Error::new("draw", "instance draw is absent"))?;
        let id = look
            .or(self.default_look())
            .ok_or_else(|| Error::new("look", "no material look selected"))?;
        let look = self
            .look(id)
            .ok_or_else(|| Error::new("look", "look is absent or belongs to another graph"))?;
        look.bindings
            .iter()
            .find(|binding| binding.instance == instance && binding.draw == draw)
            .map(|binding| binding.material)
            .ok_or_else(|| Error::new("look.bindings", "draw has no material assignment"))
    }
    /// Hierarchy node definitions in stable identity order.
    pub fn nodes(&self) -> &Table<NodeObject, Node<'a, P>> {
        &self.nodes
    }
    /// Resolves a hierarchy node; returns `None` when the identity is absent from this graph.
    pub fn node(&self, id: NodeId) -> Option<&Node<'a, P>> {
        self.nodes.get(id)
    }
    /// Shared geometry definitions in stable identity order.
    pub fn meshes(&self) -> &Table<MeshObject, Mesh<'a, P>> {
        &self.meshes
    }
    /// Resolves a shared geometry; returns `None` when the identity is absent from this graph.
    pub fn mesh(&self, id: MeshId) -> Option<&Mesh<'a, P>> {
        self.meshes.get(id)
    }
    /// Surface definitions in stable identity order.
    pub fn materials(&self) -> &Table<MaterialObject, Material<'a, P>> {
        &self.materials
    }
    /// Resolves a surface; returns `None` when the identity is absent from this graph.
    pub fn material(&self, id: MaterialId) -> Option<&Material<'a, P>> {
        self.materials.get(id)
    }
    /// Resolves a clip's playback program through its alias chain.
    pub fn clip_program(&self, id: ClipId) -> Option<&Clip<'a, P>> {
        let mut clip = self.clip(id)?;
        while let Some(target) = clip.alias {
            clip = self.clip(target)?;
        }
        Some(clip)
    }
    /// Playback clip definitions in stable identity order.
    pub fn clips(&self) -> &Table<ClipObject, Clip<'a, P>> {
        &self.clips
    }
    /// Resolves a playback clip; returns `None` when the identity is absent from this graph.
    pub fn clip(&self, id: ClipId) -> Option<&Clip<'a, P>> {
        self.clips.get(id)
    }
    /// Joint binding definitions in stable identity order.
    pub fn skins(&self) -> &Table<SkinObject, Skin<'a, P>> {
        &self.skins
    }
    /// Resolves a joint binding; returns `None` when the identity is absent from this graph.
    pub fn skin(&self, id: SkinId) -> Option<&Skin<'a, P>> {
        self.skins.get(id)
    }
    /// Geometry placement definitions in stable identity order.
    pub fn instances(&self) -> &Table<InstanceObject, MeshInstance<'a, P>> {
        &self.instances
    }
    /// Resolves a geometry placement; returns `None` when the identity is absent from this graph.
    pub fn instance(&self, id: InstanceId) -> Option<&MeshInstance<'a, P>> {
        self.instances.get(id)
    }
    /// Shared animation clock definitions in stable identity order.
    pub fn clocks(&self) -> &Table<ClockObject, SharedClock> {
        &self.clocks
    }
    /// Resolves a shared animation clock; returns `None` when the identity is absent from this graph.
    pub fn clock(&self, id: ClockId) -> Option<&SharedClock> {
        self.clocks.get(id)
    }
}
impl<'a> Model<'a> {
    /// Creates an empty graph accepting every built-in logical family.
    pub fn standalone(name: impl Into<Cow<'a, str>>) -> Model<'a> {
        Self::empty(name).expect("built-in empty vocabulary is valid")
    }
}

mod builder;
pub use builder::ModelBuilder;
mod narrowing;
mod validation;
mod widening;
