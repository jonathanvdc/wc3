//! Native asset loading without lowering through Warcraft III records.
use super::textures::{texture_requests, TextureImageKey};
use super::{compile_universal_model, CompiledUniversalModel};
use crate::{DecodedModel, ModelError};
use bevy::asset::{
    io::{AssetReaderError, Reader, SliceReader},
    AssetLoader, AssetPath, LoadContext, ReadAssetBytesError,
};
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt as _;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use wc3::model::common::{FileKey, IntoOwned as _};
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::{
    diablo::{d3, d4},
    m2, m3, wmo,
};
use wc3::model::io::{
    diablo::{d3 as semantic_d3, d4 as semantic_d4},
    m2 as semantic_m2, m3 as semantic_m3, wmo as semantic_wmo,
};
use wc3::model::{AddressMode, Model, Resource as ModelResource};
use wc3::visit_m2_model;

/// Resolves complete semantic identities to Bevy asset paths.
///
/// Providers may map file-data IDs or namespaced SNOs to any registered Bevy
/// asset source, including CASC. Candidate order is authoritative. Returning no
/// path leaves optional images unresolved and fails required native companions.
/// No filename is invented for an archive ID or an external animation hash.
pub trait UniversalResourceResolver: Send + Sync + 'static {
    /// Candidate paths, retaining any named Bevy asset source.
    fn candidates(
        &self,
        resource: &ModelResource<'_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>>;
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
pub struct UniversalPathResolver;
impl UniversalResourceResolver for UniversalPathResolver {
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
pub struct UniversalModelAsset {
    /// Shared CPU render definitions, ready for GPU preparation.
    pub compiled: CompiledUniversalModel,
    /// Resolved image variants keyed by semantic identity and effective color space.
    pub images: HashMap<TextureImageKey, Handle<Image>>,
    /// Optional images requiring an application resource provider or override.
    pub unresolved_images: Vec<ModelResource<'static>>,
}

/// Resource-provider configuration installed before `Wc3BevyPlugin` is added.
#[derive(Resource, Clone)]
pub struct UniversalAssetSettings {
    /// Native identity provider used by the model asset loader.
    pub resolver: Arc<dyn UniversalResourceResolver>,
}
impl Default for UniversalAssetSettings {
    fn default() -> Self {
        Self {
            resolver: Arc::new(UniversalPathResolver),
        }
    }
}

/// Native Universal loader. Applications can register a configured resolver.
#[derive(TypePath)]
pub struct UniversalModelLoader {
    resolver: Arc<dyn UniversalResourceResolver>,
}
impl Default for UniversalModelLoader {
    fn default() -> Self {
        Self::new(Arc::new(UniversalPathResolver))
    }
}
impl UniversalModelLoader {
    /// Uses an application-owned archive or resource-identity mapping.
    pub fn new(resolver: Arc<dyn UniversalResourceResolver>) -> Self {
        Self { resolver }
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
    resolver: &dyn UniversalResourceResolver,
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
                )))
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
    context: &mut LoadContext<'_>,
    resolver: &dyn UniversalResourceResolver,
    owner: &AssetPath<'_>,
) -> Result<UniversalModelAsset, ModelError> {
    let mut images = HashMap::new();
    let mut variants: HashMap<_, Handle<Image>> = HashMap::new();
    let mut encoded = HashMap::new();
    let mut unresolved_images = Vec::new();
    for request in texture_requests(model) {
        let id = request.id;
        let texture = request.texture;
        let image_key = request.key();
        let resource = texture.resource.clone().into_owned();
        let address_key = |mode| match mode {
            AddressMode::Repeat => 0u16,
            AddressMode::Clamp => 1,
            AddressMode::Mirror => 2,
            AddressMode::Unknown(value) => value as u16 + 3,
        };
        let srgb = request.color_space.is_srgb();
        let key = (
            resource.clone(),
            address_key(texture.sampler.u),
            address_key(texture.sampler.v),
            srgb,
        );
        if let Some(handle) = variants.get(&key) {
            images.insert(image_key, handle.clone());
            continue;
        }
        let mut resolved = None;
        for path in resolver.candidates(&resource, owner) {
            if !encoded.contains_key(&path) {
                match context.read_asset_bytes(path.clone()).await {
                    Ok(bytes) => {
                        encoded.insert(path.clone(), bytes);
                    }
                    Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => {
                        continue
                    }
                    Err(error) => {
                        return Err(ModelError(format!(
                            "cannot load image {resource:?}: {error}"
                        )))
                    }
                }
            }
            let mut sampler = ImageSamplerDescriptor::linear();
            let address = |mode| match mode {
                AddressMode::Repeat => ImageAddressMode::Repeat,
                AddressMode::Mirror => ImageAddressMode::MirrorRepeat,
                _ => ImageAddressMode::ClampToEdge,
            };
            sampler.address_mode_u = address(texture.sampler.u);
            sampler.address_mode_v = address(texture.sampler.v);
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
                format!("UniversalTexture{id:?}{:?}", request.color_space),
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
    let compiled = compile_universal_model(model, |request| images.get(&request.key()).cloned())?;
    Ok(UniversalModelAsset {
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
    resolver: &dyn UniversalResourceResolver,
    owner: &AssetPath<'_>,
) -> Result<UniversalModelAsset, ModelError> {
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
    let model = semantic_wmo::import(&bundle, &ImportOptions::default())
        .map_err(|error| ModelError(error.to_string()))?
        .value
        .widen();
    compile_loaded(&model, context, resolver, owner).await
}
async fn load_m3<E: m3::Envelope + Send + Sync>(
    document: m3::Document<'_, E>,
    context: &mut LoadContext<'_>,
    resolver: &dyn UniversalResourceResolver,
    owner: &AssetPath<'_>,
) -> Result<UniversalModelAsset, ModelError> {
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
    let model = semantic_m3::import(&bundle, &ImportOptions::default())
        .map_err(|error| ModelError(error.to_string()))?
        .value
        .widen();
    compile_loaded(&model, context, resolver, owner).await
}

impl AssetLoader for UniversalModelLoader {
    type Asset = UniversalModelAsset;
    type Settings = ();
    type Error = ModelError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, ModelError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| ModelError(error.to_string()))?;
        let owner = context.path().clone_owned();
        let extension = owner
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if extension == "mdx" || extension == "mdl" {
            let decoded = DecodedModel::decode(&bytes)?;
            return compile_loaded(
                &decoded.model.widen(),
                context,
                self.resolver.as_ref(),
                &owner,
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
                            let model = semantic_m2::import(&bundle, &ImportOptions::default())
                                .map_err(|error| ModelError(error.to_string()))?
                                .value
                                .widen();
                            Some(
                                compile_loaded(&model, context, self.resolver.as_ref(), &owner)
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
            let profile = self.resolver.wmo_profile(&owner).ok_or_else(|| ModelError("WMO loading requires explicit client layout provenance from UniversalResourceResolver::wmo_profile".into()))?;
            macro_rules! load {
                ($dialect:ty) => {
                    load_wmo::<$dialect>(&bytes, context, self.resolver.as_ref(), &owner).await
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
                    load_m3(document, context, self.resolver.as_ref(), &owner).await
                }
                m3::DynamicDocument::Md34(document) => {
                    load_m3(document, context, self.resolver.as_ref(), &owner).await
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
                    let model = semantic_d3::import(&bundle, &ImportOptions::default())
                        .map_err(|error| ModelError(error.to_string()))?
                        .value
                        .widen();
                    return compile_loaded(&model, context, self.resolver.as_ref(), &owner).await;
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
            let model = semantic_d4::import_with_layouts(
                &appearance,
                &self.resolver.diablo4_layouts(&owner),
                &ImportOptions::default(),
            )
            .map_err(|error| ModelError(error.to_string()))?
            .value
            .widen();
            return compile_loaded(&model, context, self.resolver.as_ref(), &owner).await;
        }
        Err(ModelError(format!(
            "native Universal loading is not implemented for .{extension}"
        )))
    }
    fn extensions(&self) -> &[&str] {
        &["mdl", "mdx", "m2", "wmo", "m3", "app", "d3app", "d4app"]
    }
}

#[cfg(test)]
#[path = "assets_tests.rs"]
mod tests;
