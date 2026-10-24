//! Native asset loading without lowering through Warcraft III records.
use super::images::ImageLoadCache;
use super::particle_program::resolve_programmes;
use crate::child_models::{requests as child_model_requests, ChildModelBinding, ChildModelRole};
use crate::diagnostics::{RenderDiagnosticKind, RenderDiagnosticSource};
use crate::error::{ModelError, ModelLoadError};
use crate::textures::{texture_requests, TextureRequest, TextureRequestKey};
use crate::{
    compile_model_with_options, CompileOptions, CompiledModel, ParticleTextureRequest,
    RenderDiagnostic,
};
use bevy::asset::{
    io::{AssetReaderError, Reader},
    AssetLoader, AssetPath, LoadContext, ReadAssetBytesError,
};
use bevy::prelude::*;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt as _;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use wc3::model::common::FileKey;
use wc3::model::conversion::{ImportOptions, IssueKind, Report};
use wc3::model::formats::wmo;
use wc3::model::io::asset::{Format, Progress, Reader as NativeReader, Request};
use wc3::model::{FeatureId, ImageResource, Model, ResourceLocator as ModelResource};

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
    /// `index` is the zero-based position in the model's image dependency table,
    /// shared by every sampling and role variant of that image. MDLX imports
    /// retain one declaration per native bitmap entry, including equal
    /// declarations, in their original table order. Image selections can precede
    /// replaceable-category or archive identity mappings here. The
    /// request retains the complete selected input, original resource, color role,
    /// and effective sampler. Returning
    /// a different resource does not mutate the model or any shared image asset.
    fn texture_resource(
        &self,
        _index: usize,
        request: &TextureRequest<'_, '_>,
        _owner: &AssetPath<'_>,
    ) -> ImageResource<'static> {
        request.resource.clone().into_owned()
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
        let locator = match resource {
            ImageResource::FileDataId { id, .. } => Some(ModelResource::FileDataId(id)),
            resource => resource.try_into().ok(),
        };
        image_candidates(
            locator.map_or_else(Vec::new, |resource| self.candidates(&resource, owner)),
        )
    }
    /// Candidate child models for an authored attachment or particle emitter.
    /// Missing optional children remain visible as integration diagnostics.
    fn child_model_candidates(
        &self,
        _feature: FeatureId,
        _role: ChildModelRole,
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
    /// Baked external effect companions in provider lookup order.
    /// Authored `.pkfx` resources resolve to `.pkb` companions at each location.
    fn particle_effect_candidates(
        &self,
        _feature: FeatureId,
        resource: &ModelResource<'_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>> {
        self.candidates(resource, owner)
            .into_iter()
            .map(|path| {
                if path
                    .path()
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("pkfx"))
                {
                    AssetPath::from(path.path().with_extension("pkb"))
                        .with_source(path.source().clone_owned())
                } else {
                    path
                }
            })
            .collect()
    }
    /// Image interpretations owned by a prepared external effect.
    /// `owner` identifies the resolved baked companion, retaining its asset source.
    fn particle_effect_texture_candidates(
        &self,
        _feature: FeatureId,
        request: &ParticleTextureRequest<'_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>> {
        let resource: Option<ModelResource<'_>> = request.resource.clone().try_into().ok();
        image_candidates(
            resource.map_or_else(Vec::new, |resource| self.candidates(&resource, owner)),
        )
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
    pub images: HashMap<TextureRequestKey, Handle<Image>>,
    /// Optional images requiring an application resource provider or override.
    pub unresolved_images: Vec<ImageResource<'static>>,
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
async fn read_companion(
    context: &mut LoadContext<'_>,
    resolver: &dyn ResourceResolver,
    owner: &AssetPath<'_>,
    request: &Request,
) -> Result<Vec<u8>, ModelLoadError> {
    let candidates = resolver.candidates(&file_resource(&request.key), owner);
    for path in &candidates {
        match context.read_asset_bytes(path.clone()).await {
            Ok(bytes) => return Ok(bytes),
            Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => {}
            Err(source) => {
                return Err(ModelLoadError::CompanionRead {
                    owner: owner.clone_owned(),
                    request: Box::new(request.clone()),
                    path: Box::new(path.clone()),
                    source: Box::new(source),
                });
            }
        }
    }
    Err(ModelLoadError::MissingCompanion {
        owner: owner.clone_owned(),
        request: Box::new(request.clone()),
        candidates,
    })
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
        .map(|issue| {
            RenderDiagnostic::new(
                if issue.kind == IssueKind::Dropped {
                    RenderDiagnosticKind::Unsupported
                } else {
                    RenderDiagnosticKind::Approximation
                },
                RenderDiagnosticSource::NativeImport {
                    path: issue.path.to_string(),
                },
                format!(
                    "native import {:?} at {}: {}",
                    issue.kind, issue.path, issue.description
                ),
            )
        })
        .collect::<Vec<_>>();
    let mut resolved_paths: HashMap<AssetPath<'static>, Handle<ModelAsset>> = HashMap::new();
    for (key, resource_id) in child_model_requests(model) {
        if options.child_models.contains_key(&key) {
            continue;
        }
        let resource = model
            .model_resource(resource_id)
            .expect("validated child model resource")
            .clone()
            .into_owned();
        let mut resolved = false;
        for path in resolver.child_model_candidates(key.feature, key.role, &resource, owner) {
            if let Some(handle) = resolved_paths.get(&path) {
                options
                    .child_models
                    .insert(key, ChildModelBinding::new(handle.clone()));
                resolved = true;
                break;
            }
            match context.read_asset_bytes(path.clone()).await {
                Ok(_) => {}
                Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => {
                    continue;
                }
                Err(error) => {
                    return Err(ModelError::new(format!(
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
                child_diagnostics.push(RenderDiagnostic::new(
                    RenderDiagnosticKind::Resource,
                    RenderDiagnosticSource::Feature(key.feature),
                    format!(
                        "cyclic child model reference {resource:?} for {:?} at {path} is not instantiated", key.role
                    ),
                ));
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
                .load(path.clone());
            resolved_paths.insert(path, handle.clone());
            options
                .child_models
                .insert(key, ChildModelBinding::new(handle));
            resolved = true;
            break;
        }
        if !resolved {
            child_diagnostics.push(RenderDiagnostic::new(
                RenderDiagnosticKind::Resource,
                RenderDiagnosticSource::Feature(key.feature),
                format!(
                    "optional {:?} child model {resource:?} could not be resolved",
                    key.role
                ),
            ));
        }
    }
    let mut images = HashMap::new();
    let mut image_cache = ImageLoadCache::default();
    let indices: HashMap<_, _> = model
        .dependencies()
        .images()
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id, index))
        .collect();
    let mut unresolved_images = Vec::new();
    for request in texture_requests(model) {
        let id = request.texture;
        let image_key = request.key();
        let resource = request.resource.clone().into_owned();
        let candidates = resolver.texture_candidates(indices[&id.resource], &request, owner);
        let resolved = image_cache
            .resolve(context, candidates, request.color_space, request.sampler)
            .await?;
        if let Some(handle) = resolved {
            images.insert(image_key, handle);
        } else if !unresolved_images.contains(&resource) {
            unresolved_images.push(resource);
        }
    }
    child_diagnostics.extend(
        resolve_programmes(
            model,
            &mut options,
            context,
            resolver,
            owner,
            &mut image_cache,
        )
        .await?,
    );
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

impl ModelLoader {
    async fn load_native(
        &self,
        generation: u64,
        reader: &mut dyn Reader,
        settings: &ModelLoaderSettings,
        context: &mut LoadContext<'_>,
    ) -> Result<ModelAsset, ModelLoadError> {
        let owner = context.path().clone_owned();
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|source| ModelLoadError::RootRead {
                owner: owner.clone(),
                source,
            })?;
        let extension = owner
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let format = match extension.as_str() {
            "mdx" | "mdl" => Format::Mdlx,
            "m2" => Format::M2,
            "wmo" => Format::Wmo(self.resolver.wmo_profile(&owner).ok_or_else(|| {
                ModelLoadError::MissingWmoProfile {
                    owner: owner.clone(),
                }
            })?),
            _ => {
                return Err(ModelLoadError::UnsupportedExtension { owner, extension });
            }
        };
        let mut native = NativeReader::new(
            format,
            owner_key(&owner),
            bytes,
            self.resolver.import_options(&owner),
        );
        let conversion = loop {
            match native.poll().map_err(|source| ModelLoadError::Native {
                owner: owner.clone(),
                source: Box::new(source),
            })? {
                Progress::Pending(request) => {
                    let bytes =
                        read_companion(context, self.resolver.as_ref(), &owner, &request).await?;
                    native
                        .provide(&request, bytes)
                        .map_err(|source| ModelLoadError::Native {
                            owner: owner.clone(),
                            source: Box::new(source),
                        })?;
                }
                Progress::Ready(conversion) => break conversion,
            }
        };
        let children = ChildModelContext {
            ancestry: &settings.ancestry,
            graph: &self.dependencies,
            generation,
        };
        compile_loaded(
            &conversion.value,
            &conversion.report,
            context,
            self.resolver.as_ref(),
            &owner,
            &children,
        )
        .await
        .map_err(|source| ModelLoadError::Compile { owner, source })
    }
}

impl AssetLoader for ModelLoader {
    type Asset = ModelAsset;
    type Settings = ModelLoaderSettings;
    type Error = ModelLoadError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &ModelLoaderSettings,
        context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, ModelLoadError> {
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
        &["mdl", "mdx", "m2", "wmo"]
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
