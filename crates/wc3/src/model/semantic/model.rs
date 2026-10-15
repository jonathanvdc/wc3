//! Checked logical graphs and transactional construction.
use crate::model::semantic::ids::Scope;
use crate::model::semantic::profiles::{Profile, Universal};
use crate::model::semantic::DrawId;
use crate::model::semantic::{
    AppearanceSchema as _, AttachedFeature, Basis, Bounds, Channel, Clip, ClipId, ClockId,
    Controller, ControllerId, Error, Event, EventId, FeatureId, InstanceId, LengthUnit, Look,
    LookId, Material, MaterialId, Mesh, MeshId, MeshInstance, ModelId, Node, NodeId, Skin, SkinId,
    SystemId, TextureTransform, TextureTransformId, Time,
};
use crate::model::semantic::{Resources, ResourcesBuilder, Texture, TextureId};
use std::borrow::Cow;
use std::collections::BTreeMap;
/// A repeating clock shared by animation channels.
#[derive(Clone, Debug)]
pub struct SharedClock {
    /// Positive duration of one clock cycle.
    pub period: Time,
}
/// An encoding-independent model. All public access is immutable; edits are checked before publication.
#[derive(Clone, Debug)]
pub struct Model<'a, P: Profile = Universal> {
    resources: Resources<'a, P>,
    scope: Scope,
    id: ModelId,
    name: Cow<'a, str>,
    basis: Basis,
    units: LengthUnit,
    bounds: Option<Bounds>,
    metadata: P::ModelMetadata<'a>,
    systems: BTreeMap<SystemId, P::System<'a>>,
    timeline_channels: Vec<Channel<'a, P>>,
    timeline_events: Vec<Event<'a>>,
    shared_channels: Vec<Channel<'a, P>>,
    shared_events: Vec<Event<'a>>,
    features: BTreeMap<FeatureId, AttachedFeature<'a, P>>,
    appearance: P::Appearance<'a>,
    nodes: BTreeMap<NodeId, Node<'a, P>>,
    meshes: BTreeMap<MeshId, Mesh<'a, P>>,
    materials: BTreeMap<MaterialId, Material<'a, P>>,
    clips: BTreeMap<ClipId, Clip<'a, P>>,
    skins: BTreeMap<SkinId, Skin<'a, P>>,
    instances: BTreeMap<InstanceId, MeshInstance<'a, P>>,
    clocks: BTreeMap<ClockId, SharedClock>,
    controllers: BTreeMap<ControllerId, Controller<'a, P>>,
    texture_transforms: BTreeMap<TextureTransformId, TextureTransform>,
}
impl<'a, P: Profile> Model<'a, P> {
    /// Texture definitions owned by this model.
    pub fn resources(&self) -> &Resources<'a, P> {
        &self.resources
    }
    /// Resolves a texture identity in this model.
    pub fn texture(&self, id: TextureId) -> Option<&Texture<'a, P>> {
        self.resources.texture(id)
    }
    /// Iterates texture definitions in identity order.
    pub fn textures(&self) -> impl ExactSizeIterator<Item = (TextureId, &Texture<'a, P>)> {
        self.resources.textures()
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
    /// Checks resource edits against the model before publishing them.
    pub fn edit_resources<R>(
        &mut self,
        edit: impl FnOnce(&mut ResourcesBuilder<'a, P>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut resources = self.resources.draft();
        let result = edit(&mut resources)?;
        let mut draft = self.clone();
        draft.resources = resources.finish();
        draft.validate()?;
        *self = draft;
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
    pub fn timeline_channels(&self) -> &[Channel<'a, P>] {
        &self.timeline_channels
    }
    /// Event occurrences authored on the absolute model timeline.
    pub fn timeline_events(&self) -> &[Event<'a>] {
        &self.timeline_events
    }
    /// Channels governed by shared clocks rather than an individual playback clip.
    pub fn shared_channels(&self) -> &[Channel<'a, P>] {
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
    /// Iterates independent scene systems in stable graph identity order.
    pub fn systems(&self) -> impl ExactSizeIterator<Item = (SystemId, &P::System<'a>)> {
        self.systems.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves an independent scene system belonging to this graph.
    pub fn system(&self, id: SystemId) -> Option<&P::System<'a>> {
        self.systems.get(&id)
    }
    /// Iterates shared texture-coordinate transforms in stable graph identity order.
    pub fn texture_transforms(
        &self,
    ) -> impl ExactSizeIterator<Item = (TextureTransformId, &TextureTransform)> {
        self.texture_transforms
            .iter()
            .map(|(id, value)| (*id, value))
    }
    /// Resolves a shared texture-coordinate transform belonging to this graph.
    pub fn texture_transform(&self, id: TextureTransformId) -> Option<&TextureTransform> {
        self.texture_transforms.get(&id)
    }
    /// Iterates animation controller definitions in stable graph identity order.
    pub fn controllers(&self) -> impl ExactSizeIterator<Item = (ControllerId, &Controller<'a, P>)> {
        self.controllers.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a animation controller; returns `None` when the identity is absent from this graph.
    pub fn controller(&self, id: ControllerId) -> Option<&Controller<'a, P>> {
        self.controllers.get(&id)
    }
    /// Owner-local playback behavior of a clip belonging to this graph.
    pub fn clip_controls(&self, id: ClipId) -> Option<&P::Clip<'a>> {
        Some(&self.clip(id)?.controls)
    }
    /// Owner-local behavior of a controller belonging to this graph.
    pub fn controller_controls(&self, id: ControllerId) -> Option<&P::Controller<'a>> {
        Some(&self.controller(id)?.controls)
    }
    /// Iterates attached behavior definitions in stable graph identity order.
    pub fn features(&self) -> impl ExactSizeIterator<Item = (FeatureId, &AttachedFeature<'a, P>)> {
        self.features.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a attached behavior; returns `None` when the identity is absent from this graph.
    pub fn feature(&self, id: FeatureId) -> Option<&AttachedFeature<'a, P>> {
        self.features.get(&id)
    }
    /// Iterates the independently attached behaviors anchored to a hierarchy node.
    pub fn node_features(
        &self,
        node: NodeId,
    ) -> impl Iterator<Item = (FeatureId, &P::Feature<'a>)> {
        self.features
            .iter()
            .filter(move |(_, value)| value.node == node)
            .map(|(id, value)| (*id, &value.payload))
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
    /// Iterates hierarchy node definitions in stable graph identity order.
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = (NodeId, &Node<'a, P>)> {
        self.nodes.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a hierarchy node; returns `None` when the identity is absent from this graph.
    pub fn node(&self, id: NodeId) -> Option<&Node<'a, P>> {
        self.nodes.get(&id)
    }
    /// Iterates shared geometry definitions in stable graph identity order.
    pub fn meshes(&self) -> impl ExactSizeIterator<Item = (MeshId, &Mesh<'a, P>)> {
        self.meshes.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a shared geometry; returns `None` when the identity is absent from this graph.
    pub fn mesh(&self, id: MeshId) -> Option<&Mesh<'a, P>> {
        self.meshes.get(&id)
    }
    /// Iterates surface definitions in stable graph identity order.
    pub fn materials(&self) -> impl ExactSizeIterator<Item = (MaterialId, &Material<'a, P>)> {
        self.materials.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a surface; returns `None` when the identity is absent from this graph.
    pub fn material(&self, id: MaterialId) -> Option<&Material<'a, P>> {
        self.materials.get(&id)
    }
    /// Resolves a clip's playback program through its alias chain.
    pub fn clip_program(&self, id: ClipId) -> Option<&Clip<'a, P>> {
        let mut clip = self.clip(id)?;
        while let Some(target) = clip.alias {
            clip = self.clip(target)?;
        }
        Some(clip)
    }
    /// Iterates playback clip definitions in stable graph identity order.
    pub fn clips(&self) -> impl ExactSizeIterator<Item = (ClipId, &Clip<'a, P>)> {
        self.clips.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a playback clip; returns `None` when the identity is absent from this graph.
    pub fn clip(&self, id: ClipId) -> Option<&Clip<'a, P>> {
        self.clips.get(&id)
    }
    /// Iterates joint binding definitions in stable graph identity order.
    pub fn skins(&self) -> impl ExactSizeIterator<Item = (SkinId, &Skin<'a, P>)> {
        self.skins.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a joint binding; returns `None` when the identity is absent from this graph.
    pub fn skin(&self, id: SkinId) -> Option<&Skin<'a, P>> {
        self.skins.get(&id)
    }
    /// Iterates geometry placement definitions in stable graph identity order.
    pub fn instances(&self) -> impl ExactSizeIterator<Item = (InstanceId, &MeshInstance<'a, P>)> {
        self.instances.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a geometry placement; returns `None` when the identity is absent from this graph.
    pub fn instance(&self, id: InstanceId) -> Option<&MeshInstance<'a, P>> {
        self.instances.get(&id)
    }
    /// Iterates repeating animation clock definitions in stable graph identity order.
    pub fn clocks(&self) -> impl ExactSizeIterator<Item = (ClockId, &SharedClock)> {
        self.clocks.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a repeating animation clock; returns `None` when the identity is absent from this graph.
    pub fn clock(&self, id: ClockId) -> Option<&SharedClock> {
        self.clocks.get(&id)
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
