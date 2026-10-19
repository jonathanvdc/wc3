//! Resource-driven assembly independent of native game appearance vocabularies.
mod atlas;
mod compile;
mod images;
mod plan;

use crate::TextureInput;
use crate::{DrawSelection, ModelError, PartBinding};
pub use atlas::{compose_atlas, Atlas, AtlasBlend, AtlasError, AtlasLayer, PixelRect, RgbaImage};
pub use compile::compile_assembly;
pub(crate) use compile::compile_with_bindings;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use wc3::model::{ClipId, ImageResource, ImageResourceId, Model, ResourceLocator};

/// Models resolved from a coherent application resource provider.
pub type Models<'data> = HashMap<ResourceLocator<'static>, Model<'data>>;
/// One source graph and its exact typed appearance choices.
#[derive(Clone, Debug)]
pub struct Source {
    /// Resource locating the source model.
    pub model: ResourceLocator<'static>,
    /// Geometry selection before compilation.
    pub draws: DrawSelection,
    /// Replace an image dependency wherever it is sampled.
    pub images: BTreeMap<ImageResourceId, TextureReplacement>,
    /// Replace one input's image source, overriding a resource-wide replacement.
    /// Sampling, color interpretation and coordinates remain authored by the input.
    pub inputs: BTreeMap<TextureInput, TextureReplacement>,
}
impl Source {
    /// Select all authored geometry without texture replacements.
    pub fn new(model: ResourceLocator<'static>) -> Self {
        Self {
            model,
            draws: DrawSelection::default(),
            images: BTreeMap::new(),
            inputs: BTreeMap::new(),
        }
    }
}
/// A component source and its relationship to the root graph.
#[derive(Clone, Debug)]
pub struct Part {
    /// Source graph and appearance.
    pub source: Source,
    /// Explicit attachment or skeleton relationship.
    pub binding: PartBinding,
    /// Root-to-source animation clip correspondence.
    pub clips: BTreeMap<ClipId, ClipId>,
}
impl Part {
    /// Add a component without synchronized clip mappings.
    pub fn new(source: Source, binding: PartBinding) -> Self {
        Self {
            source,
            binding,
            clips: BTreeMap::new(),
        }
    }
}
/// Fully resolved resource assembly, ready for checked compilation.
#[derive(Clone, Debug)]
pub struct Assembly {
    /// Carrying graph.
    pub root: Source,
    /// Ordered components.
    pub parts: Vec<Part>,
}
impl Assembly {
    /// Start an assembly with a root and no components.
    pub fn new(root: Source) -> Self {
        Self {
            root,
            parts: Vec::new(),
        }
    }
}

// Native adapters defer correspondence until selected geometry has been
// compiled. Keep this intermediate representation inside the crate so public
// assembly plans always have consumable, fully resolved bindings.
#[derive(Clone, Debug)]
pub(crate) struct PendingPart<B> {
    pub(crate) source: Source,
    pub(crate) binding: B,
    pub(crate) clips: BTreeMap<ClipId, ClipId>,
}
impl<B> PendingPart<B> {
    pub(crate) fn new(source: Source, binding: B) -> Self {
        Self {
            source,
            binding,
            clips: BTreeMap::new(),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct PendingAssembly<B> {
    pub(crate) root: Source,
    pub(crate) parts: Vec<PendingPart<B>>,
}
impl<B> PendingAssembly<B> {
    pub(crate) fn new(root: Source) -> Self {
        Self {
            root,
            parts: Vec::new(),
        }
    }
}
/// Application-defined selection vocabulary resolved into generic model assembly.
pub trait Resolver<Selection> {
    /// Application resolver failure.
    type Error: Error;
    /// Resolve one selection using the available semantic models.
    fn resolve(&self, selection: &Selection, models: &Models<'_>) -> Result<Assembly, Self::Error>;
}
/// One ordered contribution to a composed image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureLayer {
    /// Source image resource.
    pub resource: ImageResource<'static>,
    /// Source rectangle; absent means the full image.
    pub source: Option<PixelRect>,
    /// Destination atlas rectangle.
    pub destination: PixelRect,
    /// Opacity multiplier, with 255 retaining source alpha.
    pub opacity: u8,
    /// Encoded-color compositing equation.
    pub blend: AtlasBlend,
}
/// Direct image replacement or ordered atlas composition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextureReplacement {
    /// A decoded image from the provider.
    Image(ImageResource<'static>),
    /// Explicit atlas geometry and layers.
    Atlas {
        /// Output dimensions and background.
        atlas: Atlas,
        /// Back-to-front layers.
        layers: Vec<TextureLayer>,
    },
}
impl TextureReplacement {
    /// Validate dimensions and declared rectangles before resolving images.
    pub fn validate(&self) -> Result<(), ModelError> {
        if let Self::Atlas { atlas, layers } = self {
            atlas::pixel_len(atlas.width, atlas.height).map_err(|e| ModelError(e.to_string()))?;
            for layer in layers {
                if !atlas::valid(layer.destination, atlas.width, atlas.height)
                    || layer.source.is_some_and(|r| {
                        r.width == 0
                            || r.height == 0
                            || r.x.checked_add(r.width).is_none()
                            || r.y.checked_add(r.height).is_none()
                    })
                {
                    return Err(ModelError(AtlasError::Rectangle.to_string()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
