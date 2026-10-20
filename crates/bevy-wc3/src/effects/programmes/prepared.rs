//! Prepared external-effect images and renderer bindings.
use crate::diagnostics::{RenderDiagnostic, RenderDiagnosticKind, RenderDiagnosticSource};
use crate::effects::age_sprites_gpu::{AgeSpriteComposition, AgeSpriteStyle};
use crate::effects::render::SortPolicy;
use crate::textures::{TextureColorSpace, TextureRole};
use crate::ModelError;
use bevy::prelude::*;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use wc3::model::scene::popcorn::{
    EffectAttributeId, EffectFieldId, EffectLayerId, EffectLimits, EffectProgram, EffectRenderer,
    EffectRendererProperty, Value, ValueType,
};
use wc3::model::values::model::ParticleBlend;
use wc3::model::{ImageResource, Sampler};

/// One image interpretation requested by an external particle programme.
#[derive(Clone, Debug)]
pub struct ParticleTextureRequest<'a> {
    /// Zero-based renderer declaration within the programme.
    pub renderer: usize,
    /// External image identity, independent of a model's dependency table.
    pub resource: ImageResource<'a>,
    /// Purpose of the image.
    pub role: TextureRole,
    /// Effective GPU transfer function.
    pub color_space: TextureColorSpace,
    /// Effective sampling controls.
    pub sampler: Sampler,
}

/// Shared compiled particle programme and its prepared Bevy renderers.
#[derive(Clone)]
pub struct ParticleEffect {
    pub(crate) program: EffectProgram,
    pub(crate) renderers: Arc<[SpriteRenderer]>,
    pub(crate) limits: EffectLimits,
    pub(crate) overrides: Arc<Vec<(EffectAttributeId, Value)>>,
    diagnostics: Arc<[RenderDiagnostic]>,
}
impl Debug for ParticleEffect {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("ParticleEffect")
            .field("program", &self.program)
            .field("renderers", &self.renderers.len())
            .field("diagnostics", &self.diagnostics)
            .finish()
    }
}
#[derive(Clone)]
pub(crate) struct SpriteRenderer {
    pub(crate) layer: EffectLayerId,
    pub(crate) position: EffectFieldId,
    pub(crate) size: EffectFieldId,
    pub(crate) enabled: Option<EffectFieldId>,
    pub(crate) rotation: Option<EffectFieldId>,
    pub(crate) color: Option<EffectFieldId>,
    pub(crate) cursor: Option<EffectFieldId>,
    pub(crate) atlas: [u32; 2],
    pub(crate) style: AgeSpriteStyle,
}
impl ParticleEffect {
    /// Prepare renderer bindings once, resolving external images through the caller.
    /// Unsupported declarations and missing images remain explicit in diagnostics.
    pub fn new(
        program: EffectProgram,
        mut resolve: impl FnMut(ParticleTextureRequest<'_>) -> Option<Handle<Image>>,
    ) -> Self {
        let mut renderers = Vec::new();
        let mut diagnostics = Vec::new();
        for (index, renderer) in program.renderers().iter().enumerate() {
            match prepare(index, renderer, &program) {
                Ok(Some(mut prepared)) => {
                    if let Some(request) = image_request(index, renderer) {
                        let image = resolve(request);
                        if image.is_none() {
                            diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Resource, RenderDiagnosticSource::Model, format!("external particle renderer {index} has an unresolved diffuse image; using white")));
                        }
                        prepared.style.textures[0] = image;
                    }
                    renderers.push(prepared);
                }
                Ok(None) => {}
                Err(message) => diagnostics.push(RenderDiagnostic::new(
                    RenderDiagnosticKind::Unsupported,
                    RenderDiagnosticSource::Model,
                    format!("external particle renderer {index}: {message}"),
                )),
            }
        }
        Self {
            program,
            limits: EffectLimits::default(),
            overrides: Arc::default(),
            renderers: renderers.into(),
            diagnostics: diagnostics.into(),
        }
    }
    /// Override an exposed programme attribute for every new model instance.
    /// The name and value layout are checked during preparation. Explicit values
    /// take precedence over model controls bound to the same attribute.
    pub fn with_attribute(
        mut self,
        name: &str,
        value: impl Into<Value>,
    ) -> Result<Self, ModelError> {
        let (index, declaration) = self
            .program
            .attributes()
            .iter()
            .enumerate()
            .find(|(_, attribute)| attribute.name.as_ref() == name.as_bytes())
            .ok_or_else(|| ModelError(format!("unknown particle attribute {name:?}")))?;
        let value = value.into();
        if value.kind() != declaration.default.kind() {
            return Err(ModelError(format!(
                "particle attribute {name:?} requires {:?}, received {:?}",
                declaration.default.kind(),
                value.kind()
            )));
        }
        let id = EffectAttributeId(index as u32);
        let overrides = Arc::make_mut(&mut self.overrides);
        if let Some((_, target)) = overrides.iter_mut().find(|(key, _)| *key == id) {
            *target = value;
        } else {
            overrides.push((id, value));
        }
        Ok(self)
    }
    /// Select finite instance capacities and simulation work limits.
    /// Invalid limits appear as a checked execution error on `ParticleProgramEmitter`.
    pub fn with_limits(mut self, limits: EffectLimits) -> Self {
        self.limits = limits;
        self
    }
    /// Capacities used when creating independent instances.
    pub fn limits(&self) -> EffectLimits {
        self.limits
    }
    /// Immutable renderer-independent programme, shared by all model instances.
    pub fn program(&self) -> &EffectProgram {
        &self.program
    }
    /// Preparation limitations, with a programme-wide source until attached to a model.
    pub fn diagnostics(&self) -> &[RenderDiagnostic] {
        &self.diagnostics
    }
    /// Diffuse image interpretations needed by supported renderers.
    /// Use this before asynchronous image loading, then supply handles to [`Self::new`].
    pub fn texture_requests(
        program: &EffectProgram,
    ) -> impl Iterator<Item = ParticleTextureRequest<'_>> {
        program
            .renderers()
            .iter()
            .enumerate()
            .filter_map(move |(index, renderer)| {
                prepare(index, renderer, program).ok().flatten()?;
                image_request(index, renderer)
            })
    }
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        for renderer in Arc::make_mut(&mut self.renderers) {
            for handle in renderer.style.textures.iter_mut().flatten() {
                if let Some(next) = map(handle) {
                    *handle = next;
                }
            }
        }
    }
}
fn image_request(index: usize, renderer: &EffectRenderer) -> Option<ParticleTextureRequest<'_>> {
    let path = renderer.primary_image()?;
    Some(ParticleTextureRequest {
        renderer: index,
        resource: ImageResource::Path(path.into()),
        role: TextureRole::Color,
        color_space: TextureColorSpace::Srgb,
        sampler: Sampler::default(),
    })
}
fn property<'a>(renderer: &'a EffectRenderer, name: &[u8]) -> Option<&'a EffectRendererProperty> {
    renderer.property(name)
}
fn toggle(renderer: &EffectRenderer, name: &[u8], default: bool) -> bool {
    property(renderer, name).map_or(default, |value| value.numeric[0] != 0)
}
fn prepare(
    _index: usize,
    renderer: &EffectRenderer,
    program: &EffectProgram,
) -> Result<Option<SpriteRenderer>, &'static str> {
    if !toggle(renderer, b"EnableRendering", true) {
        return Ok(None);
    }
    let family = b"popcorn/library/popcornfxcore/materials/default_billboard.pkma";
    if renderer.feature_set.len() != family.len()
        || !renderer
            .feature_set
            .iter()
            .zip(family)
            .all(|(byte, expected)| {
                let byte = if *byte == b'\\' {
                    b'/'
                } else {
                    byte.to_ascii_lowercase()
                };
                byte == *expected
            })
    {
        return Err("the material feature family has no renderer interpreter");
    }
    if renderer.class != 0 || !toggle(renderer, b"GeometryBillboard", false) {
        return Err("only billboard sprite geometry is supported");
    }
    for feature in [
        b"SoftParticles".as_slice(),
        b"AlphaRemap",
        b"NormalBend",
        b"NormalWrap",
        b"DiffuseRamp",
        b"Emissive",
        b"TransformUVs",
        b"EnableSize2D",
        b"FlipUVs",
        b"RotateTexture",
        b"TextureUVs.FlipU",
        b"TextureUVs.FlipV",
        b"GeometryMesh",
        b"Distortion",
    ] {
        if toggle(renderer, feature, false) {
            return Err("an enabled material feature has no interpreter");
        }
    }
    if property(renderer, b"BillboardingMode").is_some_and(|value| value.numeric[0] != 0) {
        return Err("the billboard facing mode is unsupported");
    }
    let field = |semantic| {
        renderer
            .inputs
            .iter()
            .find(|input| input.semantic == semantic)
            .map(|input| input.field)
    };
    let custom = |name: &[u8]| {
        renderer
            .inputs
            .iter()
            .find(|input| input.additional_name.as_deref() == Some(name))
            .map(|input| input.field)
    };
    let position = field(0).ok_or("missing position stream")?;
    let size = field(1).ok_or("missing size stream")?;
    let enabled = field(2);
    let rotation = field(6);
    let color = custom(b"Color");
    let cursor = custom(b"TextureID");
    let fields = &program
        .layers()
        .get(renderer.layer.0 as usize)
        .ok_or("invalid owning layer")?
        .fields;
    for (id, kind) in [
        (Some(position), ValueType::Float3),
        (Some(size), ValueType::Float),
        (enabled, ValueType::Bool),
        (rotation, ValueType::Float),
        (color, ValueType::Float4),
        (cursor, ValueType::Float),
    ] {
        if let Some(id) = id {
            if fields.get(id.0 as usize).and_then(|field| field.value_type) != Some(kind) {
                return Err("a sprite stream has an unsupported value layout");
            }
        }
    }
    let blend = if !toggle(renderer, b"Transparent", false) {
        ParticleBlend::Opaque
    } else {
        match property(renderer, b"Transparent.Type").map_or(0, |value| value.numeric[0]) {
            0 => ParticleBlend::Add,
            1 => ParticleBlend::NoAlphaAdd,
            2 => ParticleBlend::Alpha,
            3 => ParticleBlend::BlendAdd,
            4 => ParticleBlend::Opaque,
            5 => ParticleBlend::AlphaKey,
            _ => return Err("the transparent blend mode is unsupported"),
        }
    };
    let atlas = if toggle(renderer, b"Atlas", false) {
        if property(renderer, b"Atlas.Source").is_some_and(|value| value.numeric[0] != 1) {
            return Err("the atlas source is unsupported");
        }
        if property(renderer, b"Atlas.Blending").is_some_and(|value| value.numeric[0] != 0) {
            return Err("interpolated atlas frames are unsupported");
        }
        property(renderer, b"Atlas.SubDiv").map_or([1, 1], |value| {
            [value.numeric[1].max(1), value.numeric[0].max(1)]
        })
    } else {
        [1, 1]
    };
    if atlas[0].checked_mul(atlas[1]).is_none() {
        return Err("atlas dimensions exceed the cell index range");
    }
    Ok(Some(SpriteRenderer {
        layer: renderer.layer,
        position,
        size,
        enabled,
        rotation,
        color,
        cursor,
        atlas,
        style: AgeSpriteStyle {
            textures: [None, None, None],
            blend,
            priority: renderer.draw_order as f32,
            composition: AgeSpriteComposition::Single,
            unlit: !toggle(renderer, b"Lit", false) && !toggle(renderer, b"LegacyLit", false),
            head: true,
            tail: false,
            uv_flip_v: true,
            sort: SortPolicy::Far,
        },
    }))
}
