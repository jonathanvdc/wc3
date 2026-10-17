//! Diablo IV shader inputs use checked image-resource identities and explicit parameter selections.
use super::bindings::TextureBinding;
use super::parameters::{OrderedParameters, ParameterEntry, ShaderValue};
use super::program::MaterialProgram;
use crate::model::profiles::SurfacePayload;
use crate::model::{Error, ImageResourceId, Link, PropertyIdentityKey, ValidateLinks};

/// A texture input retains negative source selections independently of image handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureSelection {
    /// An image defined in the owning model.
    Image(ImageResourceId),
    /// A negative native selection whose precise absent-image convention is unresolved.
    Absent(i32),
}
impl Default for TextureSelection {
    fn default() -> Self {
        Self::Absent(-1)
    }
}
impl ValidateLinks for TextureSelection {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Image(id) => id.visit_links(visitor),
            Self::Absent(code) if *code >= 0 => visitor(Link::InvalidValue),
            _ => {}
        }
    }
}
/// An authored texture animation-state selection.
#[derive(Clone, Debug, Default)]
pub struct TexturePlayback {
    /// Animation-state index, including unresolved negative sentinels.
    pub animation_state_index: i32,
}
/// MaterialStep-defined texture role and independently authored control.
#[derive(Clone, Debug, Default)]
pub struct TextureContribution {
    /// Producer-defined shader texture semantic.
    pub semantic: i32,
    /// Independently authored unresolved control.
    pub unclassified_control: i32,
}
/// A Diablo IV image with playback and shader contribution declarations.
pub type TextureInput = TextureBinding<TextureSelection, (), TexturePlayback, TextureContribution>;
impl ValidateLinks for TexturePlayback {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for TextureContribution {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
/// An explicitly absent parameter graph, retaining the native negative selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GraphAbsence(i32);
impl GraphAbsence {
    /// Retains an absent selection; active graph indices require a separate graph vocabulary.
    pub fn new(code: i32) -> Result<Self, Error> {
        if code < 0 {
            Ok(Self(code))
        } else {
            Err(Error::new(
                "d4.material.graph",
                "active parameter graphs require an established semantic mapping",
            ))
        }
    }
    /// Native negative selection retained by this declaration.
    pub fn code(self) -> i32 {
        self.0
    }
}
impl Default for GraphAbsence {
    fn default() -> Self {
        Self(-1)
    }
}
/// Declarations shared by scalar and vector shader parameters.
#[derive(Clone, Copy, Debug, Default)]
pub struct ParameterControls {
    /// Explicitly absent parameter graph selection.
    pub graph: GraphAbsence,
    /// Authored light-group selection.
    pub light_group: i32,
}
/// An interpreted scalar parameter in the Diablo IV shader-value namespace.
pub type ScalarValue = ParameterEntry<ShaderValue, f32, ParameterControls>;
/// An interpreted vector parameter in the Diablo IV shader-value namespace.
pub type VectorValue = ParameterEntry<ShaderValue, [f32; 4], ParameterControls>;
/// One ordered runtime material parameter set.
#[derive(Clone, Debug, Default)]
pub struct MaterialValues {
    /// Scalar parameter values in authored order.
    pub scalars: OrderedParameters<ShaderValue, f32, ParameterControls>,
    /// Vector parameter values in authored order.
    pub vectors: OrderedParameters<ShaderValue, [f32; 4], ParameterControls>,
}
/// Diablo IV shader declaration, independent of native material and appearance layouts.
#[derive(Clone, Debug, Default)]
pub struct MaterialStep {
    /// Independently authored material-resource declarations, when imported from a material document.
    pub asset: Option<MaterialAssetControls>,
    /// External shader-map SNO in the Diablo IV namespace.
    pub shader_map: i32,
    /// External material-value-set override, including absent selections.
    pub value_set_override: i32,
    /// Authored sorting priority.
    pub sort_priority: i32,
    /// Ordered shader texture inputs referencing shared images.
    pub textures: Vec<TextureInput>,
    /// Ordered material parameter sets.
    pub values: Vec<MaterialValues>,
    /// Declared number of texture-animation states.
    pub texture_animation_state_count: i32,
    /// Source graph declaration, preserving unresolved integer choices.
    pub has_graph: i32,
}
impl ValidateLinks for MaterialStep {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.asset.as_ref().is_some_and(|a| {
            a.unclassified_scalars
                .iter()
                .chain([
                    &a.duration,
                    &a.outro_duration,
                    &a.stiffness,
                    &a.gravity_magnitude,
                ])
                .any(|v| !v.is_finite())
        }) {
            visitor(Link::InvalidValue);
        }
        for input in &self.textures {
            input.visit_links(visitor);
        }
        self.values.visit_links(visitor);
        if self.texture_animation_state_count < 0 {
            visitor(Link::InvalidValue);
        }
    }
}
impl SurfacePayload for MaterialStep {
    fn validate(&self) -> Result<(), Error> {
        let mut invalid = false;
        self.visit_links(&mut |link| {
            if matches!(link, Link::InvalidValue) {
                invalid = true;
            }
        });
        if invalid {
            Err(Error::new(
                "d4.material",
                "nonfinite material value or invalid texture selection",
            ))
        } else {
            Ok(())
        }
    }
}

/// Authoring controls belonging to a standalone Diablo IV material resource.
#[derive(Clone, Debug, Default)]
pub struct MaterialAssetControls {
    /// External material resource identity.
    pub sno_id: i32,
    /// Root record options.
    pub record_flags: u32,
    /// Material resource options.
    pub flags: u32,
    /// External surface resource selection.
    pub surface: i32,
    /// Independently authored scalar controls with unresolved meanings.
    pub unclassified_scalars: [f32; 3],
    /// Independently authored unsigned control with unresolved meaning.
    pub unclassified_unsigned: u32,
    /// Independently authored signed controls with unresolved meanings.
    pub unclassified_signed: [i32; 2],
    /// Authored material duration.
    pub duration: f32,
    /// Authored material outro duration.
    pub outro_duration: f32,
    /// Authored deformation stiffness.
    pub stiffness: f32,
    /// Authored gravity magnitude.
    pub gravity_magnitude: f32,
    /// Authored animated influence selections.
    pub animated_influences: [i32; 8],
}

impl ValidateLinks for GraphAbsence {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for ParameterControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.graph.visit_links(visitor);
    }
}
impl ValidateLinks for MaterialValues {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.scalars.visit_links(visitor);
        self.vectors.visit_links(visitor);
    }
}

impl SurfacePayload for MaterialProgram<MaterialStep> {
    fn supports_property(&self, _: &PropertyIdentityKey) -> bool {
        false
    }
    fn validate(&self) -> Result<(), Error> {
        if self.steps.len() > 1 {
            return Err(Error::new(
                "d4.material",
                "native material resources admit at most one resolved shader",
            ));
        }
        self.steps.iter().try_for_each(SurfacePayload::validate)
    }
}
