//! Typed native renderer declarations, independent of execution or GPU policy.
use super::{Document, ReadError};

/// Native renderer with resolved stream and property declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Renderer<'a> {
    /// Native renderer class; sparse omission denotes zero.
    pub class: i32,
    /// Ordered inputs whose storage indices refer to the owning layer's fields.
    pub inputs: Vec<RendererInput<'a>>,
    /// Authored properties only; absent toggles are not synthesized.
    pub properties: Vec<RendererProperty<'a>>,
    /// Native feature declaration path, rather than a mesh dependency.
    pub feature_set: &'a [u8],
    /// Signed draw priority; sparse omission denotes zero.
    pub draw_order: i32,
}
/// One native renderer input; custom names retain their native type metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererInput<'a> {
    /// Lossless semantic selector; sparse omission denotes zero.
    pub semantic: u32,
    /// Index into the owning compiled layer's field array.
    pub storage_index: u32,
    /// Optional custom field name bytes.
    pub additional_name: Option<&'a [u8]>,
    /// Optional native custom field type identifier.
    pub additional_type: Option<i32>,
}
/// A renderer property preserving numeric bits independently of its type selector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererProperty<'a> {
    /// Native property name bytes.
    pub name: &'a [u8],
    /// Lossless native property type; sparse omission denotes zero.
    pub property_type: i32,
    /// Raw component words; sparse omission denotes all-zero words.
    pub numeric: [u32; 4],
    /// Optional authored string value, independently of numeric storage.
    pub text: Option<&'a [u8]>,
}
impl<'a> Document<'a> {
    /// Resolve one renderer and its typed input/property links.
    pub fn renderer(&self, link: u32) -> Result<Renderer<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheRenderer")?;
        let (mut inputs, mut properties, mut feature_set) = (None, None, None);
        let (mut class, mut draw_order) = (0, 0);
        self.fields(object, |field, input| {
            match field {
                0 => class = input.word()? as i32,
                1 => inputs = Some(self.links(input)?),
                2 => properties = Some(self.links(input)?),
                3 => feature_set = Some(self.string(input)?),
                4 => draw_order = input.word()? as i32,
                _ => return Err(input.error("unsupported renderer field")),
            }
            Ok(())
        })?;
        Ok(Renderer {
            class,
            inputs: self
                .required(object, inputs)?
                .into_iter()
                .map(|link| self.renderer_input(link))
                .collect::<Result<_, _>>()?,
            properties: self
                .required(object, properties)?
                .into_iter()
                .map(|link| self.renderer_property(link))
                .collect::<Result<_, _>>()?,
            feature_set: self.required(object, feature_set)?,
            draw_order,
        })
    }
    /// Read a stream binding without interpreting custom semantics as fixed enums.
    pub fn renderer_input(&self, link: u32) -> Result<RendererInput<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheRendererParticleInput")?;
        let mut semantic = 0;
        let (mut storage_index, mut additional_name, mut additional_type) = (None, None, None);
        self.fields(object, |field, input| {
            match field {
                0 => semantic = input.word()?,
                1 => storage_index = Some(input.word()?),
                2 => additional_name = Some(self.string(input)?),
                3 => additional_type = Some(input.word()? as i32),
                _ => return Err(input.error("unsupported renderer-input field")),
            }
            Ok(())
        })?;
        Ok(RendererInput {
            semantic,
            storage_index: self.required(object, storage_index)?,
            additional_name,
            additional_type,
        })
    }
    /// Read numeric words and text without guessing the property's application meaning.
    pub fn renderer_property(&self, link: u32) -> Result<RendererProperty<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheRendererProperty")?;
        let mut name = None;
        let (mut property_type, mut numeric, mut text) = (0, [0; 4], None);
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => property_type = input.word()? as i32,
                2 => numeric = self.words(input)?,
                3 => text = Some(self.string(input)?),
                _ => return Err(input.error("unsupported renderer-property field")),
            }
            Ok(())
        })?;
        Ok(RendererProperty {
            name: self.required(object, name)?,
            property_type,
            numeric,
            text,
        })
    }
}
