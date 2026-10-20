//! Borrowed constant storage and typed compiled layer declarations.
use super::{Cursor, Document, NativeType, ReadError};
use crate::model::common::{PackedArray, PackedValue};

/// Native compiled-layer storage and links to immutable programmes/resources.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerCache<'a> {
    /// Raw component bits; no float conversion is applied.
    pub constants: PackedArray<'a, [u32; 4]>,
    /// Raw native range records, interpreted by field declarations.
    pub ranges: PackedArray<'a, [u32; 4]>,
    /// Canonical named fields in native storage-index order.
    pub fields: Vec<LayerField<'a>>,
    /// One-based authored attribute links.
    pub attributes: Vec<u32>,
    /// Event declarations in native order.
    pub events: Vec<LayerEvent<'a>>,
    /// One-based spatial-layer declaration links.
    pub spatial_layers: Vec<u32>,
    /// One-based named sampler declaration links.
    pub samplers: Vec<u32>,
    /// One-based renderer declaration links.
    pub renderers: Vec<u32>,
    /// Optional native varying-programme link.
    pub varying: Option<u32>,
    /// Native initialization and alternative fixed/physics programme links.
    pub fixed: Vec<u32>,
    /// Payload declaration used when the layer is born.
    pub root_event: LayerEvent<'a>,
    /// Native usage controls, with sparse zero default.
    pub usage: u32,
    /// Evolution can cause external effects.
    pub evolution_side_effects: bool,
    /// Native death-step evolution request.
    pub evolve_on_death: bool,
    /// Native deterministic seed modifier, with sparse zero default.
    pub seed_modifier: u32,
    /// Authored LOD endpoints; absence remains distinct from explicit minus one.
    pub lod: [Option<f32>; 2],
}
/// Named native storage field, independently of temporary register addresses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerField<'a> {
    /// Canonical field name bytes.
    pub name: &'a [u8],
    /// Lossless native type; metadata fields may have unsupported layouts.
    pub native_type: NativeType,
    /// Native byte footprint.
    pub storage_bytes: u32,
    /// Native allocation and constant controls.
    pub flags: u32,
    /// Native constant index; omission means zero, minus one denotes absence.
    pub constant_slot: i32,
    /// Native range index; omission means zero, minus one denotes absence.
    pub range_slot: i32,
}
/// Typed payload declaration for one layer event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerEvent<'a> {
    /// Native channel name; omission means an empty name.
    pub name: &'a [u8],
    /// Native event controls, with sparse zero default.
    pub flags: u32,
    /// Ordered payload field declarations.
    pub payload: Vec<PayloadField<'a>>,
    /// Declared payload footprint, with sparse zero default.
    pub payload_bytes: u32,
}
/// One declared payload element, preserving independent type and footprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadField<'a> {
    /// Canonical payload name bytes.
    pub name: &'a [u8],
    /// Lossless native declaration type.
    pub native_type: NativeType,
    /// Native element footprint in bytes.
    pub footprint_bytes: u32,
    /// Native controls, with sparse zero default.
    pub flags: u32,
    /// Native element category, with sparse zero default.
    pub kind: u32,
    /// Native range index; sparse zero and explicit MAX remain distinct.
    pub range_slot: u32,
}
impl<'a> Document<'a> {
    pub(super) fn packed<T: PackedValue>(
        &self,
        input: &mut Cursor<'a>,
        width: usize,
    ) -> Result<PackedArray<'a, T>, ReadError> {
        let count = input.count(width)?;
        let bytes = input.take(count * width)?;
        PackedArray::parse_bytes(bytes).map_err(|_| input.error("invalid packed declaration array"))
    }
    fn boolean(&self, input: &mut Cursor<'_>) -> Result<bool, ReadError> {
        match input.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(input.error("native boolean is neither zero nor one")),
        }
    }
    /// Read a compiled layer with borrowed constant/range arrays and checked links.
    /// Unsupported native field layouts remain inspectable, without becoming VM values.
    pub fn layer_cache(&self, link: u32) -> Result<LayerCache<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCache")?;
        let mut constants = PackedArray::new();
        let mut ranges = None;
        let mut fields = None;
        let mut attributes = Vec::new();
        let mut events = Vec::new();
        let mut spatial_layers = Vec::new();
        let mut samplers = Vec::new();
        let mut renderers = Vec::new();
        let mut varying = None;
        let mut fixed = None;
        let mut root_event = None;
        let mut usage = 0;
        let mut evolution_side_effects = false;
        let mut evolve_on_death = false;
        let mut seed_modifier = 0;
        let mut lod = [None; 2];
        self.fields(object, |field, input| {
            match field {
                0 => constants = self.packed(input, 16)?,
                1 => ranges = Some(self.packed(input, 16)?),
                2 => fields = Some(self.links(input)?),
                3 => attributes = self.links(input)?,
                5 => events = self.links(input)?,
                6 => spatial_layers = self.links(input)?,
                7 => samplers = self.links(input)?,
                8 => renderers = self.links(input)?,
                9 => varying = Some(self.required_link(input)?),
                10 => fixed = Some(self.links(input)?),
                12 => root_event = Some(self.required_link(input)?),
                13 => usage = input.word()?,
                14 => evolution_side_effects = self.boolean(input)?,
                15 => evolve_on_death = self.boolean(input)?,
                20 => seed_modifier = input.word()?,
                22 | 23 => lod[usize::from(field - 22)] = Some(f32::from_bits(input.word()?)),
                _ => return Err(input.error("unsupported compiled-layer field")),
            }
            Ok(())
        })?;
        let fields = self
            .required(object, fields)?
            .into_iter()
            .map(|link| self.layer_field(link))
            .collect::<Result<Vec<_>, _>>()?;
        let events = events
            .into_iter()
            .map(|link| self.layer_event(link))
            .collect::<Result<Vec<_>, _>>()?;
        let fixed = self.required(object, fixed)?;
        for link in fixed.iter().copied().chain(varying) {
            self.declaration(link, b"CCompilerBlobCache")?;
        }
        let root_event = self.layer_event(self.required(object, root_event)?)?;
        Ok(LayerCache {
            constants,
            ranges: self.required(object, ranges)?,
            fields,
            attributes,
            events,
            spatial_layers,
            samplers,
            renderers,
            varying,
            fixed,
            root_event,
            usage,
            evolution_side_effects,
            evolve_on_death,
            seed_modifier,
            lod,
        })
    }
    /// Read field storage controls, preserving unknown native type identifiers.
    pub fn layer_field(&self, link: u32) -> Result<LayerField<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheField")?;
        let (mut name, mut native_type, mut storage_bytes, mut flags) = (None, None, None, None);
        let mut constant_slot = 0;
        let mut range_slot = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => native_type = Some(NativeType(input.word()?)),
                2 => storage_bytes = Some(input.word()?),
                3 => flags = Some(input.word()?),
                4 => constant_slot = input.word()? as i32,
                5 => range_slot = input.word()? as i32,
                _ => return Err(input.error("unsupported compiled-field declaration")),
            }
            Ok(())
        })?;
        if constant_slot < -1 || range_slot < -1 {
            return Err(ReadError {
                offset: self.object_origin(object),
                message: "unsupported negative field-storage sentinel",
            });
        }
        Ok(LayerField {
            name: self.required(object, name)?,
            native_type: self.required(object, native_type)?,
            storage_bytes: self.required(object, storage_bytes)?,
            flags: self.required(object, flags)?,
            constant_slot,
            range_slot,
        })
    }
    /// Resolve a layer event and its payload declaration links.
    pub fn layer_event(&self, link: u32) -> Result<LayerEvent<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheEvent")?;
        let mut name = &b""[..];
        let mut flags = 0;
        let mut payload = Vec::new();
        let mut payload_bytes = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = self.string(input)?,
                1 => flags = input.word()?,
                2 => payload = self.links(input)?,
                3 => payload_bytes = input.word()?,
                _ => return Err(input.error("unsupported layer-event field")),
            }
            Ok(())
        })?;
        let payload = payload
            .into_iter()
            .map(|link| self.payload_field(link))
            .collect::<Result<Vec<_>, _>>()?;
        let bytes = payload.iter().try_fold(0u32, |bytes, field| {
            bytes.checked_add(field.footprint_bytes)
        });
        if bytes != Some(payload_bytes) {
            return Err(ReadError {
                offset: self.object_origin(object),
                message: "event payload footprints do not match the declared byte size",
            });
        }
        Ok(LayerEvent {
            name,
            flags,
            payload,
            payload_bytes,
        })
    }
    /// Read an event payload element without inferring its category semantics.
    pub fn payload_field(&self, link: u32) -> Result<PayloadField<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheEventPayload")?;
        let (mut name, mut native_type, mut footprint_bytes) = (None, None, None);
        let mut flags = 0;
        let mut kind = 0;
        let mut range_slot = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => native_type = Some(NativeType(input.word()?)),
                2 => footprint_bytes = Some(input.word()?),
                3 => flags = input.word()?,
                4 => kind = input.word()?,
                5 => range_slot = input.word()?,
                _ => return Err(input.error("unsupported event-payload field")),
            }
            Ok(())
        })?;
        Ok(PayloadField {
            name: self.required(object, name)?,
            native_type: self.required(object, native_type)?,
            footprint_bytes: self.required(object, footprint_bytes)?,
            flags,
            kind,
            range_slot,
        })
    }
}
