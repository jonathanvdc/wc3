//! Prepared external-effect images and renderer bindings.
use super::ribbon::RibbonMode;
use crate::diagnostics::{RenderDiagnostic, RenderDiagnosticKind, RenderDiagnosticSource};
use crate::effects::age_sprites_gpu::immutable::ImmutableAppearance;
use crate::effects::age_sprites_gpu::{AgeSpriteComposition, AgeSpriteStyle, SpriteUv};
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
use wc3::model::{AddressMode, ImageResource, Sampler};

/// Image binding within one external particle renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleTextureBinding {
    /// Primary diffuse color.
    Diffuse,
    /// Numeric alpha replacement lookup.
    AlphaRemap,
}
impl ParticleTextureBinding {
    fn slot(self) -> usize {
        match self {
            Self::Diffuse => 0,
            Self::AlphaRemap => 1,
        }
    }
}

/// One image interpretation requested by an external particle programme.
#[derive(Clone, Debug)]
pub struct ParticleTextureRequest<'a> {
    /// Zero-based renderer declaration within the programme.
    pub renderer: usize,
    /// Distinct binding, even when multiple images share a role or path.
    pub binding: ParticleTextureBinding,
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
    pub(crate) renderers: Arc<[ProgrammeRenderer]>,
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
pub(crate) struct ProgrammeRenderer {
    pub(crate) layer: EffectLayerId,
    pub(crate) immutable: Option<Arc<ImmutableAppearance>>,
    pub(crate) position: EffectFieldId,
    pub(crate) size: EffectFieldId,
    pub(crate) enabled: Option<EffectFieldId>,
    pub(crate) color: Option<EffectFieldId>,
    pub(crate) texture_id: Option<EffectFieldId>,
    pub(crate) geometry: RendererGeometry,
    pub(crate) atlas: [u32; 2],
    pub(crate) style: AgeSpriteStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RendererGeometry {
    Billboard {
        facing: SpriteFacing,
        axes: [Option<EffectFieldId>; 2],
        rotation: Option<EffectFieldId>,
    },
    Ribbon {
        mode: RibbonMode,
        axis: Option<EffectFieldId>,
        texture_u: Option<EffectFieldId>,
        cursor: Option<EffectFieldId>,
    },
}
impl RendererGeometry {
    pub(super) fn is_ribbon(self) -> bool {
        matches!(self, Self::Ribbon { .. })
    }
    pub(super) fn code(self) -> u32 {
        match self {
            Self::Billboard { facing, .. } => facing.code(),
            Self::Ribbon { mode, .. } => mode.code(),
        }
    }
    pub(super) fn axis_fields(self) -> [Option<EffectFieldId>; 2] {
        match self {
            Self::Billboard { axes, .. } => axes,
            Self::Ribbon { axis, .. } => [axis, None],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpriteFacing {
    Camera,
    Axial,
    Spheroid,
    Capsule,
    Planar,
}
impl SpriteFacing {
    pub(super) fn code(self) -> u32 {
        match self {
            Self::Camera => 0,
            Self::Axial => 2,
            Self::Spheroid => 3,
            Self::Capsule => 4,
            Self::Planar => 5,
        }
    }
    pub(super) fn longitudinal_axis(self) -> bool {
        matches!(self, Self::Axial | Self::Spheroid | Self::Capsule)
    }
}
impl ProgrammeRenderer {
    pub(super) fn capsule_geometry(&self) -> bool {
        matches!(
            self.geometry,
            RendererGeometry::Billboard {
                facing: SpriteFacing::Capsule,
                axes: [Some(_), _],
                ..
            }
        )
    }
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
                    prepared.immutable = super::immutable::prepare(&program, &prepared);
                    if program.immutable_recipe(prepared.layer).is_some() {
                        diagnostics.push(RenderDiagnostic::new(
                            RenderDiagnosticKind::Unverified, RenderDiagnosticSource::Model,
                            if prepared.immutable.is_some() {
                                "external billboard appearance can use immutable Graphics expressions; uncertified births and incoherent authorization retain CPU samples"
                            } else {
                                "external renderer retains CPU samples: immutable appearance requires billboard geometry, stationary visibility, and bounded generated expressions"
                            }));
                    }
                    if toggle(renderer, b"SoftParticles", false) {
                        diagnostics.push(RenderDiagnostic::new(
                            RenderDiagnosticKind::Unsupported,
                            RenderDiagnosticSource::Model,
                            "external particle soft intersections are not applied; retaining ordinary particle geometry",
                        ));
                    }
                    if let RendererGeometry::Billboard { facing, axes, .. } = prepared.geometry {
                        let missing_axes = match facing {
                            SpriteFacing::Camera => false,
                            SpriteFacing::Axial
                            | SpriteFacing::Spheroid
                            | SpriteFacing::Capsule => axes[0].is_none(),
                            SpriteFacing::Planar => axes.iter().any(Option::is_none),
                        };
                        if missing_axes {
                            diagnostics.push(RenderDiagnostic::new(
                            RenderDiagnosticKind::Approximation,
                            RenderDiagnosticSource::Model,
                            format!("external particle renderer {index} is missing facing axes; using a camera-facing quad"),
                        ));
                        }
                        if facing != SpriteFacing::Camera || axes.iter().all(Option::is_some) {
                            diagnostics.push(RenderDiagnostic::new(
                            RenderDiagnosticKind::Unverified,
                            RenderDiagnosticSource::Model,
                            "external particle authored-axis facing uses per-view geometry; exact native facing and degenerate-axis fidelity remain unverified",
                        ));
                        }
                    }
                    for request in image_requests(index, renderer) {
                        let binding = request.binding;
                        let image = resolve(request);
                        if image.is_none() {
                            diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Resource,
                                RenderDiagnosticSource::Model,
                                format!("external particle renderer {index} has an unresolved {binding:?} image")));
                            if binding == ParticleTextureBinding::AlphaRemap {
                                if let AgeSpriteComposition::Sampled { alpha_remap, .. }
                                | AgeSpriteComposition::Ribbon { alpha_remap, .. } =
                                    &mut prepared.style.composition
                                {
                                    *alpha_remap = false;
                                }
                            }
                        }
                        prepared.style.textures[binding.slot()] = image;
                    }
                    if toggle(renderer, b"AlphaRemap", false)
                        && renderer
                            .property(b"AlphaRemap.AlphaMap")
                            .and_then(|p| p.text.as_deref())
                            .is_none()
                    {
                        diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Resource,
                            RenderDiagnosticSource::Model,
                            format!("external particle renderer {index} enables alpha remap without an image; preserving diffuse alpha")));
                    }
                    match prepared.geometry {
                        RendererGeometry::Billboard { .. } => {
                            if toggle(renderer, b"AlphaRemap", false) {
                                diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Unverified,
                            RenderDiagnosticSource::Model,
                            "billboard alpha-remap V uses a stable slot-incarnation hash; native random fidelity remains unverified"));
                            }
                            if toggle(renderer, b"Atlas", false) {
                                diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Unverified,
                            RenderDiagnosticSource::Model,
                            "billboard atlas uses adjacent-frame RGBA crossfade; authored atlas blending-mode fidelity remains unverified"));
                            }
                        }
                        RendererGeometry::Ribbon { .. } => {
                            diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Unverified,
                            RenderDiagnosticSource::Model,
                            "external ribbons connect surviving particles by exact parent identity and birth order; degenerate cross sections use world +X and collapsed segments are omitted; native client fidelity remains unverified"));
                            for feature in [b"TransformUVs".as_slice(), b"CorrectDeformation"] {
                                if toggle(renderer, feature, false) {
                                    diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Unsupported,
                                    RenderDiagnosticSource::Model,
                                    "external ribbon UV programme and deformation corrections are not applied; retaining connected ribbon geometry"));
                                }
                            }
                        }
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
            .ok_or_else(|| ModelError::new(format!("unknown particle attribute {name:?}")))?;
        let value = value.into();
        if value.kind() != declaration.default.kind() {
            return Err(ModelError::new(format!(
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
    /// Color and data image interpretations needed by supported renderers.
    /// Use this before asynchronous image loading, then supply handles to [`Self::new`].
    pub fn texture_requests(
        program: &EffectProgram,
    ) -> impl Iterator<Item = ParticleTextureRequest<'_>> {
        program
            .renderers()
            .iter()
            .enumerate()
            .filter(move |(index, renderer)| {
                prepare(*index, renderer, program).ok().flatten().is_some()
            })
            .flat_map(|(index, renderer)| image_requests(index, renderer))
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
fn image_requests(
    index: usize,
    renderer: &EffectRenderer,
) -> impl Iterator<Item = ParticleTextureRequest<'_>> {
    [
        ParticleTextureBinding::Diffuse,
        ParticleTextureBinding::AlphaRemap,
    ]
    .into_iter()
    .filter_map(move |binding| {
        let (path, role, color_space) = match binding {
            ParticleTextureBinding::Diffuse => (
                renderer.primary_image()?,
                TextureRole::Color,
                TextureColorSpace::Srgb,
            ),
            ParticleTextureBinding::AlphaRemap => {
                if !toggle(renderer, b"AlphaRemap", false) {
                    return None;
                }
                let path = renderer.property(b"AlphaRemap.AlphaMap")?.text.as_deref()?;
                (path, TextureRole::Data, TextureColorSpace::Linear)
            }
        };
        Some(ParticleTextureRequest {
            renderer: index,
            binding,
            resource: ImageResource::Path(path.into()),
            role,
            color_space,
            sampler: match binding {
                ParticleTextureBinding::Diffuse => Sampler::default(),
                ParticleTextureBinding::AlphaRemap => Sampler {
                    u: AddressMode::Clamp,
                    v: AddressMode::Clamp,
                },
            },
        })
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
) -> Result<Option<ProgrammeRenderer>, &'static str> {
    if !toggle(renderer, b"EnableRendering", true) {
        return Ok(None);
    }
    let ribbon_geometry = renderer.class == 1 && toggle(renderer, b"GeometryRibbon", false);
    let family = if ribbon_geometry {
        b"popcorn/library/popcornfxcore/materials/default_ribbon.pkma".as_slice()
    } else {
        b"popcorn/library/popcornfxcore/materials/default_billboard.pkma".as_slice()
    };
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
    if !ribbon_geometry && (renderer.class != 0 || !toggle(renderer, b"GeometryBillboard", false)) {
        return Err("only billboard sprite geometry is supported");
    }
    for feature in [
        b"NormalBend".as_slice(),
        b"NormalWrap",
        b"DiffuseRamp",
        b"Emissive",
        b"TransformUVs",
        b"EnableSize2D",
        b"GeometryMesh",
        b"Distortion",
    ] {
        if toggle(renderer, feature, false) && !(ribbon_geometry && feature == b"TransformUVs") {
            return Err("an enabled material feature has no interpreter");
        }
    }
    let mode = property(renderer, b"BillboardingMode").map_or(0, |value| value.numeric[0]);
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
    let texture_id = custom(b"TextureID");
    let axis = |semantics: &[u32], names: &[&[u8]]| {
        renderer
            .inputs
            .iter()
            .find(|input| {
                match input
                    .additional_name
                    .as_deref()
                    .filter(|name| !name.is_empty())
                {
                    Some(name) => names.contains(&name),
                    None => semantics.contains(&input.semantic),
                }
            })
            .map(|input| input.field)
    };
    let axes = [
        axis(&[4, 8], &[b"Axis0", b"Axis"]),
        axis(&[5], &[b"Axis1", b"NormalAxis"]),
    ];
    let geometry = if ribbon_geometry {
        let mode = match mode {
            0 => RibbonMode::View,
            1 => RibbonMode::NormalAxis,
            2 => RibbonMode::SideAxis,
            _ => return Err("the ribbon facing mode is unsupported"),
        };
        if mode != RibbonMode::View && axes[0].is_none() {
            return Err("the ribbon facing mode requires Axis0");
        }
        let texture_u = if toggle(renderer, b"CustomTextureU", false) {
            Some(custom(b"TextureU").ok_or("custom ribbon UVs require TextureU")?)
        } else {
            None
        };
        RendererGeometry::Ribbon {
            mode,
            axis: if mode == RibbonMode::View {
                None
            } else {
                axes[0]
            },
            texture_u,
            cursor: custom(b"Cursor"),
        }
    } else {
        let facing = match mode {
            0 => SpriteFacing::Camera,
            2 => SpriteFacing::Axial,
            3 => SpriteFacing::Spheroid,
            4 => SpriteFacing::Capsule,
            5 => SpriteFacing::Planar,
            _ => return Err("the billboard facing mode is unsupported"),
        };
        RendererGeometry::Billboard {
            facing,
            axes,
            rotation,
        }
    };
    let (rotation, extra_scalars) = match geometry {
        RendererGeometry::Billboard { rotation, .. } => (rotation, [None, None]),
        RendererGeometry::Ribbon {
            texture_u, cursor, ..
        } => (None, [texture_u, cursor]),
    };
    let axes = geometry.axis_fields();
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
        (texture_id, ValueType::Float),
        (axes[0], ValueType::Float3),
        (axes[1], ValueType::Float3),
        (extra_scalars[0], ValueType::Float),
        (extra_scalars[1], ValueType::Float),
    ] {
        if let Some(id) = id {
            if fields.get(id.0 as usize).and_then(|field| field.value_type) != Some(kind) {
                return Err("a renderer stream has an unsupported value layout");
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
        property(renderer, b"Atlas.SubDiv").map_or([1, 1], |value| {
            [value.numeric[1].max(1), value.numeric[0].max(1)]
        })
    } else {
        [1, 1]
    };
    if atlas[0].checked_mul(atlas[1]).is_none() {
        return Err("atlas dimensions exceed the cell index range");
    }
    let uv = SpriteUv {
        rotate: toggle(renderer, b"RotateTexture", false),
        flip_u: toggle(renderer, b"FlipUVs", false) || toggle(renderer, b"TextureUVs.FlipU", false),
        flip_v: toggle(renderer, b"FlipUVs", false) || toggle(renderer, b"TextureUVs.FlipV", false),
    };
    let alpha_remap = toggle(renderer, b"AlphaRemap", false)
        && renderer
            .property(b"AlphaRemap.AlphaMap")
            .and_then(|property| property.text.as_deref())
            .is_some();
    Ok(Some(ProgrammeRenderer {
        immutable: None,
        layer: renderer.layer,
        position,
        size,
        enabled,
        color,
        texture_id,
        geometry,
        atlas,
        style: AgeSpriteStyle {
            textures: [None, None, None],
            blend,
            priority: renderer.draw_order as f32,
            composition: if geometry.is_ribbon() {
                AgeSpriteComposition::Ribbon { uv, alpha_remap }
            } else {
                AgeSpriteComposition::Sampled {
                    uv,
                    crossfade: toggle(renderer, b"Atlas", false),
                    alpha_remap,
                }
            },
            unlit: !toggle(renderer, b"Lit", false) && !toggle(renderer, b"LegacyLit", false),
            head: true,
            tail: false,
            uv_flip_v: !geometry.is_ribbon(),
            sort: SortPolicy::Far,
        },
    }))
}

#[cfg(test)]
#[path = "appearance_tests.rs"]
mod appearance_tests;
