//! Transactional graph construction and editing.
use crate::model::ids::Scope;
use crate::model::profiles::{Profile, Universal};
use crate::model::{
    AttachedFeature, Basis, Bounds, Channel, Clip, ClipId, Clock, ClockId, Controller,
    ControllerId, DrawAppearanceSchema, DrawSubsetId, Error, Event, EventId, FeatureId, InstanceId,
    LengthUnit, Look, LookBinding, LookId, Material, MaterialId, Mesh, MeshId, MeshInstance, Model,
    ModelId, Node, NodeId, SharedClock, Skin, SkinId, SystemId, Texture, TextureId,
    TextureTransform, TextureTransformId,
};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
/// A draft that accepts forward node relationships and publishes only a validated graph.
pub struct ModelBuilder<'a, P: Profile = Universal> {
    pub(super) model: Model<'a, P>,
    pub(super) pending_nodes: BTreeSet<NodeId>,
    pub(super) pending_features: BTreeMap<FeatureId, NodeId>,
    pub(super) pending_materials: BTreeSet<MaterialId>,
    pub(super) pending_controllers: BTreeSet<ControllerId>,
}
impl<'a, P: Profile> ModelBuilder<'a, P> {
    pub(crate) fn from_model(model: Model<'a, P>) -> Self {
        Self {
            model,
            pending_nodes: BTreeSet::new(),
            pending_features: BTreeMap::new(),
            pending_materials: BTreeSet::new(),
            pending_controllers: BTreeSet::new(),
        }
    }
    /// Identity reserved for this model before publication.
    pub fn id(&self) -> ModelId {
        self.model.id()
    }
    /// Resolves a texture defined in the draft model.
    pub fn texture(&self, id: TextureId) -> Option<&Texture<'a, P>> {
        self.model.texture(id)
    }

    /// Begins an unpublished graph using the explicit payload vocabulary `P`.
    pub(crate) fn with_profile(name: impl Into<Cow<'a, str>>) -> Self {
        let scope = Scope::default();
        let id = scope.allocate();
        Self {
            model: Model {
                textures: BTreeMap::new(),
                scope,
                id,
                name: name.into(),
                basis: Basis::default(),
                units: LengthUnit::default(),
                bounds: P::default_bounds(),
                metadata: Default::default(),
                systems: BTreeMap::new(),
                timeline_channels: Vec::new(),
                timeline_events: Vec::new(),
                shared_channels: Vec::new(),
                shared_events: Vec::new(),
                features: BTreeMap::new(),
                appearance: Default::default(),
                nodes: BTreeMap::new(),
                meshes: BTreeMap::new(),
                materials: BTreeMap::new(),
                clips: BTreeMap::new(),
                skins: BTreeMap::new(),
                instances: BTreeMap::new(),
                clocks: BTreeMap::new(),
                controllers: BTreeMap::new(),
                texture_transforms: BTreeMap::new(),
            },
            pending_nodes: BTreeSet::new(),
            pending_features: BTreeMap::new(),
            pending_materials: BTreeSet::new(),
            pending_controllers: BTreeSet::new(),
        }
    }
    /// Reserves a hierarchy node identity for forward references; publication requires its definition.
    pub fn reserve_node(&mut self) -> NodeId {
        let id = self.model.scope.allocate();
        self.pending_nodes.insert(id);
        id
    }
    /// Adds an independently shared texture-coordinate transform.
    pub fn add_texture_transform(
        &mut self,
        value: TextureTransform,
    ) -> Result<TextureTransformId, Error> {
        let id = self.model.scope.allocate();
        self.model.texture_transforms.insert(id, value);
        Ok(id)
    }
    /// Replaces a shared texture-coordinate transform belonging to this graph.
    pub fn set_texture_transform(
        &mut self,
        id: TextureTransformId,
        value: TextureTransform,
    ) -> Result<(), Error> {
        let target = self
            .model
            .texture_transforms
            .get_mut(&id)
            .ok_or_else(|| Error::new("transform", "unknown identity"))?;
        *target = value;
        Ok(())
    }
    /// Reserves a animation controller identity for forward references; publication requires its definition.
    pub fn reserve_controller(&mut self) -> ControllerId {
        let id = self.model.scope.allocate();
        self.pending_controllers.insert(id);
        id
    }
    /// Adds a animation controller to the draft; publication validates its values and relationships.
    pub fn add_controller(&mut self, value: Controller<'a, P>) -> Result<ControllerId, Error> {
        let id = self.model.scope.allocate();
        self.model.controllers.insert(id, value);
        Ok(id)
    }
    /// Defines or replaces a animation controller belonging to this draft.
    pub fn set_controller(
        &mut self,
        id: ControllerId,
        value: Controller<'a, P>,
    ) -> Result<(), Error> {
        if !self.model.scope.contains(id)
            || (!self.pending_controllers.remove(&id) && !self.model.controllers.contains_key(&id))
        {
            return Err(Error::new("controller", "unknown identity"));
        }
        self.model.controllers.insert(id, value);
        Ok(())
    }
    /// Adds a curve to its actual programme owner.
    pub fn add_controller_channel(
        &mut self,
        id: ControllerId,
        channel: Channel<'a, P>,
    ) -> Result<(), Error> {
        self.model
            .controllers
            .get_mut(&id)
            .ok_or_else(|| Error::new("controller", "unknown identity"))?
            .channels
            .push(channel);
        Ok(())
    }
    /// Adds an event occurrence to its actual programme owner.
    pub fn add_controller_event(
        &mut self,
        id: ControllerId,
        event: Event<'a>,
    ) -> Result<EventId, Error> {
        let programme = self
            .model
            .controllers
            .get_mut(&id)
            .ok_or_else(|| Error::new("controller", "unknown identity"))?;
        let event_id = self.model.scope.allocate();
        programme.events.insert(event_id, event);
        Ok(event_id)
    }
    /// Replaces playback behavior directly on an existing clip.
    pub fn set_clip_controls(
        &mut self,
        id: ClipId,
        value: impl Into<P::Clip<'a>>,
    ) -> Result<(), Error> {
        if !self.model.clips.contains_key(&id) {
            return Err(Error::new("clip.controls", "unknown owner"));
        }
        self.model
            .clips
            .get_mut(&id)
            .expect("checked owner")
            .controls = value.into();
        Ok(())
    }
    /// Replaces controller behavior directly on an existing controller.
    pub fn set_controller_controls(
        &mut self,
        id: ControllerId,
        value: impl Into<P::Controller<'a>>,
    ) -> Result<(), Error> {
        if !self.model.controllers.contains_key(&id) {
            return Err(Error::new("controller.controls", "unknown owner"));
        }
        self.model
            .controllers
            .get_mut(&id)
            .expect("checked owner")
            .controls = value.into();
        Ok(())
    }
    /// Defines or replaces a hierarchy node belonging to this draft.
    pub fn set_node(&mut self, id: NodeId, node: Node<'a, P>) -> Result<(), Error> {
        if !self.model.scope.contains(id)
            || (!self.pending_nodes.remove(&id) && !self.model.nodes.contains_key(&id))
        {
            return Err(Error::new("node", "unknown identity"));
        }
        self.model.nodes.insert(id, node);
        Ok(())
    }
    /// Changes the parent link; publication rejects missing parents and hierarchy cycles.
    pub fn set_node_parent(&mut self, id: NodeId, parent: Option<NodeId>) -> Result<(), Error> {
        self.model
            .nodes
            .get_mut(&id)
            .ok_or_else(|| Error::new("node", "unknown identity"))?
            .parent = parent;
        Ok(())
    }
    /// Replaces a surface with a payload allowed by this graph vocabulary.
    pub fn set_material_surface(
        &mut self,
        id: MaterialId,
        surface: P::Surface<'a>,
    ) -> Result<(), Error> {
        self.model
            .materials
            .get_mut(&id)
            .ok_or_else(|| Error::new("material", "unknown identity"))?
            .surface = surface;
        Ok(())
    }
    /// Borrows the draft for inspection; references may remain unresolved until publication.
    pub fn model(&self) -> &Model<'a, P> {
        &self.model
    }
    /// Declares the coordinate axes and handedness of the draft.
    pub fn set_basis(&mut self, basis: Basis) {
        self.model.basis = basis;
    }
    /// Declares the length unit; publication rejects invalid scale factors.
    pub fn set_units(&mut self, units: LengthUnit) {
        self.model.units = units;
    }
    /// Sets or clears the authored model bounds.
    pub fn set_bounds(&mut self, bounds: Option<Bounds>) {
        self.model.bounds = bounds;
    }
    /// Adds a channel authored on the absolute model timeline.
    pub fn add_timeline_channel(&mut self, channel: Channel<'a, P>) -> Result<(), Error> {
        if channel.clock() != Clock::Timeline {
            return Err(Error::new(
                "timeline_channels.clock",
                "expected the absolute authoring clock",
            ));
        }
        self.model.timeline_channels.push(channel);
        Ok(())
    }
    /// Replaces an absolute-timeline channel by its authoring order.
    pub fn set_timeline_channel(
        &mut self,
        index: usize,
        channel: Channel<'a, P>,
    ) -> Result<(), Error> {
        if channel.clock() != Clock::Timeline {
            return Err(Error::new(
                "timeline_channels.clock",
                "expected the absolute authoring clock",
            ));
        }
        let slot = self
            .model
            .timeline_channels
            .get_mut(index)
            .ok_or_else(|| Error::new("timeline_channels", "unknown index"))?;
        *slot = channel;
        Ok(())
    }
    /// Adds an event occurrence on the absolute model timeline.
    pub fn add_timeline_event(&mut self, event: Event<'a>) -> Result<(), Error> {
        if event.clock != Clock::Timeline {
            return Err(Error::new(
                "timeline_events.clock",
                "expected the absolute authoring clock",
            ));
        }
        self.model.timeline_events.push(event);
        Ok(())
    }
    /// Replaces an absolute-timeline event occurrence by its authoring order.
    pub fn set_timeline_event(&mut self, index: usize, event: Event<'a>) -> Result<(), Error> {
        if event.clock != Clock::Timeline {
            return Err(Error::new(
                "timeline_events.clock",
                "expected the absolute authoring clock",
            ));
        }
        let slot = self
            .model
            .timeline_events
            .get_mut(index)
            .ok_or_else(|| Error::new("timeline_events", "unknown index"))?;
        *slot = event;
        Ok(())
    }
    /// Adds a channel that must use a shared repeating clock.
    pub fn add_shared_channel(&mut self, channel: Channel<'a, P>) -> Result<(), Error> {
        if !matches!(channel.clock(), Clock::Shared(_)) {
            return Err(Error::new(
                "shared_channels.clock",
                "shared channel requires a shared clock",
            ));
        }
        self.model.shared_channels.push(channel);
        Ok(())
    }
    /// Replaces an existing shared-clock channel without appending a duplicate program.
    pub fn set_shared_channel(
        &mut self,
        index: usize,
        channel: Channel<'a, P>,
    ) -> Result<(), Error> {
        if !matches!(channel.clock(), Clock::Shared(_)) {
            return Err(Error::new(
                "shared_channels.clock",
                "shared channel requires a shared clock",
            ));
        }
        let slot = self
            .model
            .shared_channels
            .get_mut(index)
            .ok_or_else(|| Error::new("shared_channels", "unknown index"))?;
        *slot = channel;
        Ok(())
    }
    /// Replaces an existing shared-clock event occurrence.
    pub fn set_shared_event(&mut self, index: usize, event: Event<'a>) -> Result<(), Error> {
        if !matches!(event.clock, Clock::Shared(_)) {
            return Err(Error::new(
                "shared_events.clock",
                "shared event requires a shared clock",
            ));
        }
        let slot = self
            .model
            .shared_events
            .get_mut(index)
            .ok_or_else(|| Error::new("shared_events", "unknown index"))?;
        *slot = event;
        Ok(())
    }
    /// Adds an event occurrence that must use a shared repeating clock.
    pub fn add_shared_event(&mut self, event: Event<'a>) -> Result<(), Error> {
        if !matches!(event.clock, Clock::Shared(_)) {
            return Err(Error::new(
                "shared_events.clock",
                "a shared event requires a shared clock",
            ));
        }
        self.model.shared_events.push(event);
        Ok(())
    }
    /// Adds a attached behavior to the draft; publication validates its values and relationships.
    pub fn add_feature(
        &mut self,
        node: NodeId,
        payload: P::Feature<'a>,
    ) -> Result<FeatureId, Error> {
        let id = self.model.scope.allocate();
        self.model
            .features
            .insert(id, AttachedFeature { node, payload });
        Ok(id)
    }
    /// Reserves a attached behavior identity for forward references; publication requires its definition.
    pub fn reserve_feature(&mut self, node: NodeId) -> FeatureId {
        let id = self.model.scope.allocate();
        self.pending_features.insert(id, node);
        id
    }
    /// Defines or replaces a attached behavior belonging to this draft.
    pub fn set_feature(&mut self, id: FeatureId, payload: P::Feature<'a>) -> Result<(), Error> {
        if let Some(node) = self.pending_features.remove(&id) {
            self.model
                .features
                .insert(id, AttachedFeature { node, payload });
            return Ok(());
        }
        self.model
            .features
            .get_mut(&id)
            .ok_or_else(|| Error::new("feature", "unknown feature identity"))?
            .payload = payload;
        Ok(())
    }
    /// Replaces model-wide authoring metadata.
    pub fn set_metadata(&mut self, value: impl Into<P::ModelMetadata<'a>>) {
        self.model.metadata = value.into();
    }
    /// Adds an independently authored scene system.
    pub fn add_system(&mut self, value: P::System<'a>) -> Result<SystemId, Error> {
        let id = self.model.scope.allocate();
        self.model.systems.insert(id, value);
        Ok(id)
    }
    /// Replaces an independently authored scene system belonging to this graph.
    pub fn set_system(&mut self, id: SystemId, value: P::System<'a>) -> Result<(), Error> {
        let target = self
            .model
            .systems
            .get_mut(&id)
            .ok_or_else(|| Error::new("system", "unknown identity"))?;
        *target = value;
        Ok(())
    }
    /// Replaces the complete appearance vocabulary; publication checks its relationships.
    pub fn set_appearance(&mut self, appearance: P::Appearance<'a>) {
        self.model.appearance = appearance;
    }
    /// Adds a material look to the draft; publication validates its values and relationships.
    pub fn add_look(&mut self, look: Look<'a>) -> Result<LookId, Error>
    where
        P::Appearance<'a>: DrawAppearanceSchema<'a>,
    {
        let id = self.model.scope.allocate();
        self.model.appearance.add_look(id, look)?;
        Ok(id)
    }
    /// Selects or clears the default material look.
    pub fn set_default_look(&mut self, look: Option<LookId>) -> Result<(), Error>
    where
        P::Appearance<'a>: DrawAppearanceSchema<'a>,
    {
        self.model.appearance.set_default_look(look)
    }
    /// Replaces a complete look without changing its identity or default selection.
    pub fn set_look(&mut self, id: LookId, look: Look<'a>) -> Result<(), Error>
    where
        P::Appearance<'a>: DrawAppearanceSchema<'a>,
    {
        *self
            .model
            .appearance
            .look_mut(id)
            .ok_or_else(|| Error::new("look", "unknown identity"))? = look;
        Ok(())
    }
    /// Inserts a placement and appends its materials to the default complete look.
    /// Material order must match the placement's draw order. Alternative looks must
    /// be completed separately before publication.
    pub fn add_instance_with_materials(
        &mut self,
        instance: MeshInstance<'a, P>,
        materials: Vec<MaterialId>,
    ) -> Result<InstanceId, Error>
    where
        P::Appearance<'a>: DrawAppearanceSchema<'a>,
    {
        if instance.draws.len() != materials.len() {
            return Err(Error::new(
                "instance.materials",
                "one material is required per draw",
            ));
        }
        let id = self.model.scope.allocate();
        if !instance.draws.is_empty() {
            let bindings = instance
                .draws
                .iter()
                .zip(materials)
                .map(|(draw, material)| LookBinding {
                    instance: id,
                    draw: draw.id(),
                    material,
                });
            if let Some(selected) = self.model.default_look() {
                self.model
                    .appearance
                    .look_mut(selected)
                    .ok_or_else(|| Error::new("appearance.default", "default look is absent"))?
                    .bindings
                    .extend(bindings);
            } else {
                let selected = self.model.scope.allocate();
                let look = Look {
                    name: Cow::Borrowed(""),
                    bindings: bindings.collect(),
                };
                let mut appearance = self.model.appearance.clone();
                appearance.add_look(selected, look)?;
                appearance.set_default_look(Some(selected))?;
                self.model.appearance = appearance;
            }
        }
        self.model.instances.insert(id, instance);
        Ok(id)
    }
    /// Adds a mesh-owned primitive selection to the draft.
    pub fn add_primary_subset(
        &mut self,
        mesh: MeshId,
        first: u32,
        count: u32,
    ) -> Result<DrawSubsetId, Error> {
        Ok(self
            .model
            .meshes
            .get_mut(&mesh)
            .ok_or_else(|| Error::new("mesh.subsets", "mesh is absent"))?
            .add_primary_subset(first, count))
    }
    /// Replaces a draft mesh; publication validates its data and relationships.
    pub fn set_mesh(&mut self, id: MeshId, value: Mesh<'a, P>) -> Result<(), Error> {
        let slot = self
            .model
            .meshes
            .get_mut(&id)
            .ok_or_else(|| Error::new("mesh", "unknown identity"))?;
        *slot = value;
        Ok(())
    }
    /// Replaces a draft material; publication validates its data and relationships.
    pub fn reserve_material(&mut self) -> MaterialId {
        let id = self.model.scope.allocate();
        self.pending_materials.insert(id);
        id
    }
    /// Defines or replaces a surface belonging to this draft.
    pub fn set_material(&mut self, id: MaterialId, value: Material<'a, P>) -> Result<(), Error> {
        if self.pending_materials.remove(&id) {
            self.model.materials.insert(id, value);
            return Ok(());
        }
        let slot = self
            .model
            .materials
            .get_mut(&id)
            .ok_or_else(|| Error::new("material", "unknown identity"))?;
        *slot = value;
        Ok(())
    }
    /// Replaces a draft texture; publication validates its data and relationships.
    pub fn set_texture(&mut self, id: TextureId, value: Texture<'a, P>) -> Result<(), Error> {
        let slot = self
            .model
            .textures
            .get_mut(&id)
            .ok_or_else(|| Error::new("texture", "unknown identity"))?;
        *slot = value;
        Ok(())
    }
    /// Replaces a draft clip; publication validates its data and relationships.
    pub fn set_clip(&mut self, id: ClipId, value: Clip<'a, P>) -> Result<(), Error> {
        let slot = self
            .model
            .clips
            .get_mut(&id)
            .ok_or_else(|| Error::new("clip", "unknown identity"))?;
        *slot = value;
        Ok(())
    }
    /// Replaces a draft skin; publication validates its data and relationships.
    pub fn set_skin(&mut self, id: SkinId, value: Skin<'a, P>) -> Result<(), Error> {
        let slot = self
            .model
            .skins
            .get_mut(&id)
            .ok_or_else(|| Error::new("skin", "unknown identity"))?;
        *slot = value;
        Ok(())
    }
    /// Replaces a draft instance; publication validates its data and relationships.
    pub fn set_instance(
        &mut self,
        id: InstanceId,
        value: MeshInstance<'a, P>,
    ) -> Result<(), Error> {
        let slot = self
            .model
            .instances
            .get_mut(&id)
            .ok_or_else(|| Error::new("instance", "unknown identity"))?;
        *slot = value;
        Ok(())
    }
    /// Replaces a draft clock; publication validates its data and relationships.
    pub fn set_clock(&mut self, id: ClockId, value: SharedClock) -> Result<(), Error> {
        let slot = self
            .model
            .clocks
            .get_mut(&id)
            .ok_or_else(|| Error::new("clock", "unknown identity"))?;
        *slot = value;
        Ok(())
    }
    /// Adds a hierarchy node to the draft; publication validates its values and relationships.
    pub fn add_node(&mut self, value: Node<'a, P>) -> Result<NodeId, Error> {
        let id = self.model.scope.allocate();
        self.model.nodes.insert(id, value);
        Ok(id)
    }
    /// Adds a shared geometry to the draft; publication validates its values and relationships.
    pub fn add_mesh(&mut self, value: Mesh<'a, P>) -> Result<MeshId, Error> {
        let id = self.model.scope.allocate();
        self.model.meshes.insert(id, value);
        Ok(id)
    }
    /// Adds a surface to the draft; publication validates its values and relationships.
    pub fn add_material(&mut self, value: Material<'a, P>) -> Result<MaterialId, Error> {
        let id = self.model.scope.allocate();
        self.model.materials.insert(id, value);
        Ok(id)
    }
    /// Adds a texture resource to the draft; publication validates its values and relationships.
    pub fn add_texture(&mut self, value: Texture<'a, P>) -> Result<TextureId, Error> {
        let id = self.model.scope.allocate();
        self.model.textures.insert(id, value);
        Ok(id)
    }
    /// Adds a playback clip to the draft; publication validates its values and relationships.
    pub fn add_clip(&mut self, value: Clip<'a, P>) -> Result<ClipId, Error> {
        let id = self.model.scope.allocate();
        self.model.clips.insert(id, value);
        Ok(id)
    }
    /// Adds a joint binding to the draft; publication validates its values and relationships.
    pub fn add_skin(&mut self, value: Skin<'a, P>) -> Result<SkinId, Error> {
        let id = self.model.scope.allocate();
        self.model.skins.insert(id, value);
        Ok(id)
    }
    /// Adds a geometry placement to the draft; publication validates its values and relationships.
    pub fn add_instance(&mut self, value: MeshInstance<'a, P>) -> Result<InstanceId, Error> {
        let id = self.model.scope.allocate();
        self.model.instances.insert(id, value);
        Ok(id)
    }
    /// Adds a repeating animation clock to the draft; publication validates its values and relationships.
    pub fn add_clock(&mut self, value: SharedClock) -> Result<ClockId, Error> {
        let id = self.model.scope.allocate();
        self.model.clocks.insert(id, value);
        Ok(id)
    }
    /// Appends a typed channel to an existing playback clip.
    pub fn add_channel(&mut self, id: ClipId, channel: Channel<'a, P>) -> Result<(), Error> {
        self.model
            .clips
            .get_mut(&id)
            .ok_or_else(|| Error::new("clip", "unknown identity"))?
            .channels
            .push(channel);
        Ok(())
    }
    /// Appends an event occurrence to an existing playback clip.
    pub fn add_event(&mut self, id: ClipId, event: Event<'a>) -> Result<(), Error> {
        self.model
            .clips
            .get_mut(&id)
            .ok_or_else(|| Error::new("clip", "unknown identity"))?
            .events
            .push(event);
        Ok(())
    }
    /// Publishes an immutable graph after validating all values, identities, and relationships.
    pub fn finish(self) -> Result<Model<'a, P>, Error> {
        if !self.pending_controllers.is_empty() {
            return Err(Error::new(
                "controllers",
                "reserved controllers have no definition",
            ));
        }
        if !self.pending_materials.is_empty() {
            return Err(Error::new(
                "materials",
                "reserved materials have no definition",
            ));
        }
        if !self.pending_features.is_empty() {
            return Err(Error::new(
                "features",
                "reserved features have no definition",
            ));
        }
        if !self.pending_nodes.is_empty() {
            return Err(Error::new("nodes", "reserved nodes have no definition"));
        }
        self.model.validate()?;
        Ok(self.model)
    }
}
impl<'a> ModelBuilder<'a> {
    /// Begins an empty graph accepting every built-in logical family.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self::with_profile(name)
    }
}

impl<'a, P: Profile> ModelBuilder<'a, P> {
    /// Removes this node from the draft; publication rejects references left pointing to it.
    pub fn remove_node(&mut self, id: NodeId) -> Option<Node<'a, P>> {
        self.pending_nodes.remove(&id);
        self.model.nodes.remove(&id)
    }
    /// Removes this feature from the draft; publication rejects references left pointing to it.
    pub fn remove_feature(&mut self, id: FeatureId) -> Option<AttachedFeature<'a, P>> {
        self.pending_features.remove(&id);
        self.model.features.remove(&id)
    }
    /// Removes this mesh from the draft; publication rejects references left pointing to it.
    pub fn remove_mesh(&mut self, id: MeshId) -> Option<Mesh<'a, P>> {
        self.model.meshes.remove(&id)
    }
    /// Removes this material from the draft; publication rejects references left pointing to it.
    pub fn remove_material(&mut self, id: MaterialId) -> Option<Material<'a, P>> {
        self.pending_materials.remove(&id);
        self.model.materials.remove(&id)
    }
    /// Removes this texture from the draft; publication rejects references left pointing to it.
    pub fn remove_texture(&mut self, id: TextureId) -> Option<Texture<'a, P>> {
        self.model.textures.remove(&id)
    }
    /// Removes this clip from the draft; publication rejects references left pointing to it.
    pub fn remove_clip(&mut self, id: ClipId) -> Option<Clip<'a, P>> {
        self.model.clips.remove(&id)
    }
    /// Removes this skin from the draft; publication rejects references left pointing to it.
    pub fn remove_skin(&mut self, id: SkinId) -> Option<Skin<'a, P>> {
        self.model.skins.remove(&id)
    }
    /// Removes this instance from the draft; publication rejects references left pointing to it.
    pub fn remove_instance(&mut self, id: InstanceId) -> Option<MeshInstance<'a, P>> {
        self.model.instances.remove(&id)
    }
    /// Removes this clock from the draft; publication rejects references left pointing to it.
    pub fn remove_clock(&mut self, id: ClockId) -> Option<SharedClock> {
        self.model.clocks.remove(&id)
    }
    /// Removes this controller from the draft; publication rejects references left pointing to it.
    pub fn remove_controller(&mut self, id: ControllerId) -> Option<Controller<'a, P>> {
        self.pending_controllers.remove(&id);
        self.model.controllers.remove(&id)
    }
    /// Removes this transform from the draft; publication rejects references left pointing to it.
    pub fn remove_texture_transform(&mut self, id: TextureTransformId) -> Option<TextureTransform> {
        self.model.texture_transforms.remove(&id)
    }
    /// Removes this system from the draft; publication rejects references left pointing to it.
    pub fn remove_system(&mut self, id: SystemId) -> Option<P::System<'a>> {
        self.model.systems.remove(&id)
    }
    /// Removes this look from the draft; publication rejects references left pointing to it.
    pub fn remove_look(&mut self, id: LookId) -> Option<Look<'a>>
    where
        P::Appearance<'a>: DrawAppearanceSchema<'a>,
    {
        self.model.appearance.remove_look(id)
    }
}
