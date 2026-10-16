//! Native asset loading without lowering through Warcraft III records.
use crate::error::ModelError;
use crate::textures::{texture_requests, TextureImageKey, TextureRequest};
use crate::{compile_model_with_options, CompileOptions, CompiledModel, RenderDiagnostic};
use bevy::asset::{
    io::{AssetReaderError, Reader, SliceReader},
    AssetLoader, AssetPath, LoadContext, ReadAssetBytesError,
};
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt as _;
use std::path::{Component, Path, PathBuf};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use wc3::model::common::{FileKey, IntoOwned as _};
use wc3::model::conversion::{ImportOptions, IssueKind, Report};
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::{DynamicDocument, NoExtensions};
use wc3::model::formats::{
    diablo::{d3, d4},
    m2, m3, wmo,
};
use wc3::model::io::mdlx as semantic_mdlx;
use wc3::model::io::{
    diablo::{d3 as semantic_d3, d4 as semantic_d4},
    m2 as semantic_m2, m3 as semantic_m3, wmo as semantic_wmo,
};
use wc3::model::{
    AddressMode, Attachment, FeatureId, Model, NodeFeature, ParticleDetails, ParticleEmitter,
    Resource as ModelResource,
};
use wc3::visit_m2_model;

/// Authored use of an optional child model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourcePurpose {
    /// External model mounted at an authored attachment.
    Attachment,
    /// External model instantiated for each particle birth.
    Particle,
}

/// Per-load ancestry propagated to optional model dependencies.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ModelLoaderSettings {
    ancestry: Vec<String>,
}

#[derive(Default)]
struct ModelDependencyGraph {
    edges: HashMap<AssetPath<'static>, HashSet<AssetPath<'static>>>,
    pending: HashMap<AssetPath<'static>, (u64, HashSet<AssetPath<'static>>)>,
    generation: u64,
}
impl ModelDependencyGraph {
    fn begin_load(&mut self, owner: &AssetPath<'_>) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.pending
            .insert(owner.clone_owned(), (self.generation, HashSet::new()));
        self.generation
    }
    fn finish_load(&mut self, owner: &AssetPath<'_>, generation: u64, success: bool) {
        if self
            .pending
            .get(owner)
            .is_none_or(|(current, _)| *current != generation)
        {
            return;
        }
        let pending = self
            .pending
            .remove(&owner.clone_owned())
            .map(|(_, edges)| edges);
        if success {
            self.edges.remove(&owner.clone_owned());
            if let Some(edges) = pending.filter(|edges| !edges.is_empty()) {
                self.edges.insert(owner.clone_owned(), edges);
            }
        }
    }
    fn insert_for_load(
        &mut self,
        owner: &AssetPath<'_>,
        generation: u64,
        child: &AssetPath<'_>,
    ) -> bool {
        if self
            .pending
            .get(owner)
            .is_none_or(|(current, _)| *current != generation)
        {
            return false;
        }
        self.insert(owner, child)
    }
    fn insert(&mut self, owner: &AssetPath<'_>, child: &AssetPath<'_>) -> bool {
        let mut pending = vec![child.clone_owned()];
        let mut visited = HashSet::new();
        while let Some(path) = pending.pop() {
            if &path == owner {
                return false;
            }
            if visited.insert(path.clone()) {
                for children in [
                    self.edges.get(&path),
                    self.pending.get(&path).map(|(_, edges)| edges),
                ]
                .into_iter()
                .flatten()
                {
                    pending.extend(children.iter().cloned());
                }
            }
        }
        if let Some((_, edges)) = self.pending.get_mut(&owner.clone_owned()) {
            edges.insert(child.clone_owned());
        } else {
            self.edges
                .entry(owner.clone_owned())
                .or_default()
                .insert(child.clone_owned());
        }
        true
    }
}
// Pending and committed edges both guard against concurrent cycles. A failed
// reload keeps the old asset live, so only a successful publication replaces its
// committed dependencies; failure discards the provisional edges.
struct ModelDependencyLoad<'a> {
    graph: &'a Mutex<ModelDependencyGraph>,
    owner: AssetPath<'static>,
    committed: bool,
    generation: u64,
}
impl<'a> ModelDependencyLoad<'a> {
    fn begin(graph: &'a Mutex<ModelDependencyGraph>, owner: &AssetPath<'_>) -> Self {
        let generation = graph
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .begin_load(owner);
        Self {
            graph,
            owner: owner.clone_owned(),
            committed: false,
            generation,
        }
    }
    fn commit(mut self) {
        self.graph
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .finish_load(&self.owner, self.generation, true);
        self.committed = true;
    }
}
impl Drop for ModelDependencyLoad<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.graph
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .finish_load(&self.owner, self.generation, false);
        }
    }
}

struct ChildModelContext<'a> {
    ancestry: &'a [String],
    graph: &'a Mutex<ModelDependencyGraph>,
    generation: u64,
}

/// Resolves complete semantic identities to Bevy asset paths.
///
/// Providers may map file-data IDs or namespaced SNOs to any registered Bevy
/// asset source, including CASC. Candidate order is authoritative. Returning no
/// path leaves optional images unresolved and fails required native companions.
/// No filename is invented for an archive ID or an external animation hash.
pub trait ResourceResolver: Send + Sync + 'static {
    /// Native conversion policy for this model. Strict preservation is the default.
    /// Accepted losses and approximations are retained in compiled diagnostics.
    fn import_options(&self, _owner: &AssetPath<'_>) -> ImportOptions {
        ImportOptions::default()
    }
    /// Candidate paths, retaining any named Bevy asset source.
    fn candidates(
        &self,
        resource: &ModelResource<'_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>>;
    /// Select the resource for one texture before image interpretation.
    ///
    /// `index` is its zero-based position in the semantic texture list, shared by
    /// every role variant of that texture. Exact texture selections can precede
    /// replaceable-category or archive identity mappings here. The request retains
    /// typed identity, original resource, color role, and authored sampler. Returning
    /// a different resource does not mutate the model or any shared image asset.
    fn texture_resource(
        &self,
        _index: usize,
        request: &TextureRequest<'_, '_>,
        _owner: &AssetPath<'_>,
    ) -> ModelResource<'static> {
        request.texture.resource.clone().into_owned()
    }
    /// Image paths in lookup order, after application texture selection.
    ///
    /// At each resource location, try the literal path followed by BLP, DDS, PNG,
    /// and TGA extensions. Path case is preserved; case-insensitive archive lookup
    /// belongs to the registered asset source. Non-image companions use `candidates`
    /// directly and never receive these image-specific extension fallbacks.
    fn texture_candidates(
        &self,
        index: usize,
        request: &TextureRequest<'_, '_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>> {
        let resource = self.texture_resource(index, request, owner);
        image_candidates(self.candidates(&resource, owner))
    }
    /// Candidate child models for an authored attachment or particle emitter.
    /// Missing optional children remain visible as integration diagnostics.
    fn child_model_candidates(
        &self,
        _feature: FeatureId,
        _purpose: ResourcePurpose,
        resource: &ModelResource<'_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>> {
        let mut result = Vec::new();
        for path in self.candidates(resource, owner) {
            result.push(path.clone());
            if path
                .path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("mdl"))
            {
                result.push(
                    AssetPath::from(path.path().with_extension("mdx"))
                        .with_source(path.source().clone_owned()),
                );
            }
        }
        result
    }
    /// Appearance and other semantic compilation choices for this asset.
    /// The borrowed imported graph exposes valid owner-local look and feature IDs;
    /// implementations must not retain this graph beyond the callback.
    fn compile_options(&self, _owner: &AssetPath<'_>, _model: &Model<'_>) -> CompileOptions {
        CompileOptions::default()
    }
    /// WMO layout provenance; version 17 alone does not identify a dialect.
    fn wmo_profile(&self, _owner: &AssetPath<'_>) -> Option<wmo::Profile> {
        None
    }
    /// Explicit vertex interpretations for Diablo IV source buffers.
    fn diablo4_layouts(&self, _owner: &AssetPath<'_>) -> Vec<semantic_d4::BufferLayout> {
        Vec::new()
    }
    /// External Diablo IV payload identity, if the appearance uses separate storage.
    fn diablo4_payload(&self, _owner: &AssetPath<'_>) -> Option<ModelResource<'static>> {
        None
    }
    /// Independently selected Diablo III animations to bind by bone name.
    fn diablo3_animations(&self, _owner: &AssetPath<'_>) -> Vec<ModelResource<'static>> {
        Vec::new()
    }
}

/// Filesystem-style path resolution beside the model, then at its asset root.
#[derive(Default)]
pub struct PathResolver;
impl ResourceResolver for PathResolver {
    fn candidates(
        &self,
        resource: &ModelResource<'_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>> {
        let Some(path) = resource_path(resource, owner.path()) else {
            return Vec::new();
        };
        if path.as_os_str().is_empty()
            || (path.is_absolute() && !root_anchored(resource))
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Vec::new();
        }
        if root_anchored(resource) {
            return vec![AssetPath::from(path).with_source(owner.source().clone_owned())];
        }
        let local = owner.path().parent().unwrap_or(Path::new("")).join(&path);
        let mut paths = vec![AssetPath::from(local).with_source(owner.source().clone_owned())];
        if path != paths[0].path() {
            paths.push(AssetPath::from(path).with_source(owner.source().clone_owned()));
        }
        paths
    }
}

fn image_candidates(paths: Vec<AssetPath<'static>>) -> Vec<AssetPath<'static>> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for path in paths {
        if seen.insert(path.clone()) {
            result.push(path.clone());
        }
        for extension in ["blp", "dds", "png", "tga"] {
            let candidate = AssetPath::from(path.path().with_extension(extension))
                .with_source(path.source().clone_owned());
            if seen.insert(candidate.clone()) {
                result.push(candidate);
            }
        }
    }
    result
}

fn root_anchored(resource: &ModelResource<'_>) -> bool {
    match resource {
        ModelResource::Root => true,
        ModelResource::Relative { owner, .. } => root_anchored(owner),
        _ => false,
    }
}
fn resource_path(resource: &ModelResource<'_>, root: &Path) -> Option<PathBuf> {
    match resource {
        ModelResource::Root => Some(root.to_owned()),
        ModelResource::Path(bytes) => {
            let normalized: Vec<u8> = bytes
                .iter()
                .map(|byte| if *byte == b'\\' { b'/' } else { *byte })
                .collect();
            #[cfg(unix)]
            {
                Some(PathBuf::from(OsString::from_vec(normalized)))
            }
            #[cfg(not(unix))]
            {
                String::from_utf8(normalized).ok().map(PathBuf::from)
            }
        }
        ModelResource::Relative { owner, suffix } => {
            let mut path = resource_path(owner, root)?;
            let mut name = OsString::from(path.file_stem()?);
            name.push(suffix.as_ref());
            path.set_file_name(name);
            Some(path)
        }
        _ => None,
    }
}

/// Owned compiled model and observable unresolved external image identities.
#[derive(Asset, TypePath)]
pub struct ModelAsset {
    /// Shared CPU render definitions, ready for GPU preparation.
    pub compiled: CompiledModel,
    /// Resolved image variants keyed by semantic identity and effective color space.
    pub images: HashMap<TextureImageKey, Handle<Image>>,
    /// Optional images requiring an application resource provider or override.
    pub unresolved_images: Vec<ModelResource<'static>>,
}

/// Resource-provider configuration installed before `Wc3BevyPlugin` is added.
#[derive(Resource, Clone)]
pub struct AssetSettings {
    /// Native identity provider used by the model asset loader.
    pub resolver: Arc<dyn ResourceResolver>,
}
impl Default for AssetSettings {
    fn default() -> Self {
        Self {
            resolver: Arc::new(PathResolver),
        }
    }
}

/// Native Universal loader. Applications can register a configured resolver.
#[derive(TypePath)]
pub struct ModelLoader {
    resolver: Arc<dyn ResourceResolver>,
    dependencies: Mutex<ModelDependencyGraph>,
}
impl Default for ModelLoader {
    fn default() -> Self {
        Self::new(Arc::new(PathResolver))
    }
}
impl ModelLoader {
    /// Uses an application-owned archive or resource-identity mapping.
    pub fn new(resolver: Arc<dyn ResourceResolver>) -> Self {
        Self {
            resolver,
            dependencies: Mutex::new(ModelDependencyGraph::default()),
        }
    }
}

fn owner_key(owner: &AssetPath<'_>) -> FileKey {
    match owner.path().to_str() {
        Some(path) => FileKey::Path(path.to_owned()),
        None => FileKey::RawPath(owner.path().as_os_str().as_encoded_bytes().to_vec()),
    }
}
fn file_resource(key: &FileKey) -> ModelResource<'static> {
    match key {
        FileKey::Root => ModelResource::Root,
        FileKey::Path(path) => ModelResource::Path(Cow::Owned(path.as_bytes().to_vec())),
        FileKey::RawPath(path) => ModelResource::Path(Cow::Owned(path.clone())),
        FileKey::FileId(id) => ModelResource::FileDataId(*id),
        FileKey::Sibling { owner, suffix } => ModelResource::Relative {
            owner: Box::new(file_resource(owner)),
            suffix: Cow::Owned(suffix.clone()),
        },
    }
}
async fn read_resource(
    context: &mut LoadContext<'_>,
    resolver: &dyn ResourceResolver,
    owner: &AssetPath<'_>,
    resource: &ModelResource<'_>,
    required: bool,
) -> Result<Option<Vec<u8>>, ModelError> {
    let candidates = resolver.candidates(resource, owner);
    for path in &candidates {
        match context.read_asset_bytes(path.clone()).await {
            Ok(bytes) => return Ok(Some(bytes)),
            Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => {}
            Err(error) => {
                return Err(ModelError(format!(
                    "cannot read {resource:?} at {path}: {error}"
                )));
            }
        }
    }
    if required {
        Err(ModelError(format!(
            "unresolved required model companion {resource:?} for {owner}; tried {candidates:?}"
        )))
    } else {
        Ok(None)
    }
}

async fn compile_loaded(
    model: &Model<'_>,
    report: &Report,
    context: &mut LoadContext<'_>,
    resolver: &dyn ResourceResolver,
    owner: &AssetPath<'_>,
    children: &ChildModelContext<'_>,
) -> Result<ModelAsset, ModelError> {
    let mut options = resolver.compile_options(owner, model);
    let mut child_diagnostics = report
        .issues
        .iter()
        .filter(|issue| matches!(issue.kind, IssueKind::Dropped | IssueKind::Approximated))
        .map(|issue| RenderDiagnostic {
            feature: None,
            material: None,
            message: format!(
                "native import {:?} at {}: {}",
                issue.kind, issue.path, issue.description
            ),
        })
        .collect::<Vec<_>>();
    for (id, feature) in model.features() {
        if options.child_models.contains_key(&id) {
            continue;
        }
        let (purpose, resource) = match &feature.payload {
            NodeFeature::Attachment(Attachment::Child(value)) => (
                ResourcePurpose::Attachment,
                value.resource.clone().into_owned(),
            ),
            NodeFeature::Particle(ParticleEmitter::Simulated(value)) => {
                let ParticleDetails::Model(definition) = &value.details else {
                    continue;
                };
                if !definition.model_resource
                    || definition.text_resource
                    || definition.resource.is_empty()
                {
                    continue;
                }
                (
                    ResourcePurpose::Particle,
                    ModelResource::Path(Cow::Owned(definition.resource.as_bytes().to_vec())),
                )
            }
            _ => continue,
        };
        let mut resolved = false;
        for path in resolver.child_model_candidates(id, purpose, &resource, owner) {
            match context.read_asset_bytes(path.clone()).await {
                Ok(_) => {}
                Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => {
                    continue;
                }
                Err(error) => {
                    return Err(ModelError(format!(
                        "cannot read optional child {resource:?}: {error}"
                    )));
                }
            }
            if children.ancestry.contains(&path.to_string())
                || !children
                    .graph
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .insert_for_load(owner, children.generation, &path)
            {
                child_diagnostics.push(RenderDiagnostic {
                    feature: Some(id),
                    material: None,
                    message: format!(
                        "cyclic child model reference {resource:?} at {path} is not instantiated"
                    ),
                });
                resolved = true;
                break;
            }
            let mut ancestry = children.ancestry.to_vec();
            ancestry.push(owner.to_string());
            let handle = context
                .load_builder()
                .with_settings::<ModelLoaderSettings>(move |settings| {
                    settings.ancestry = ancestry.clone()
                })
                .load(path);
            options.child_models.insert(id, handle);
            resolved = true;
            break;
        }
        if !resolved {
            child_diagnostics.push(RenderDiagnostic {
                feature: Some(id),
                material: None,
                message: format!(
                    "optional {purpose:?} child model {resource:?} could not be resolved"
                ),
            });
        }
    }
    let mut images = HashMap::new();
    let mut variants: HashMap<_, Handle<Image>> = HashMap::new();
    let mut encoded = HashMap::new();
    let mut missing = HashSet::new();
    let indices: HashMap<_, _> = model
        .textures()
        .enumerate()
        .map(|(index, (id, _))| (id, index))
        .collect();
    let mut unresolved_images = Vec::new();
    for request in texture_requests(model) {
        let id = request.id;
        let texture = request.texture;
        let image_key = request.key();
        let resource = texture.resource.clone().into_owned();
        let candidates = resolver.texture_candidates(indices[&id], &request, owner);
        let address_key = |mode| match mode {
            AddressMode::Repeat => 0u16,
            AddressMode::Clamp => 1,
            AddressMode::Mirror => 2,
            AddressMode::Unknown(value) => value as u16 + 3,
        };
        let srgb = request.color_space.is_srgb();
        let key = (
            candidates.clone(),
            address_key(request.sampler.u),
            address_key(request.sampler.v),
            srgb,
        );
        if let Some(handle) = variants.get(&key) {
            images.insert(image_key, handle.clone());
            continue;
        }
        let mut resolved = None;
        for path in candidates {
            if missing.contains(&path) {
                continue;
            }
            if !encoded.contains_key(&path) {
                match context.read_asset_bytes(path.clone()).await {
                    Ok(bytes) => {
                        encoded.insert(path.clone(), bytes);
                    }
                    Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => {
                        missing.insert(path);
                        continue;
                    }
                    Err(error) => {
                        return Err(ModelError(format!(
                            "cannot load image {resource:?}: {error}"
                        )));
                    }
                }
            }
            let mut sampler = ImageSamplerDescriptor::linear();
            let address = |mode| match mode {
                AddressMode::Repeat => ImageAddressMode::Repeat,
                AddressMode::Mirror => ImageAddressMode::MirrorRepeat,
                _ => ImageAddressMode::ClampToEdge,
            };
            sampler.address_mode_u = address(request.sampler.u);
            sampler.address_mode_v = address(request.sampler.v);
            let mut reader = SliceReader::new(&encoded[&path]);
            let loaded = context
                .load_builder()
                .with_settings::<ImageLoaderSettings>(move |settings| {
                    settings.sampler = ImageSampler::Descriptor(sampler.clone());
                    settings.is_srgb = srgb;
                })
                .load_untyped_value_from_reader(path, &mut reader)
                .await
                .map_err(|error| {
                    ModelError(format!("cannot decode image {resource:?}: {error}"))
                })?;
            let handle = context.add_labeled_asset(
                format!(
                    "Texture{id:?}{:?}{:?}",
                    request.color_space, request.sampler
                ),
                loaded.take::<Image>().ok_or_else(|| {
                    ModelError(format!(
                        "image resource {resource:?} resolved to a non-image asset"
                    ))
                })?,
            );
            resolved = Some(handle);
            break;
        }
        if let Some(handle) = resolved {
            variants.insert(key, handle.clone());
            images.insert(image_key, handle);
        } else if !unresolved_images.contains(&resource) {
            unresolved_images.push(resource);
        }
    }
    let mut compiled = compile_model_with_options(model, options, |request| {
        images.get(&request.key()).cloned()
    })?;
    if !child_diagnostics.is_empty() {
        let mut diagnostics = compiled.diagnostics.to_vec();
        diagnostics.extend(child_diagnostics);
        compiled.diagnostics = diagnostics.into();
    }
    Ok(ModelAsset {
        compiled,
        images,
        unresolved_images,
    })
}

fn decode_m2_bundle<'a, D: m2::Dialect>(
    _: &m2::Document<'_, D>,
    bytes: &'a [u8],
    key: FileKey,
    buffers: &'a BTreeMap<FileKey, Vec<u8>>,
    missing: &mut Option<FileKey>,
) -> Result<m2::Bundle<'a, D>, m2::ReadError> {
    m2::Bundle::decode(bytes, key, |dependency| {
        if let Some(bytes) = buffers.get(&dependency.key) {
            Ok(Some(bytes))
        } else {
            *missing = Some(dependency.key.clone());
            Ok(None)
        }
    })
}

async fn load_wmo<D: wmo::Dialect>(
    bytes: &[u8],
    context: &mut LoadContext<'_>,
    resolver: &dyn ResourceResolver,
    owner: &AssetPath<'_>,
    children: &ChildModelContext<'_>,
) -> Result<ModelAsset, ModelError> {
    let root = wmo::Root::<D>::decode(bytes).map_err(|error| ModelError(error.to_string()))?;
    let mut bundle = wmo::Bundle::from_root(root).with_key(owner_key(owner));
    for dependency in bundle
        .dependencies()
        .map_err(|error| ModelError(error.to_string()))?
    {
        if let wmo::DependencyKind::Group { index, lod } = dependency.kind {
            let bytes = read_resource(
                context,
                resolver,
                owner,
                &file_resource(&dependency.key),
                true,
            )
            .await?
            .unwrap();
            let group =
                wmo::Group::<D>::decode(&bytes).map_err(|error| ModelError(error.to_string()))?;
            bundle.insert_group(
                wmo::GroupKey { index, lod },
                dependency.key,
                group.into_owned(),
            );
        }
    }
    bundle
        .validate_complete()
        .map_err(|error| ModelError(error.to_string()))?;
    let conversion = semantic_wmo::import(&bundle, &resolver.import_options(owner))
        .map_err(|error| ModelError(error.to_string()))?;
    let model = conversion.value.widen();
    compile_loaded(
        &model,
        &conversion.report,
        context,
        resolver,
        owner,
        children,
    )
    .await
}
async fn load_m3<E: m3::Envelope + Send + Sync>(
    document: m3::Document<'_, E>,
    context: &mut LoadContext<'_>,
    resolver: &dyn ResourceResolver,
    owner: &AssetPath<'_>,
    children: &ChildModelContext<'_>,
) -> Result<ModelAsset, ModelError> {
    let mut bundle = m3::Bundle::from_model(document);
    loop {
        let pending: Vec<_> = bundle
            .dependencies()
            .map_err(|error| ModelError(error.to_string()))?
            .into_iter()
            .filter(|hash| bundle.animation(*hash).is_none())
            .collect();
        if pending.is_empty() {
            break;
        }
        for hash in pending {
            let resource = ModelResource::Named {
                namespace: Cow::Borrowed("m3-animation-hash"),
                name: Cow::Owned(format!("{hash:08x}")),
            };
            let data = read_resource(context, resolver, owner, &resource, true)
                .await?
                .unwrap();
            let animation = m3::DynamicDocument::read(&data)
                .map_err(|error| ModelError(error.to_string()))?
                .into_owned();
            bundle
                .insert_animation(hash, animation)
                .map_err(|error| ModelError(error.to_string()))?;
        }
    }
    bundle
        .validate_complete()
        .map_err(|error| ModelError(error.to_string()))?;
    let conversion = semantic_m3::import(&bundle, &resolver.import_options(owner))
        .map_err(|error| ModelError(error.to_string()))?;
    let model = conversion.value.widen();
    compile_loaded(
        &model,
        &conversion.report,
        context,
        resolver,
        owner,
        children,
    )
    .await
}

impl ModelLoader {
    async fn load_native(
        &self,
        generation: u64,
        reader: &mut dyn Reader,
        settings: &ModelLoaderSettings,
        context: &mut LoadContext<'_>,
    ) -> Result<ModelAsset, ModelError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| ModelError(error.to_string()))?;
        let owner = context.path().clone_owned();
        let children = ChildModelContext {
            ancestry: &settings.ancestry,
            graph: &self.dependencies,
            generation,
        };
        let extension = owner
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if extension == "mdx" || extension == "mdl" {
            let source = if bytes.starts_with(b"MDLX") {
                DynamicDocument::<NoExtensions>::decode_mdx(&bytes, 800)
                    .map_err(|error| ModelError(error.to_string()))?
            } else {
                let text = from_utf8(&bytes)
                    .map_err(|error| ModelError(format!("MDL source is not UTF-8: {error}")))?;
                DynamicDocument::<NoExtensions>::decode_mdl(text)
                    .map_err(|error| ModelError(error.to_string()))?
            };
            let conversion =
                semantic_mdlx::import_dynamic_owned(&source, &self.resolver.import_options(&owner))
                    .map_err(|error| ModelError(error.to_string()))?;
            let model = conversion.value.widen();
            return compile_loaded(
                &model,
                &conversion.report,
                context,
                self.resolver.as_ref(),
                &owner,
                &children,
            )
            .await;
        }
        if extension == "m2" {
            let source = m2::DynamicDocument::decode_m2(&bytes)
                .map_err(|error| ModelError(error.to_string()))?;
            let key = owner_key(&owner);
            let mut buffers = BTreeMap::new();
            for dependency in source.dependencies(&key) {
                if matches!(dependency.kind, m2::DependencyKind::Texture)
                    || buffers.contains_key(&dependency.key)
                {
                    continue;
                }
                let data = read_resource(
                    context,
                    self.resolver.as_ref(),
                    &owner,
                    &file_resource(&dependency.key),
                    true,
                )
                .await?
                .unwrap();
                buffers.insert(dependency.key, data);
            }
            loop {
                let mut missing = None;
                let result = visit_m2_model!(&source, |typed| {
                    match decode_m2_bundle(typed, &bytes, key.clone(), &buffers, &mut missing) {
                        Ok(bundle) => {
                            let conversion =
                                semantic_m2::import(&bundle, &self.resolver.import_options(&owner))
                                    .map_err(|error| ModelError(error.to_string()))?;
                            let model = conversion.value.widen();
                            Some(
                                compile_loaded(
                                    &model,
                                    &conversion.report,
                                    context,
                                    self.resolver.as_ref(),
                                    &owner,
                                    &children,
                                )
                                .await,
                            )
                        }
                        Err(error) if missing.is_none() => Some(Err(ModelError(error.to_string()))),
                        Err(_) => None,
                    }
                });
                if let Some(result) = result {
                    return result;
                }
                let key = missing
                    .ok_or_else(|| ModelError("M2 companion resolver made no progress".into()))?;
                let data = read_resource(
                    context,
                    self.resolver.as_ref(),
                    &owner,
                    &file_resource(&key),
                    true,
                )
                .await?
                .unwrap();
                buffers.insert(key, data);
            }
        }
        if extension == "wmo" {
            let profile = self.resolver.wmo_profile(&owner).ok_or_else(|| ModelError("WMO loading requires explicit client layout provenance from ResourceResolver::wmo_profile".into()))?;
            macro_rules! load {
                ($dialect:ty) => {
                    load_wmo::<$dialect>(&bytes, context, self.resolver.as_ref(), &owner, &children)
                        .await
                };
            }
            return match profile {
                wmo::Profile::Classic => load!(wmo::Classic),
                wmo::Profile::Legion => load!(wmo::Legion),
                wmo::Profile::Bfa => load!(wmo::Bfa),
                wmo::Profile::ShadowlandsPre92 => load!(wmo::ShadowlandsPre92),
                wmo::Profile::Shadowlands => load!(wmo::Shadowlands),
                wmo::Profile::Dragonflight => load!(wmo::Dragonflight),
                wmo::Profile::WarWithin => load!(wmo::WarWithin),
                _ => Err(ModelError("legacy WMO asset loading is not implemented; import its native bundle and compile the Universal model directly".into())),
            };
        }
        if extension == "m3" {
            return match m3::DynamicDocument::read(&bytes)
                .map_err(|error| ModelError(error.to_string()))?
            {
                m3::DynamicDocument::Md33(document) => {
                    load_m3(document, context, self.resolver.as_ref(), &owner, &children).await
                }
                m3::DynamicDocument::Md34(document) => {
                    load_m3(document, context, self.resolver.as_ref(), &owner, &children).await
                }
            };
        }
        if extension == "app" || extension == "d3app" || extension == "d4app" {
            if extension != "d4app" {
                if let Ok(appearance) = d3::Appearance::<d3::V260>::read(&bytes) {
                    let appearance = appearance.into_owned();
                    let mut animations = Vec::new();
                    for resource in self.resolver.diablo3_animations(&owner) {
                        let data =
                            read_resource(context, self.resolver.as_ref(), &owner, &resource, true)
                                .await?
                                .unwrap();
                        animations.push(
                            d3::Animation::<d3::V118>::read(&data)
                                .map_err(|error| ModelError(error.to_string()))?
                                .into_owned(),
                        );
                    }
                    let bundle = semantic_d3::Bundle {
                        appearance: &appearance,
                        animations: animations.iter().collect(),
                    };
                    let conversion =
                        semantic_d3::import(&bundle, &self.resolver.import_options(&owner))
                            .map_err(|error| ModelError(error.to_string()))?;
                    let model = conversion.value.widen();
                    return compile_loaded(
                        &model,
                        &conversion.report,
                        context,
                        self.resolver.as_ref(),
                        &owner,
                        &children,
                    )
                    .await;
                }
            }
            let payload = if let Some(resource) = self.resolver.diablo4_payload(&owner) {
                read_resource(context, self.resolver.as_ref(), &owner, &resource, true).await?
            } else {
                None
            };
            let appearance = match payload.as_deref() {
                Some(payload) => d4::Appearance::read_with_payload(&bytes, payload),
                None => d4::Appearance::read(&bytes),
            }
            .map_err(|error| ModelError(error.to_string()))?;
            let conversion = semantic_d4::import_with_layouts(
                &appearance,
                &self.resolver.diablo4_layouts(&owner),
                &self.resolver.import_options(&owner),
            )
            .map_err(|error| ModelError(error.to_string()))?;
            let model = conversion.value.widen();
            return compile_loaded(
                &model,
                &conversion.report,
                context,
                self.resolver.as_ref(),
                &owner,
                &children,
            )
            .await;
        }
        Err(ModelError(format!(
            "native Universal loading is not implemented for .{extension}"
        )))
    }
}

impl AssetLoader for ModelLoader {
    type Asset = ModelAsset;
    type Settings = ModelLoaderSettings;
    type Error = ModelError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &ModelLoaderSettings,
        context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, ModelError> {
        let attempt = ModelDependencyLoad::begin(&self.dependencies, context.path());
        let result = self
            .load_native(attempt.generation, reader, settings, context)
            .await;
        if result.is_ok() {
            attempt.commit();
        }
        result
    }
    fn extensions(&self) -> &[&str] {
        &["mdl", "mdx", "m2", "wmo", "m3", "app", "d3app", "d4app"]
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
