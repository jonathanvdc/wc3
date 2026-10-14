//! Warcraft shader layers reference shared images while external transform selectors stay explicit.
use super::shader_values::{
    WarcraftFilter, WarcraftMaterialFlags, WarcraftShader, WarcraftShadingFlags,
    WarcraftTextureSlot,
};
use super::ShaderLayerId;
use crate::model::semantic::{Link, Scope, TextureId, ValidateLinks};
use std::borrow::Cow;
/// Texture-animation identity interpreted in the external Warcraft III source material.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WarcraftTransformSelection(pub u32);
/// Native input assigned to one high-definition shader role.
#[derive(Clone, Debug, Default)]
pub struct WarcraftInput {
    /// Shared image assigned to this shader role.
    pub texture: Option<TextureId>,
    /// Role supplied to the shader permutation.
    pub slot: WarcraftTextureSlot,
}
/// One authoritative image assignment mode for a Warcraft shader layer.
#[derive(Clone, Debug)]
pub enum WarcraftTextures {
    /// One image assigned without an explicit role table.
    Single(Option<TextureId>),
    /// Ordered shader-role image assignments; this table must be nonempty.
    Slots(Vec<WarcraftInput>),
}
impl Default for WarcraftTextures {
    fn default() -> Self {
        Self::Single(None)
    }
}
impl ValidateLinks for WarcraftTextures {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Single(texture) => texture.visit_links(visitor),
            Self::Slots(inputs) => {
                if inputs.is_empty() {
                    visitor(Link::InvalidValue);
                }
                for input in inputs {
                    input.texture.visit_links(visitor);
                }
            }
        }
    }
}
/// Independently authored Warcraft III shader layer.
#[derive(Clone, Debug)]
pub struct WarcraftLayer {
    id: ShaderLayerId,
    /// Color compositing operation.
    pub blend: WarcraftFilter,
    /// Native layer shading behavior.
    pub shading: WarcraftShadingFlags,
    /// Image assignment mode; role inputs replace a single primary image.
    pub textures: WarcraftTextures,
    /// External texture-coordinate animation selection.
    pub transform: WarcraftTransformSelection,
    /// Texture coordinate set identity.
    pub coordinate_set: u32,
    /// Opacity multiplier.
    pub opacity: f32,
    /// Self-illumination multiplier.
    pub emissive: f32,
    /// Fresnel reflection color.
    pub fresnel_color: [f32; 3],
    /// Fresnel opacity contribution.
    pub fresnel_opacity: f32,
    /// Team-color contribution to the Fresnel term.
    pub fresnel_team_color: f32,
    /// Shader permutation prefix.
    pub shader: WarcraftShader,
    /// Selects the high-definition layer representation independently of the shader prefix.
    pub high_definition: bool,
}

impl Default for WarcraftLayer {
    fn default() -> Self {
        Self {
            id: Scope::default().allocate(),
            blend: WarcraftFilter::default(),
            shading: WarcraftShadingFlags::default(),
            textures: WarcraftTextures::default(),
            transform: WarcraftTransformSelection::default(),
            coordinate_set: 0,
            opacity: 1.,
            emissive: 0.,
            fresnel_color: [0.; 3],
            fresnel_opacity: 0.,
            fresnel_team_color: 0.,
            shader: WarcraftShader::default(),
            high_definition: false,
        }
    }
}
impl WarcraftLayer {
    /// Stable contribution identity preserved when the layer moves.
    pub fn id(&self) -> ShaderLayerId {
        self.id
    }
    /// Creates a shader contribution with one explicit image assignment mode.
    pub fn new(textures: WarcraftTextures) -> Self {
        Self {
            textures,
            ..Default::default()
        }
    }
}

/// Native Warcraft III pass retained independently of the common WEM shader.
#[derive(Clone, Debug, Default)]
pub struct WarcraftMaterial<'a> {
    /// Native draw-order priority plane.
    pub priority_plane: i32,
    /// Native material behavior.
    pub flags: WarcraftMaterialFlags,
    /// Authored native shader label.
    pub shader: Cow<'a, str>,
    /// Ordered compositing layers.
    pub layers: Vec<WarcraftLayer>,
}
impl ValidateLinks for WarcraftMaterial<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for layer in &self.layers {
            layer.textures.visit_links(visitor);
        }
        if self.layers.iter().any(|layer| {
            [
                layer.opacity,
                layer.emissive,
                layer.fresnel_opacity,
                layer.fresnel_team_color,
            ]
            .iter()
            .chain(&layer.fresnel_color)
            .any(|v| !v.is_finite())
        }) {
            visitor(Link::InvalidValue);
        }
    }
}

impl WarcraftMaterial<'_> {
    /// Checks authored coefficients independently of native table resolution.
    pub fn validate(&self) -> Result<(), crate::model::semantic::Error> {
        let mut valid = true;
        self.visit_links(&mut |link| {
            if matches!(link, Link::InvalidValue) {
                valid = false;
            }
        });
        if valid {
            Ok(())
        } else {
            Err(crate::model::semantic::Error::new(
                "wem.material.native.warcraft",
                "nonfinite layer coefficient",
            ))
        }
    }
}
