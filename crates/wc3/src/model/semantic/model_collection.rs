//! Checked model collections share one resource arena and retain their cardinality in the type.
use crate::model::semantic::profiles::{
    Either, Extends, FeaturePayload as _, Narrows, Profile, Union, Universal,
};
use crate::model::semantic::{
    AppearanceSchema as _, Error, GeometrySchema as _, Link, Model, ModelBuilder, ModelId,
    Resources, ResourcesBuilder, SystemPayload as _, Texture, TextureId, ValidateLinks as _,
};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, Index};

/// The number of models a container can represent.
pub trait Cardinality: Clone + Debug + 'static {
    /// Storage enforcing this cardinality structurally.
    type Storage<T: Clone + Debug>: Clone + Debug;
    /// Stores a single model.
    fn single<T: Clone + Debug>(value: T) -> Self::Storage<T>;
    /// Checks a collection before choosing its storage.
    fn collect<T: Clone + Debug>(values: Vec<T>) -> Result<Self::Storage<T>, Error>;
    /// Inspects the collection without allocation.
    fn as_slice<T: Clone + Debug>(values: &Self::Storage<T>) -> &[T];
    /// Consumes the collection in model order.
    fn into_vec<T: Clone + Debug>(values: Self::Storage<T>) -> Vec<T>;
}
/// Exactly one model, as required by native MDLX, M2, M3, and WMO assets.
#[derive(Clone, Debug)]
pub struct Single;
impl Cardinality for Single {
    type Storage<T: Clone + Debug> = [T; 1];
    fn single<T: Clone + Debug>(value: T) -> [T; 1] {
        [value]
    }
    fn collect<T: Clone + Debug>(values: Vec<T>) -> Result<[T; 1], Error> {
        values
            .try_into()
            .map_err(|_| Error::new("models", "this container requires exactly one model"))
    }
    fn as_slice<T: Clone + Debug>(values: &[T; 1]) -> &[T] {
        values
    }
    fn into_vec<T: Clone + Debug>(values: [T; 1]) -> Vec<T> {
        Vec::from(values)
    }
}
/// An ordered collection of models, including an empty authoring collection.
#[derive(Clone, Debug)]
pub struct Multiple;
impl Cardinality for Multiple {
    type Storage<T: Clone + Debug> = Vec<T>;
    fn single<T: Clone + Debug>(value: T) -> Vec<T> {
        vec![value]
    }
    fn collect<T: Clone + Debug>(values: Vec<T>) -> Result<Vec<T>, Error> {
        Ok(values)
    }
    fn as_slice<T: Clone + Debug>(values: &Vec<T>) -> &[T] {
        values
    }
    fn into_vec<T: Clone + Debug>(values: Vec<T>) -> Vec<T> {
        values
    }
}
/// Proof that widening a container does not exclude its source cardinality.
pub trait ContainsCardinality<Source: Cardinality>: Cardinality {
    /// Embeds the source collection without changing model order.
    fn embed<T: Clone + Debug>(source: Source::Storage<T>) -> Self::Storage<T>;
}
impl<C: Cardinality> ContainsCardinality<C> for C {
    fn embed<T: Clone + Debug>(source: C::Storage<T>) -> C::Storage<T> {
        source
    }
}
impl ContainsCardinality<Single> for Multiple {
    fn embed<T: Clone + Debug>(source: [T; 1]) -> Vec<T> {
        Vec::from(source)
    }
}

/// Models and their authoritative resource arena.
///
/// Models contain resource identities, never a resource handle. A borrowed view
/// supplies the collection context when those identities need resolving.
#[derive(Clone, Debug)]
pub struct ModelCollection<'data, P: Profile = Universal, C: Cardinality = Multiple> {
    models: C::Storage<Model<'data, P>>,
    resources: Resources<'data, P>,
    controls: P::Asset<'data>,
}
/// An unpublished collection that validates models and shared authoring domains together.
///
/// Unlike finishing standalone models independently, this permits relationships
/// to collection controls that are defined after the model drafts are assembled.
pub struct ModelCollectionBuilder<'data, P: Profile = Universal, C: Cardinality = Multiple> {
    models: Vec<ModelBuilder<'data, P>>,
    resources: ResourcesBuilder<'data, P>,
    controls: P::Asset<'data>,
    cardinality: PhantomData<C>,
}
impl<'data, P: Profile, C: Cardinality> ModelCollectionBuilder<'data, P, C> {
    /// Starts an empty draft with the profile's default collection controls.
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            resources: Resources::builder(),
            controls: Default::default(),
            cardinality: Default::default(),
        }
    }
    /// Edits the shared texture definitions before publication.
    pub fn resources_mut(&mut self) -> &mut ResourcesBuilder<'data, P> {
        &mut self.resources
    }
    /// Replaces collection-wide controls, checked when the collection is finished.
    pub fn set_collection_controls(&mut self, controls: P::Asset<'data>) {
        self.controls = controls;
    }
    /// Appends an unpublished model whose resource references resolve at publication.
    pub fn add_model(&mut self, name: impl Into<Cow<'data, str>>) -> usize {
        let model = Model::builder(name);
        let index = self.models.len();
        self.models.push(model);
        index
    }
    /// Reads a reserved model identity without depending on serialized table positions.
    pub fn model_id(&self, index: usize) -> Option<ModelId> {
        self.models.get(index).map(ModelBuilder::id)
    }
    /// Accesses a model draft by its collection position.
    pub fn model_mut(&mut self, index: usize) -> Option<&mut ModelBuilder<'data, P>> {
        self.models.get_mut(index)
    }
    /// Edits a model and its collection's shared resource draft together.
    pub fn model_and_resources_mut(
        &mut self,
        index: usize,
    ) -> Option<(&mut ModelBuilder<'data, P>, &mut ResourcesBuilder<'data, P>)> {
        Some((self.models.get_mut(index)?, &mut self.resources))
    }
    /// Adds an existing model draft without publishing or validating it early.
    pub fn push(&mut self, model: ModelBuilder<'data, P>) -> usize {
        let index = self.models.len();
        self.models.push(model);
        index
    }
    /// Validates all models against one resource arena and the final collection controls.
    pub fn finish(self) -> Result<ModelCollection<'data, P, C>, Error> {
        let mut resources = self.resources.finish();
        let mut arenas = vec![resources.authority_id()];
        let mut models = Vec::with_capacity(self.models.len());
        for draft in self.models {
            let (model, arena) = draft.finish_parts()?;
            if !arenas.contains(&arena.authority_id()) {
                arenas.push(arena.authority_id());
                resources = resources.merge(arena)?;
            }
            models.push(model);
        }
        ModelCollection::from_parts(models, resources, self.controls)
    }
}
impl<P: Profile, C: Cardinality> Default for ModelCollectionBuilder<'_, P, C> {
    fn default() -> Self {
        Self::new()
    }
}
/// A complete standalone model and its dependency arena.
pub type SingleModel<'data, P = Universal> = ModelCollection<'data, P, Single>;
impl<'data, P: Profile, C: Cardinality> ModelCollection<'data, P, C> {
    /// Begins unpublished models and their shared dependencies in one collection.
    pub fn builder() -> ModelCollectionBuilder<'data, P, C> {
        ModelCollectionBuilder::new()
    }
    /// Places a complete standalone model in this collection cardinality.
    pub fn single(single: SingleModel<'data, P>) -> Self {
        let [model] = single.models;
        Self {
            models: C::single(model),
            resources: single.resources,
            controls: single.controls,
        }
    }
    /// Joins standalone models with an explicitly chosen collection authoring context.
    ///
    /// The caller chooses collection controls rather than silently inheriting one
    /// model's document context. Conflicting resource edits are rejected.
    pub fn from_models(
        singles: Vec<SingleModel<'data, P>>,
        controls: P::Asset<'data>,
    ) -> Result<Self, Error> {
        let mut resources = Resources::default();
        let mut authorities = Vec::new();
        let mut models = Vec::with_capacity(singles.len());
        for single in singles {
            let [model] = single.models;
            models.push(model);
            let identity = single.resources.authority_id();
            if !authorities.contains(&identity) {
                authorities.push(identity);
                resources = resources.merge(single.resources)?;
            }
        }
        Self::from_parts(models, resources, controls)
    }
    /// Checks graphs and their dependency definitions before publishing a collection.
    pub fn from_parts(
        models: Vec<Model<'data, P>>,
        resources: Resources<'data, P>,
        controls: P::Asset<'data>,
    ) -> Result<Self, Error> {
        let set = Self {
            models: C::collect(models)?,
            resources,
            controls,
        };
        set.validate()?;
        Ok(set)
    }
    /// Consumes the collection without losing its dependency or authoring context.
    pub fn into_parts(self) -> (Vec<Model<'data, P>>, Resources<'data, P>, P::Asset<'data>) {
        (C::into_vec(self.models), self.resources, self.controls)
    }
    /// Collection-wide authoring context supplied by this profile.
    pub fn collection_controls(&self) -> &P::Asset<'data> {
        &self.controls
    }
    /// Checks replacement collection controls before publishing them.
    pub fn set_collection_controls(&mut self, controls: P::Asset<'data>) -> Result<(), Error> {
        let mut draft = self.clone();
        draft.controls = controls;
        draft.validate()?;
        *self = draft;
        Ok(())
    }
    /// Immutable graphs in collection order.
    pub fn as_slice(&self) -> &[Model<'data, P>] {
        C::as_slice(&self.models)
    }
    /// Selects a graph by collection index.
    pub fn get(&self, index: usize) -> Option<&Model<'data, P>> {
        self.as_slice().get(index)
    }
    /// Looks up a model by its stable identity.
    pub fn get_by_id(&self, id: ModelId) -> Option<&Model<'data, P>> {
        self.iter().find(|model| model.id() == id)
    }
    /// Borrows a model and its collection context by stable identity.
    pub fn view_by_id(&self, id: ModelId) -> Option<ModelView<'_, 'data, P>> {
        Some(ModelView {
            model: self.get_by_id(id)?,
            resources: &self.resources,
            controls: &self.controls,
        })
    }
    /// Number of models in the collection.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
    /// Whether this is an empty authoring collection.
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    /// Inspects graphs in collection order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Model<'data, P>> {
        self.as_slice().iter()
    }
    /// Selects a graph with its complete dependency context.
    pub fn view(&self, index: usize) -> Option<ModelView<'_, 'data, P>> {
        self.get(index).map(|model| ModelView {
            model,
            resources: &self.resources,
            controls: &self.controls,
        })
    }
    /// The single owned arena used to resolve texture identities.
    pub fn resources(&self) -> &Resources<'data, P> {
        &self.resources
    }
    /// Resolves a texture identity in the collection arena.
    pub fn texture(&self, id: TextureId) -> Option<&Texture<'data, P>> {
        self.resources.texture(id)
    }
    /// Inspects the collection's texture definitions in identity order.
    pub fn textures(&self) -> impl ExactSizeIterator<Item = (TextureId, &Texture<'data, P>)> {
        self.resources.textures()
    }
    /// Checks every graph against the same arena and collection authoring context.
    pub fn validate(&self) -> Result<(), Error> {
        let model_ids: BTreeSet<_> = self.iter().map(Model::id).collect();
        for (index, model) in self.iter().enumerate() {
            if self.as_slice()[..index]
                .iter()
                .any(|other| model.same_identity(other))
            {
                return Err(Error::new(
                    "models",
                    "model identities must be distinct within a collection",
                ));
            }
            model.validate(&self.resources, &model_ids)?;
        }
        let mut invalid = false;
        {
            self.controls.visit_links(&mut |link| {
                let exists = match link {
                    Link::Model(id) => model_ids.contains(&id),
                    Link::Texture(id) => self.resources().texture(id).is_some(),
                    Link::Event(id) => self.iter().any(|model| model.event(id).is_some()),
                    Link::Clip(id) => self.iter().any(|m| m.clip(id).is_some()),
                    Link::Node(id) => self.iter().any(|m| m.node(id).is_some()),
                    Link::TypedMesh { id, kind } => self
                        .iter()
                        .any(|m| m.mesh(id).is_some_and(|mesh| mesh.geometry.kind() == kind)),
                    Link::Mesh(id) => self.iter().any(|m| m.mesh(id).is_some()),
                    Link::Material(id) => self.iter().any(|m| m.material(id).is_some()),
                    Link::Skin(id) => self.iter().any(|m| m.skin(id).is_some()),
                    Link::Clock(id) => self.iter().any(|m| m.clock(id).is_some()),
                    Link::Controller(id) => self.iter().any(|m| m.controller(id).is_some()),
                    Link::Instance(id) => self.iter().any(|m| m.instance(id).is_some()),
                    Link::Feature(id) => self.iter().any(|m| m.feature(id).is_some()),
                    Link::FeatureAlternatives { id, kinds } => self.iter().any(|m| {
                        m.feature(id)
                            .is_some_and(|feature| kinds.contains(&feature.payload.kind()))
                    }),
                    Link::TypedFeature { id, kind } => self
                        .iter()
                        .any(|m| m.feature(id).is_some_and(|f| f.payload.kind() == kind)),
                    Link::TextureTransform(id) => {
                        self.iter().any(|m| m.texture_transform(id).is_some())
                    }
                    Link::System(id) => self.iter().any(|m| m.system(id).is_some()),
                    Link::SystemMember { system, member } => self
                        .iter()
                        .any(|m| m.system(system).is_some_and(|value| value.contains(member))),
                    Link::AppearanceSlot(id) => {
                        self.iter().any(|m| m.appearance().contains_slot(id))
                    }
                    Link::InvalidValue | Link::Property { .. } => false,
                };
                invalid |= !exists;
            });
        }
        if invalid {
            return Err(Error::new(
                "asset.controls",
                "authoring controls contain an invalid value or missing relationship",
            ));
        }
        P::validate_asset(&self.controls, self.as_slice(), self.resources())
    }
    /// Edits a model and its dependencies, checking the complete collection before publication.
    pub fn edit_model<R>(
        &mut self,
        index: usize,
        edit: impl FnOnce(&mut ModelBuilder<'data, P>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let model = self
            .get(index)
            .ok_or_else(|| Error::new("model", "unknown model index"))?
            .clone();
        let mut builder = ModelBuilder::from_parts(model, self.resources.clone());
        let result = edit(&mut builder)?;
        let single = builder.finish()?;
        let [model] = single.models;
        let mut models = self.as_slice().to_vec();
        models[index] = model;
        let draft = Self::from_parts(models, single.resources, self.controls.clone())?;
        *self = draft;
        Ok(result)
    }
    /// Edits resources and checks every graph before publishing the replacement arena.
    pub fn edit_resources<R>(
        &mut self,
        edit: impl FnOnce(&mut ResourcesBuilder<'data, P>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut builder = self.resources.draft();
        let result = edit(&mut builder)?;
        let draft = Self::from_parts(
            self.as_slice().to_vec(),
            builder.finish(),
            self.controls.clone(),
        )?;
        *self = draft;
        Ok(result)
    }
    /// Embeds all models and resources in a containing vocabulary.
    pub fn widen<Q: Extends<P>>(self) -> ModelCollection<'data, Q, C> {
        ModelCollection {
            models: C::collect(
                C::into_vec(self.models)
                    .into_iter()
                    .map(Model::widen::<Q>)
                    .collect(),
            )
            .expect("unchanged cardinality"),
            resources: self.resources.widen::<Q>(),
            controls: Q::asset(self.controls),
        }
    }
    /// Adds another vocabulary while retaining this collection in its left branch.
    pub fn union<Q: Profile>(self) -> ModelCollection<'data, Union<P, Q>, C> {
        self.widen()
    }
    /// Adds another vocabulary while retaining this collection in its right branch.
    pub fn union_right<Q: Profile>(self) -> ModelCollection<'data, Union<Q, P>, C> {
        ModelCollection {
            models: C::collect(
                C::into_vec(self.models)
                    .into_iter()
                    .map(Model::union_right::<Q>)
                    .collect(),
            )
            .expect("unchanged cardinality"),
            resources: self.resources.map_controls(Either::Right),
            controls: Either::Right(self.controls),
        }
    }
    pub(crate) fn widen_cardinality<D: ContainsCardinality<C>>(
        self,
    ) -> ModelCollection<'data, P, D> {
        ModelCollection {
            models: D::embed(self.models),
            resources: self.resources,
            controls: self.controls,
        }
    }
}
impl<'data, P: Profile> ModelCollection<'data, P, Single> {
    pub(crate) fn from_single_parts(
        model: Model<'data, P>,
        resources: Resources<'data, P>,
    ) -> Result<Self, Error> {
        Self::from_parts(vec![model], resources, Default::default())
    }
    /// The single graph together with its complete dependency context.
    pub fn model(&self) -> ModelView<'_, 'data, P> {
        self.view(0).expect("single cardinality")
    }
    /// Edits the standalone model transactionally.
    pub fn edit<R>(
        &mut self,
        edit: impl FnOnce(&mut ModelBuilder<'data, P>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.edit_model(0, edit)
    }
}
impl<'data, P: Profile> Deref for ModelCollection<'data, P, Single> {
    type Target = Model<'data, P>;
    fn deref(&self) -> &Self::Target {
        &self.models[0]
    }
}
impl<'data, P: Profile, C: Cardinality> Index<usize> for ModelCollection<'data, P, C> {
    type Output = Model<'data, P>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<'view, 'data, P: Profile, C: Cardinality> IntoIterator
    for &'view ModelCollection<'data, P, C>
{
    type Item = &'view Model<'data, P>;
    type IntoIter = std::slice::Iter<'view, Model<'data, P>>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
/// A selected graph paired with its collection's dependency and authoring context.
#[derive(Debug)]
pub struct ModelView<'view, 'data, P: Profile = Universal> {
    model: &'view Model<'data, P>,
    resources: &'view Resources<'data, P>,
    controls: &'view P::Asset<'data>,
}
impl<P: Profile> Copy for ModelView<'_, '_, P> {}
impl<P: Profile> Clone for ModelView<'_, '_, P> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'view, 'data, P: Profile> ModelView<'view, 'data, P> {
    /// The selected graph, without an independently stored resource handle.
    pub fn model(&self) -> &'view Model<'data, P> {
        self.model
    }
    /// The collection arena used by every graph in this view's collection.
    pub fn resources(&self) -> &'view Resources<'data, P> {
        self.resources
    }
    /// Collection-wide authoring context, including shared banks where supported.
    pub fn collection_controls(&self) -> &'view P::Asset<'data> {
        self.controls
    }
    /// Resolves a texture identity against this view's collection.
    pub fn texture(&self, id: TextureId) -> Option<&'view Texture<'data, P>> {
        self.resources.texture(id)
    }
    /// Inspects resource definitions without copying them.
    pub fn textures(&self) -> impl ExactSizeIterator<Item = (TextureId, &'view Texture<'data, P>)> {
        self.resources.textures()
    }
}
impl<'data, P: Profile> Deref for ModelView<'_, 'data, P> {
    type Target = Model<'data, P>;
    fn deref(&self) -> &Self::Target {
        self.model
    }
}

impl<'data, P: Profile, C: Cardinality> ModelCollection<'data, P, C> {
    /// Projects every owner exactly and checks the destination asset cardinality and references.
    pub fn try_narrow<Q: Narrows<P>>(self) -> Result<ModelCollection<'data, Q, Q::Models>, Error> {
        let models = C::into_vec(self.models)
            .into_iter()
            .map(Model::try_narrow::<Q>)
            .collect::<Result<_, Error>>()?;
        ModelCollection::from_parts(
            models,
            self.resources.try_narrow::<Q>()?,
            Q::asset(self.controls)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::model::semantic::profiles::Universal;
    use crate::model::semantic::{
        ModelBuilder, ModelCollection, Multiple, Resources, Single, Texture,
    };

    #[test]
    fn joining_models_preserves_disjoint_resources() {
        let mut first = ModelBuilder::new("first");
        let first_texture = first.add_texture(Texture::file("first.blp")).unwrap();
        let mut second = ModelBuilder::new("second");
        let second_texture = second.add_texture(Texture::file("second.blp")).unwrap();
        let set = ModelCollection::<Universal, Multiple>::from_models(
            vec![first.finish().unwrap(), second.finish().unwrap()],
            None,
        )
        .unwrap();
        assert_eq!(set.resources().len(), 2);
        assert!(set.view(0).unwrap().texture(first_texture).is_some());
        assert!(set.view(1).unwrap().texture(second_texture).is_some());
        assert!(std::ptr::eq(
            set.view(0).unwrap().resources(),
            set.view(1).unwrap().resources()
        ));
    }

    #[test]
    fn collection_rejects_duplicate_graph_identity() {
        let single = ModelBuilder::new("single").finish().unwrap();
        assert!(ModelCollection::<Universal, Multiple>::from_models(
            vec![single.clone(), single],
            None
        )
        .is_err());
    }

    #[test]
    fn conflicting_resource_branches_cannot_be_joined() {
        let mut resources = Resources::builder();
        let texture = resources.add_texture(Texture::file("original.blp"));
        let resources = resources.finish();
        let mut first = ModelBuilder::new("first");
        first.set_resources(resources.clone());
        let mut second = ModelBuilder::new("second");
        second.set_resources(resources);
        second
            .set_texture(texture, Texture::file("edited.blp"))
            .unwrap();
        assert!(ModelCollection::<Universal, Multiple>::from_models(
            vec![first.finish().unwrap(), second.finish().unwrap()],
            None
        )
        .is_err());
    }

    #[test]
    fn single_container_checks_count_before_publication() {
        let first = ModelBuilder::new("first").finish().unwrap();
        let second = ModelBuilder::new("second").finish().unwrap();
        assert!(
            ModelCollection::<Universal, Single>::from_models(vec![first, second], None).is_err()
        );
    }
    #[test]
    fn cross_model_links_resolve_by_identity_and_reject_missing_targets() {
        use crate::model::semantic::profiles::{NoControls, Profile, Unavailable};
        use crate::model::semantic::{GeometryData, ModelId, SingleLook};
        #[derive(Clone, Debug)]
        struct Linked;
        impl Profile for Linked {
            type Models = Multiple;
            type Asset<'a> = NoControls;
            type Appearance<'a> = SingleLook<'a>;
            type Binding<'a> = Unavailable;
            type Geometry<'a> = GeometryData<'a>;
            type Model<'a> = Vec<ModelId>;
            type System<'a> = Unavailable;
            type Surface<'a> = Unavailable;
            type Feature<'a> = Unavailable;
            type Node<'a> = NoControls;
            type Texture<'a> = NoControls;
            type Mesh<'a> = NoControls;
            type Instance<'a> = NoControls;
            type Draw = NoControls;
            type Clip<'a> = NoControls;
            type Controller<'a> = NoControls;
        }
        let mut draft = ModelCollection::<Linked>::builder();
        let first = draft.add_model("first");
        let second = draft.add_model("second");
        let second_id = draft.model_id(second).unwrap();
        draft
            .model_mut(first)
            .unwrap()
            .set_controls(vec![second_id]);
        let collection = draft.finish().unwrap();
        assert_eq!(collection.get_by_id(second_id).unwrap().name(), "second");
        let (mut models, resources, controls) = collection.into_parts();
        models.swap(0, 1);
        let collection =
            ModelCollection::<Linked>::from_parts(models, resources, controls).unwrap();
        assert_eq!(collection[1].controls(), &[second_id]);
        let (mut models, resources, controls) = collection.into_parts();
        models.remove(0);
        assert!(ModelCollection::<Linked>::from_parts(models, resources, controls).is_err());
    }
}
