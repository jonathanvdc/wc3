//! Model resource choices shared by the interactive viewer and capture tool.
use bevy::asset::AssetPath;
use bevy_wc3::{CompileOptions, PathResolver, ResourceResolver, TextureRequest};
use std::collections::HashMap;
use wc3::model::conversion::{ImportOptions, LossPolicy, UnknownPolicy};
use wc3::model::formats::wmo::Profile as WmoProfile;
use wc3::model::scene::popcorn::{ExecutionOptions, ExecutionPreference, NumericPolicy};
use wc3::model::{ImageResource, Model, ResourceLocator as ModelResource};

#[derive(Default)]
pub struct ModelResolver {
    pub allow_loss: bool,
    pub file_ids: HashMap<u32, String>,
    pub bitmaps: HashMap<usize, String>,
    pub replaceables: HashMap<u32, String>,
    pub wmo_profile: Option<WmoProfile>,
    pub particle_execution: Option<ExecutionOptions>,
}
impl ResourceResolver for ModelResolver {
    fn compile_options(&self, _owner: &AssetPath<'_>, _model: &Model<'_>) -> CompileOptions {
        let mut options = CompileOptions::default();
        if let Some(execution) = self.particle_execution {
            options.particle_execution = execution;
        }
        options
    }

    fn import_options(&self, _owner: &AssetPath<'_>) -> ImportOptions {
        if self.allow_loss {
            ImportOptions {
                loss_policy: LossPolicy::Drop,
                unknown: UnknownPolicy::Drop,
                ..Default::default()
            }
        } else {
            ImportOptions::default()
        }
    }

    fn texture_resource(
        &self,
        index: usize,
        request: &TextureRequest<'_, '_>,
        _owner: &AssetPath<'_>,
    ) -> ImageResource<'static> {
        let path = self.bitmaps.get(&index).or_else(|| {
            if let ImageResource::Replaceable { id, .. } = request.resource {
                self.replaceables.get(id)
            } else {
                None
            }
        });
        path.map_or_else(
            || request.resource.clone().into_owned(),
            |path| ImageResource::Path(path.as_bytes().to_vec().into()),
        )
    }
    fn candidates(
        &self,
        resource: &ModelResource<'_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>> {
        if let ModelResource::FileDataId(id) = resource {
            return self
                .file_ids
                .get(id)
                .map(|path| {
                    vec![AssetPath::from(path.clone()).with_source(owner.source().clone_owned())]
                })
                .unwrap_or_default();
        }
        PathResolver.candidates(resource, owner)
    }
    fn wmo_profile(&self, _owner: &AssetPath<'_>) -> Option<WmoProfile> {
        self.wmo_profile
    }
}

/// Select a native WMO build layout explicitly.
pub fn parse_wmo_profile(name: &str) -> Result<WmoProfile, &'static str> {
    match name {
        "classic" => Ok(WmoProfile::Classic),
        "legion" => Ok(WmoProfile::Legion),
        "bfa" => Ok(WmoProfile::Bfa),
        "shadowlands-pre92" => Ok(WmoProfile::ShadowlandsPre92),
        "shadowlands" => Ok(WmoProfile::Shadowlands),
        "dragonflight" => Ok(WmoProfile::Dragonflight),
        "warwithin" => Ok(WmoProfile::WarWithin),
        _ => Err("unsupported WMO profile"),
    }
}

/// `simd-jit` requests Automatic: eligible tiles use SIMD, with checked scalar fallback.
#[allow(
    dead_code,
    reason = "Some examples sharing this module use default particle execution"
)]
pub fn parse_particle_execution(name: &str) -> Result<ExecutionOptions, &'static str> {
    let preference = match name {
        "interpreter" => ExecutionPreference::Interpreter,
        "scalar-jit" => ExecutionPreference::ScalarJit,
        "simd-jit" => ExecutionPreference::Automatic,
        "immutable" => ExecutionPreference::ImmutableAge,
        _ => {
            return Err("particle-execution must be interpreter, scalar-jit, simd-jit or immutable")
        }
    };
    Ok(ExecutionOptions {
        preference,
        numeric_policy: NumericPolicy::Graphics,
    })
}
