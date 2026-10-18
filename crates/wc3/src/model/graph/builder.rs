//! Transactional graph construction and editing.
use crate::model::ids::Scope;
use crate::model::profiles::{Profile, Universal};
use crate::model::{
    Animation, AnimationId, AttachedFeature, Clip, ClipId, ClockId, Controller, ControllerId,
    DrawAppearanceSchema, DrawSubsetId, DynamicTrack, Error, Event, EventId, FeatureId, InstanceId,
    Look, LookBinding, LookId, Material, MaterialId, Mesh, MeshId, MeshInstance, Model,
    ModelMetadata, Node, NodeId, SharedClock, Skin, SkinId, SystemId, Texture,
};
use crate::model::{
    BoneOverrideResourceId, EffectResourceId, ImageResourceId, ModelResourceId, PhysicsResourceId,
};
use crate::model::{Construct, Property, PropertyValue, PropertyValues};
use crate::model::{Dependencies, PathResource, Table};
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
    /// Reuses an equal image declaration when importing native indirection tables.
    pub(crate) fn intern_image_resource(&mut self, value: P::ImageResource<'a>) -> ImageResourceId
    where
        P::ImageResource<'a>: PartialEq,
    {
        if let Some((id, _)) = self
            .model
            .dependencies
            .images
            .iter()
            .find(|(_, existing)| **existing == value)
        {
            return id;
        }
        self.add_image_resource(value)
    }
    /// Adds an independent image dependency declaration.
    pub fn add_image_resource(&mut self, value: P::ImageResource<'a>) -> ImageResourceId {
        self.model.dependencies.images.add(&self.model.scope, value)
    }
    /// Resolves a image dependency in the draft.
    pub fn image_resource(&self, id: ImageResourceId) -> Option<&P::ImageResource<'a>> {
        self.model.image_resource(id)
    }
    /// Replaces a shared image declaration without changing its identity.
    pub fn set_image_resource(
        &mut self,
        id: ImageResourceId,
        value: P::ImageResource<'a>,
    ) -> Result<(), Error> {
        self.model
            .dependencies
            .images
            .replace(id, value, "resources")
    }
    /// Removes a declaration; publication rejects any remaining references to it.
    pub fn remove_image_resource(&mut self, id: ImageResourceId) -> Option<P::ImageResource<'a>> {
        self.model.dependencies.images.remove(&id)
    }
    /// Adds an independent model dependency declaration.
    pub fn add_model_resource(&mut self, value: P::ModelResource<'a>) -> ModelResourceId {
        self.model.dependencies.models.add(&self.model.scope, value)
    }
    /// Resolves a model dependency in the draft.
    pub fn model_resource(&self, id: ModelResourceId) -> Option<&P::ModelResource<'a>> {
        self.model.model_resource(id)
    }
    /// Replaces a shared model declaration without changing its identity.
    pub fn set_model_resource(
        &mut self,
        id: ModelResourceId,
        value: P::ModelResource<'a>,
    ) -> Result<(), Error> {
        self.model
            .dependencies
            .models
            .replace(id, value, "resources")
    }
    /// Removes a declaration; publication rejects any remaining references to it.
    pub fn remove_model_resource(&mut self, id: ModelResourceId) -> Option<P::ModelResource<'a>> {
        self.model.dependencies.models.remove(&id)
    }
    /// Adds an independent effect dependency declaration.
    pub fn add_effect_resource(&mut self, value: P::EffectResource<'a>) -> EffectResourceId {
        self.model
            .dependencies
            .effects
            .add(&self.model.scope, value)
    }
    /// Resolves a effect dependency in the draft.
    pub fn effect_resource(&self, id: EffectResourceId) -> Option<&P::EffectResource<'a>> {
        self.model.effect_resource(id)
    }
    /// Replaces a shared effect declaration without changing its identity.
    pub fn set_effect_resource(
        &mut self,
        id: EffectResourceId,
        value: P::EffectResource<'a>,
    ) -> Result<(), Error> {
        self.model
            .dependencies
            .effects
            .replace(id, value, "resources")
    }
    /// Removes a declaration; publication rejects any remaining references to it.
    pub fn remove_effect_resource(
        &mut self,
        id: EffectResourceId,
    ) -> Option<P::EffectResource<'a>> {
        self.model.dependencies.effects.remove(&id)
    }
    /// Adds an independent physics dependency declaration.
    pub fn add_physics_resource(&mut self, value: P::PhysicsResource<'a>) -> PhysicsResourceId {
        self.model
            .dependencies
            .physics
            .add(&self.model.scope, value)
    }
    /// Resolves a physics dependency in the draft.
    pub fn physics_resource(&self, id: PhysicsResourceId) -> Option<&P::PhysicsResource<'a>> {
        self.model.physics_resource(id)
    }
    /// Replaces a shared physics declaration without changing its identity.
    pub fn set_physics_resource(
        &mut self,
        id: PhysicsResourceId,
        value: P::PhysicsResource<'a>,
    ) -> Result<(), Error> {
        self.model
            .dependencies
            .physics
            .replace(id, value, "resources")
    }
    /// Removes a declaration; publication rejects any remaining references to it.
    pub fn remove_physics_resource(
        &mut self,
        id: PhysicsResourceId,
    ) -> Option<P::PhysicsResource<'a>> {
        self.model.dependencies.physics.remove(&id)
    }
    /// Adds an independent boneoverride dependency declaration.
    pub fn add_bone_override_resource(
        &mut self,
        value: P::BoneOverrideResource<'a>,
    ) -> BoneOverrideResourceId {
        self.model
            .dependencies
            .bone_overrides
            .add(&self.model.scope, value)
    }
    /// Resolves a boneoverride dependency in the draft.
    pub fn bone_override_resource(
        &self,
        id: BoneOverrideResourceId,
    ) -> Option<&P::BoneOverrideResource<'a>> {
        self.model.bone_override_resource(id)
    }
    /// Replaces a shared boneoverride declaration without changing its identity.
    pub fn set_bone_override_resource(
        &mut self,
        id: BoneOverrideResourceId,
        value: P::BoneOverrideResource<'a>,
    ) -> Result<(), Error> {
        self.model
            .dependencies
            .bone_overrides
            .replace(id, value, "resources")
    }
    /// Removes a declaration; publication rejects any remaining references to it.
    pub fn remove_bone_override_resource(
        &mut self,
        id: BoneOverrideResourceId,
    ) -> Option<P::BoneOverrideResource<'a>> {
        self.model.dependencies.bone_overrides.remove(&id)
    }
    /// Creates a root node and allocates its transform and visibility slots.
    pub fn new_node(&mut self, name: impl Into<Cow<'a, str>>) -> Node<'a, P> {
        Node::named(name, &mut self.model.values)
    }
    /// Adds a root node with identity transform and visible authored state.
    pub fn add_named_node(&mut self, name: impl Into<Cow<'a, str>>) -> Result<NodeId, Error> {
        let node = self.new_node(name);
        self.add_node(node)
    }
    /// Allocates an independently animatable value in this model.
    pub fn add_value<T: PropertyValue>(&mut self, value: T) -> Property<T> {
        self.model.values.add(value)
    }
    /// Constructs a definition and allocates its animation slots.
    pub fn construct<T: Construct>(&mut self) -> T {
        T::construct(&mut self.model.values)
    }
    /// Adopts separately constructed values while preserving their scoped identities.
    pub fn import_values(&mut self, values: &PropertyValues) -> Result<(), Error> {
        self.model.values.extend(values)
    }
    /// Mutable authored constants while constructing or editing a model.
    pub fn values_mut(&mut self) -> &mut PropertyValues {
        &mut self.model.values
    }

    /// Begins an unpublished graph using the explicit payload vocabulary `P`.
    pub(crate) fn with_profile(metadata: impl Into<P::ModelMetadata<'a>>) -> Self {
        let scope = Scope::default();
        Self {
            model: Model {
                values: PropertyValues::scoped(scope.clone()),
                dependencies: Dependencies::default(),
                scope,
                metadata: metadata.into(),
                systems: Table::default(),
                animations: Table::default(),
                global_animations: Vec::new(),
                features: Table::default(),
                appearance: Default::default(),
                nodes: Table::default(),
                meshes: Table::default(),
                materials: Table::default(),
                clips: Table::default(),
                skins: Table::default(),
                instances: Table::default(),
                clocks: Table::default(),
                controllers: Table::default(),
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
    /// Reserves a animation controller identity for forward references; publication requires its definition.
    pub fn reserve_controller(&mut self) -> ControllerId {
        let id = self.model.scope.allocate();
        self.pending_controllers.insert(id);
        id
    }
    /// Adds a animation controller to the draft; publication validates its values and relationships.
    pub fn add_controller(&mut self, value: Controller<'a, P>) -> Result<ControllerId, Error> {
        Ok(self.model.controllers.add(&self.model.scope, value))
    }
    /// Defines or replaces a animation controller belonging to this draft.
    pub fn set_controller(
        &mut self,
        id: ControllerId,
        value: Controller<'a, P>,
    ) -> Result<(), Error> {
        if !self.model.scope.contains(id)
            || (!self.pending_controllers.remove(&id) && !self.model.controllers.contains_key(id))
        {
            return Err(Error::new("controller", "unknown identity"));
        }
        self.model.controllers.insert(id, value);
        Ok(())
    }
    /// Adds a reusable animation definition.
    pub fn add_animation(&mut self, value: Animation<'a>) -> AnimationId {
        self.model.animations.add(&self.model.scope, value)
    }
    /// Replaces an animation while preserving all selections of its identity.
    pub fn set_animation(&mut self, id: AnimationId, value: Animation<'a>) -> Result<(), Error> {
        self.model.animations.replace(id, value, "animation")
    }
    /// Removes an animation; publication rejects remaining selections.
    pub fn remove_animation(&mut self, id: AnimationId) -> Option<Animation<'a>> {
        self.model.animations.remove(&id)
    }
    /// Selects a reusable animation independently of clip playback.
    pub fn add_global_animation(&mut self, id: AnimationId) -> Result<(), Error> {
        if !self.model.animations.contains_key(id) {
            return Err(Error::new("animation", "unknown identity"));
        }
        self.model.global_animations.push(id);
        Ok(())
    }
    /// Replaces the ordered model-level selection.
    pub fn set_global_animations(&mut self, ids: Vec<AnimationId>) {
        self.model.global_animations = ids;
    }
    /// Appends a curve to its reusable owner.
    pub fn add_animation_track(
        &mut self,
        id: AnimationId,
        track: DynamicTrack<'a>,
    ) -> Result<(), Error> {
        self.model
            .animations
            .get_mut(&id)
            .ok_or_else(|| Error::new("animation", "unknown identity"))?
            .tracks
            .push(track);
        Ok(())
    }
    /// Adds an identified event occurrence to its reusable owner.
    pub fn add_animation_event(
        &mut self,
        id: AnimationId,
        event: Event<'a>,
    ) -> Result<EventId, Error> {
        let animation = self
            .model
            .animations
            .get_mut(&id)
            .ok_or_else(|| Error::new("animation", "unknown identity"))?;
        let event_id = self.model.scope.allocate();
        animation.events.insert(event_id, event);
        Ok(event_id)
    }
    fn controller_animation(&mut self, id: ControllerId) -> Result<AnimationId, Error> {
        let controller = self
            .model
            .controllers
            .get(id)
            .ok_or_else(|| Error::new("controller", "unknown identity"))?;
        if let Some(&animation) = controller.animations.first() {
            return Ok(animation);
        }
        let animation = self.add_animation(Animation::default());
        self.model
            .controllers
            .get_mut(&id)
            .unwrap()
            .animations
            .push(animation);
        Ok(animation)
    }
    /// Appends a curve to the controller's first animation, creating one if empty.
    pub fn add_controller_track(
        &mut self,
        id: ControllerId,
        track: DynamicTrack<'a>,
    ) -> Result<(), Error> {
        let animation = self.controller_animation(id)?;
        self.add_animation_track(animation, track)
    }
    /// Appends an occurrence to the controller's first animation, creating one if empty.
    pub fn add_controller_event(
        &mut self,
        id: ControllerId,
        event: Event<'a>,
    ) -> Result<EventId, Error> {
        let animation = self.controller_animation(id)?;
        self.add_animation_event(animation, event)
    }
    /// Replaces playback behavior directly on an existing clip.
    pub fn set_clip_controls(
        &mut self,
        id: ClipId,
        value: impl Into<P::Clip<'a>>,
    ) -> Result<(), Error> {
        if !self.model.clips.contains_key(id) {
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
        if !self.model.controllers.contains_key(id) {
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
            || (!self.pending_nodes.remove(&id) && !self.model.nodes.contains_key(id))
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
    fn global_animation(&mut self) -> AnimationId {
        if let Some(&id) = self.model.global_animations.first() {
            return id;
        }
        let id = self.add_animation(Animation::default());
        self.model.global_animations.push(id);
        id
    }
    /// Appends a model-level curve; its time source is authoritative.
    pub fn add_global_track(&mut self, track: DynamicTrack<'a>) -> Result<(), Error> {
        let id = self.global_animation();
        self.add_animation_track(id, track)
    }
    /// Appends a model-level event; its time source is authoritative.
    pub fn add_global_event(&mut self, event: Event<'a>) -> Result<EventId, Error> {
        let id = self.global_animation();
        self.add_animation_event(id, event)
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
    /// Edits model-wide declarations in this unpublished draft.
    pub fn metadata_mut(&mut self) -> &mut P::ModelMetadata<'a> {
        &mut self.model.metadata
    }
    /// Replaces model-wide declarations.
    pub fn set_metadata(&mut self, value: impl Into<P::ModelMetadata<'a>>) {
        self.model.metadata = value.into();
    }
    /// Adds an independently authored scene system.
    pub fn add_system(&mut self, value: P::System<'a>) -> Result<SystemId, Error> {
        Ok(self.model.systems.add(&self.model.scope, value))
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
        self.model.meshes.replace(id, value, "mesh")
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
        self.model.materials.replace(id, value, "material")
    }
    /// Replaces a draft clip; publication validates its data and relationships.
    pub fn set_clip(&mut self, id: ClipId, value: Clip<'a, P>) -> Result<(), Error> {
        self.model.clips.replace(id, value, "clip")
    }
    /// Replaces a draft skin; publication validates its data and relationships.
    pub fn set_skin(&mut self, id: SkinId, value: Skin<'a, P>) -> Result<(), Error> {
        self.model.skins.replace(id, value, "skin")
    }
    /// Replaces a draft instance; publication validates its data and relationships.
    pub fn set_instance(
        &mut self,
        id: InstanceId,
        value: MeshInstance<'a, P>,
    ) -> Result<(), Error> {
        self.model.instances.replace(id, value, "instance")
    }
    /// Replaces a draft clock; publication validates its data and relationships.
    pub fn set_clock(&mut self, id: ClockId, value: SharedClock) -> Result<(), Error> {
        self.model.clocks.replace(id, value, "clock")
    }
    /// Adds a hierarchy node to the draft; publication validates its values and relationships.
    pub fn add_node(&mut self, value: Node<'a, P>) -> Result<NodeId, Error> {
        Ok(self.model.nodes.add(&self.model.scope, value))
    }
    /// Adds a shared geometry to the draft; publication validates its values and relationships.
    pub fn add_mesh(&mut self, value: Mesh<'a, P>) -> Result<MeshId, Error> {
        Ok(self.model.meshes.add(&self.model.scope, value))
    }
    /// Adds a surface to the draft; publication validates its values and relationships.
    pub fn add_material(&mut self, value: Material<'a, P>) -> Result<MaterialId, Error> {
        Ok(self.model.materials.add(&self.model.scope, value))
    }
    /// Declares a path image and a texture using its default interpretation.
    pub fn add_file_texture(&mut self, path: &'a str) -> Result<Texture, Error>
    where
        P::ImageResource<'a>: From<PathResource<'a>>,
    {
        let resource =
            self.add_image_resource(PathResource::Path(Cow::Borrowed(path.as_bytes())).into());
        Ok(Texture::new(resource))
    }
    /// Adds a playback clip to the draft; publication validates its values and relationships.
    pub fn add_clip(&mut self, value: Clip<'a, P>) -> Result<ClipId, Error> {
        Ok(self.model.clips.add(&self.model.scope, value))
    }
    /// Adds a joint binding to the draft; publication validates its values and relationships.
    pub fn add_skin(&mut self, value: Skin<'a, P>) -> Result<SkinId, Error> {
        Ok(self.model.skins.add(&self.model.scope, value))
    }
    /// Adds a geometry placement to the draft; publication validates its values and relationships.
    pub fn add_instance(&mut self, value: MeshInstance<'a, P>) -> Result<InstanceId, Error> {
        Ok(self.model.instances.add(&self.model.scope, value))
    }
    /// Adds a repeating animation clock to the draft; publication validates its values and relationships.
    pub fn add_clock(&mut self, value: SharedClock) -> Result<ClockId, Error> {
        Ok(self.model.clocks.add(&self.model.scope, value))
    }
    fn clip_animation(&mut self, id: ClipId) -> Result<AnimationId, Error> {
        let clip = self
            .model
            .clips
            .get(id)
            .ok_or_else(|| Error::new("clip", "unknown identity"))?;
        if let Some(&animation) = clip.animations.first() {
            return Ok(animation);
        }
        let animation = self.add_animation(Animation::default());
        self.model
            .clips
            .get_mut(&id)
            .unwrap()
            .animations
            .push(animation);
        Ok(animation)
    }
    /// Appends a curve to the clip's first animation, creating one if empty.
    pub fn add_track(&mut self, id: ClipId, track: DynamicTrack<'a>) -> Result<(), Error> {
        let animation = self.clip_animation(id)?;
        self.add_animation_track(animation, track)
    }
    /// Appends an occurrence to the clip's first animation, creating one if empty.
    pub fn add_event(&mut self, id: ClipId, event: Event<'a>) -> Result<(), Error> {
        let animation = self.clip_animation(id)?;
        self.add_animation_event(animation, event)?;
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
    /// Begins an empty universal graph with explicit model-wide declarations.
    pub fn new(metadata: impl Into<ModelMetadata<'a>>) -> Self {
        Self::with_profile(metadata)
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
