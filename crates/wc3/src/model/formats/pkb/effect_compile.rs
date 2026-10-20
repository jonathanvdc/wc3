//! Native baked graph lowering into owned, renderer-independent effect programmes.
use super::effect_payload::{payload_layout, payload_policies, payload_reads};
use super::{
    CompileLimits, CompilerBlob, Document, Effect, Instruction as NativeInstruction, LayerCache,
    LayerIndex, NativeType, ReadError,
};
use crate::model::common::PackedArray;
use crate::model::scene::popcorn::graph::{
    BoundProgram, Callback, EffectGraph, EffectSampler, EventRoute, External,
};
use crate::model::scene::popcorn::layout::compact_registers;
use crate::model::scene::popcorn::payload::Layout as PayloadLayout;
use crate::model::scene::popcorn::shapes::{Domain, Primitive, Shape};
use crate::model::scene::popcorn::{
    EffectAttribute, EffectEntry, EffectField, EffectFieldId, EffectLayer, EffectLayerId,
    EffectLimitation, EffectProgram, EffectRandom, EffectRenderer, EffectRendererInput,
    EffectRendererProperty, EventStreamDuration, Function as Signature, Value, ValueType,
};
use glam::{Quat, Vec3};
use std::collections::HashMap;
use std::error::Error;
use std::f32::consts::FRAC_PI_2;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::iter::once;
use std::sync::Arc;

/// One selected evolution programme per simulation step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EffectEvolution {
    /// Prefer authored physics, otherwise fixed, otherwise varying evolution.
    #[default]
    Physics,
    /// Prefer authored fixed evolution, otherwise varying evolution.
    Fixed,
    /// Require authored varying evolution when an evolution programme exists.
    Varying,
}
/// Checked native graph-lowering policies, with explicit unresolved duration behavior.
#[derive(Clone, Copy, Debug)]
pub struct EffectCompileOptions {
    /// Scalar instruction/storage limits applied to every selected programme.
    pub scalar_limits: CompileLimits,
    /// Maximum native layer count accepted by this compiler.
    pub layers: usize,
    /// Alternative evolution selection policy.
    pub evolution: EffectEvolution,
    /// Explicit duration policy for authored event streams.
    pub stream_duration: EventStreamDuration,
    /// Original deterministic random policy.
    pub random: EffectRandom,
}
impl Default for EffectCompileOptions {
    fn default() -> Self {
        Self {
            scalar_limits: CompileLimits::default(),
            layers: 128,
            evolution: EffectEvolution::Physics,
            stream_duration: EventStreamDuration::LastOccurrence,
            random: EffectRandom::Counter,
        }
    }
}
/// Native graph or unsupported callback with its owning native layer and instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectCompileError {
    /// Graph layer index when the failure belongs to one layer.
    pub layer: Option<EffectLayerId>,
    /// Native stage identifier when lowering a programme.
    pub stage: Option<u32>,
    /// Native instruction byte location, when scalar compilation failed.
    pub offset: Option<usize>,
    /// Missing contract or invalid relationship.
    pub message: String,
}
impl Display for EffectCompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "effect layer {:?}, stage {:?}, byte {:?}: {}",
            self.layer, self.stage, self.offset, self.message
        )
    }
}
impl Error for EffectCompileError {}
impl From<ReadError> for EffectCompileError {
    fn from(error: ReadError) -> Self {
        Self {
            layer: None,
            stage: None,
            offset: Some(error.offset),
            message: error.message.into(),
        }
    }
}
pub(super) fn error(message: impl Into<String>) -> EffectCompileError {
    EffectCompileError {
        layer: None,
        stage: None,
        offset: None,
        message: message.into(),
    }
}
fn attribute_default(
    kind: ValueType,
    float: Option<[u32; 4]>,
    integer: Option<[u32; 4]>,
) -> Result<Value, &'static str> {
    match kind {
        ValueType::Float | ValueType::Float2 | ValueType::Float3 | ValueType::Float4 => {
            Ok(Value::from_words(kind, float.unwrap_or([0; 4])))
        }
        ValueType::Bool => {
            if integer.is_none() && float.is_some_and(|words| words.iter().any(|word| *word != 0)) {
                return Err("boolean attribute default requires integer-family storage");
            }
            match integer.unwrap_or([0; 4])[0] {
                0 => Ok(Value::from(false)),
                1 | u32::MAX => Ok(Value::from(true)),
                _ => Err("boolean attribute default is not canonical zero, one or all-ones"),
            }
        }
        _ => Err("unsupported native attribute default layout"),
    }
}
struct Lowering<'d, 'a> {
    document: &'d Document<'a>,
    options: &'d EffectCompileOptions,
    samplers: Vec<EffectSampler>,
    attributes: Vec<EffectAttribute>,
    attribute_slots: HashMap<Vec<u8>, usize>,
    curve_widths: HashMap<u32, u32>,
}
fn infer_curve_widths(
    document: &Document<'_>,
    caches: &[LayerCache<'_>],
) -> Result<HashMap<u32, u32>, EffectCompileError> {
    let mut widths = HashMap::new();
    for (layer, cache) in caches.iter().enumerate() {
        let curves = cache
            .samplers
            .iter()
            .map(|link| document.named_sampler(*link))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|named| named.sampler_type == 3)
            .map(|named| (named.name, named.sampler))
            .collect::<HashMap<_, _>>();
        if curves.is_empty() {
            continue;
        }
        for link in cache.fixed.iter().copied().chain(cache.varying) {
            let blob = document.compiler_blob_at(link)?;
            let script = blob.script()?;
            let functions = blob
                .functions
                .iter()
                .map(|link| document.function_declaration(*link))
                .collect::<Result<Vec<_>, _>>()?;
            let externals = blob
                .externals
                .iter()
                .map(|link| document.external_declaration(*link))
                .collect::<Result<Vec<_>, _>>()?;
            for located in &script.instructions {
                let NativeInstruction::Call {
                    function, result, ..
                } = &located.instruction
                else {
                    continue;
                };
                let Some(function) = functions.get(usize::from(*function)) else {
                    continue;
                };
                if function.name != b"sample" || function.symbol_slot == u32::MAX {
                    continue;
                }
                let Some(external) = externals.get(function.symbol_slot as usize) else {
                    continue;
                };
                let Some(link) = curves.get(external.name) else {
                    continue;
                };
                let width = match result.bank() {
                    0x20 => 1,
                    0x21 => 2,
                    0x22 => 3,
                    0x23 => 4,
                    _ => {
                        return Err(EffectCompileError {
                            layer: Some(EffectLayerId(layer as u32)),
                            stage: Some(blob.identifier),
                            offset: Some(located.offset),
                            message:
                                "curve sample return must have floating-point component layout"
                                    .into(),
                        })
                    }
                };
                if widths
                    .insert(*link, width)
                    .is_some_and(|previous| previous != width)
                {
                    return Err(EffectCompileError {
                        layer: Some(EffectLayerId(layer as u32)),
                        stage: Some(blob.identifier),
                        offset: Some(located.offset),
                        message: "shared curve sample uses disagree on component layout".into(),
                    });
                }
            }
        }
    }
    Ok(widths)
}
impl<'a> Document<'a> {
    /// Discover effect roots together with their checked one-based object links.
    pub fn effect_roots(&self) -> impl Iterator<Item = Result<(u32, Effect), ReadError>> + '_ {
        self.objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                self.strings[self.types[object.type_index as usize].name as usize]
                    == b"CParticleEffect"
            })
            .map(|(index, _)| {
                let link = (index + 1) as u32;
                self.effect(link).map(|effect| (link, effect))
            })
    }
    /// Compile a complete supported graph, including entry routes and standalone controllers.
    /// Unsupported callbacks return explicit errors rather than no-op host bindings.
    pub fn compile_effect(
        &self,
        link: u32,
        options: &EffectCompileOptions,
    ) -> Result<EffectProgram, EffectCompileError> {
        let effect = self.effect(link)?;
        let graph = self.layer_graph(effect.graph)?;
        if graph.layers.len() > options.layers {
            return Err(error("effect graph exceeds configured layer limit"));
        }
        if !graph.broadcasts.is_empty() {
            return Err(error("native broadcast execution is unsupported"));
        }
        if let EventStreamDuration::Explicit(duration) = options.stream_duration {
            if !duration.is_finite() || duration < 0. {
                return Err(error(
                    "explicit stream duration must be finite and nonnegative",
                ));
            }
        }
        let caches = graph
            .layers
            .iter()
            .map(|layer| self.layer_cache(layer.cache))
            .collect::<Result<Vec<_>, _>>()?;
        let mut lowering = Lowering {
            document: self,
            options,
            samplers: Vec::new(),
            attributes: Vec::new(),
            attribute_slots: HashMap::new(),
            curve_widths: infer_curve_widths(self, &caches)?,
        };
        for cache in &caches {
            for link in &cache.attributes {
                lowering.attribute(*link)?;
            }
        }
        let payloads = caches
            .iter()
            .map(|cache| payload_layout(&cache.root_event))
            .collect::<Result<Vec<_>, _>>()?;
        let mut layers = Vec::new();
        let mut renderers = Vec::new();
        for (index, cache) in caches.iter().enumerate() {
            let layer = EffectLayerId(index as u32);
            let result = (|| {
                if !cache.spatial_layers.is_empty() {
                    return Err(error(
                        "native spatial payload/query execution is unsupported",
                    ));
                }
                let fields = cache
                    .fields
                    .iter()
                    .map(|field| {
                        let initial = if field.flags & 4 != 0 && field.constant_slot >= 0 {
                            cache
                                .constants
                                .get(field.constant_slot as usize)
                                .ok_or_else(|| {
                                    error("field constant index exceeds its owning layer")
                                })?
                        } else {
                            [0; 4]
                        };
                        Ok(EffectField {
                            name: field.name.into(),
                            native_type: field.native_type.0,
                            value_type: field.native_type.value_type(),
                            initial,
                        })
                    })
                    .collect::<Result<Vec<_>, EffectCompileError>>()?;
                let mut field_slots = HashMap::new();
                for (index, field) in fields.iter().enumerate() {
                    if field_slots.insert(field.name.to_vec(), index).is_some() {
                        return Err(error("duplicate canonical field name"));
                    }
                }
                let lifecycle = |name: &[u8]| -> Result<usize, EffectCompileError> {
                    let slot = *field_slots.get(name).ok_or_else(|| {
                        error(format!(
                            "missing lifecycle field {}",
                            String::from_utf8_lossy(name)
                        ))
                    })?;
                    if fields[slot].value_type != Some(ValueType::Float) {
                        return Err(error(
                            "lifecycle fields must have scalar floating-point layout",
                        ));
                    }
                    Ok(slot)
                };
                let life = lifecycle(b"self.lifeRatio")?;
                let inverse_life = lifecycle(b"self.invLife")?;
                let mut resources = HashMap::<Vec<u8>, External>::new();
                for link in &cache.samplers {
                    let sampler = self.named_sampler(*link)?;
                    let index = lowering.sampler(sampler.sampler, sampler.sampler_type)?;
                    if resources
                        .insert(sampler.name.to_vec(), External::Sampler(index))
                        .is_some()
                    {
                        return Err(error("duplicate sampler binding name"));
                    }
                }
                for declaration in &cache.events {
                    let index = graph
                        .events
                        .iter()
                        .enumerate()
                        .find(|(_, event)| {
                            event.parent == Some(LayerIndex(layer.0))
                                && event.name == declaration.name
                        })
                        .map(|(index, _)| index)
                        .ok_or_else(|| {
                            error("layer event declaration has no matching owned graph route")
                        })?;
                    if resources
                        .insert(declaration.name.to_vec(), External::Event(index))
                        .is_some()
                    {
                        return Err(error("duplicate event or sampler binding name"));
                    }
                }
                let mut init = None;
                let (mut physics, mut fixed, mut varying) = (None, None, None);
                for link in cache.fixed.iter().copied().chain(cache.varying) {
                    let blob = self.compiler_blob_at(link)?;
                    let target = match blob.identifier {
                        0 => &mut init,
                        3 => &mut physics,
                        4 => &mut fixed,
                        5 => &mut varying,
                        _ => return Err(error("unsupported native particle programme stage")),
                    };
                    if target.replace(blob).is_some() {
                        return Err(error("duplicate native particle programme stage"));
                    }
                }
                let init =
                    init.ok_or_else(|| error("effect layer has no initialization programme"))?;
                let evolve = match options.evolution {
                    EffectEvolution::Physics => physics.or(fixed).or(varying),
                    EffectEvolution::Fixed => fixed.or(varying),
                    EffectEvolution::Varying => varying,
                };
                let init = lowering.bind(&init, &fields, &field_slots, &resources)?;
                let evolve = evolve
                    .as_ref()
                    .map(|blob| lowering.bind(blob, &fields, &field_slots, &resources))
                    .transpose()?;
                let spawner = init
                    .callbacks
                    .iter()
                    .chain(evolve.iter().flat_map(|bound| bound.callbacks.iter()))
                    .any(|callback| matches!(callback, Callback::Generate | Callback::Trigger));
                for link in &cache.renderers {
                    let renderer = self.renderer(*link)?;
                    let inputs = renderer
                        .inputs
                        .iter()
                        .map(|input| {
                            if input.storage_index as usize >= fields.len() {
                                return Err(error(
                                    "renderer stream index exceeds canonical layer fields",
                                ));
                            }
                            if fields[input.storage_index as usize].value_type.is_none() {
                                return Err(error(
                                    "renderer cannot consume an opaque metadata field",
                                ));
                            }
                            Ok(EffectRendererInput {
                                semantic: input.semantic,
                                field: EffectFieldId(input.storage_index),
                                additional_name: input.additional_name.map(Into::into),
                                additional_type: input.additional_type,
                            })
                        })
                        .collect::<Result<Vec<_>, EffectCompileError>>()?;
                    renderers.push(EffectRenderer {
                        layer,
                        class: renderer.class,
                        feature_set: renderer.feature_set.into(),
                        draw_order: renderer.draw_order,
                        inputs: inputs.into_boxed_slice(),
                        properties: renderer
                            .properties
                            .iter()
                            .map(|property| EffectRendererProperty {
                                name: property.name.into(),
                                property_type: property.property_type,
                                numeric: property.numeric,
                                text: property.text.map(Into::into),
                            })
                            .collect(),
                    });
                }
                let mut compiled = EffectLayer {
                    fields: fields.into_boxed_slice(),
                    init,
                    evolve,
                    life,
                    inverse_life,
                    standalone: graph.layers[index].parents.is_empty(),
                    spawner: spawner && cache.renderers.is_empty(),
                    rendered: !cache.renderers.is_empty(),
                    evolve_on_death: cache.evolve_on_death,
                    payload: payloads[index].clone(),
                };
                compact_registers(
                    once(&mut compiled.init.program)
                        .chain(compiled.evolve.iter_mut().map(|bound| &mut bound.program)),
                );
                Ok(compiled)
            })();
            layers.push(result.map_err(|mut error| {
                error.layer = Some(layer);
                error
            })?);
        }
        let mut events = Vec::new();
        for event in &graph.events {
            let payload = if let Some(parent) = event.parent {
                let declaration = caches[parent.0 as usize]
                    .events
                    .iter()
                    .find(|declaration| declaration.name == event.name)
                    .ok_or_else(|| {
                        error("owned graph event has no compiled payload declaration")
                    })?;
                payload_layout(declaration)?
            } else {
                PayloadLayout::default()
            };
            let mut maps = Vec::new();
            for target in &event.targets {
                let layer = &layers[target.0 as usize];
                let mapping = payload.map_to(&layer.payload).map_err(error)?;
                let mut reads = payload_reads(&layer.init, &layer.payload)?;
                if let Some(evolve) = &layer.evolve {
                    reads.extend(payload_reads(evolve, &layer.payload)?);
                }
                layer
                    .payload
                    .validate_reads(&mapping, &reads)
                    .map_err(error)?;
                maps.push(mapping);
            }
            events.push(EventRoute {
                targets: event
                    .targets
                    .iter()
                    .map(|target| EffectLayerId(target.0))
                    .collect(),
                payload,
                maps: maps.into_boxed_slice(),
            });
        }
        let layers_use_inverse = layers.iter().any(|layer| {
            Some(&layer.init)
                .into_iter()
                .chain(layer.evolve.as_ref())
                .any(|bound| {
                    bound.callbacks.iter().any(|callback| {
                        matches!(
                            callback,
                            Callback::InverseTransform | Callback::InverseDirection
                        )
                    })
                })
        });
        let payload_policies = payload_policies(&layers, &events);
        Ok(EffectProgram {
            inner: Arc::new(EffectGraph {
                identity: Arc::new(()),
                default_entry: graph.default_entry,
                entries: graph
                    .entries
                    .into_iter()
                    .map(|entry| EffectEntry {
                        name: entry.name.into(),
                        event: entry.event.0 as usize,
                    })
                    .collect(),
                layers: layers.into_boxed_slice(),
                events: events.into_boxed_slice(),
                renderers: renderers.into_boxed_slice(),
                attributes: lowering.attributes.into_boxed_slice(),
                limitations: {
                    let mut limitations = vec![
                        EffectLimitation::RandomSequence,
                        EffectLimitation::BirthScheduling,
                        EffectLimitation::StreamDuration,
                        EffectLimitation::CoordinateMetadata,
                        EffectLimitation::TransformMask,
                        EffectLimitation::FieldInitialization,
                        EffectLimitation::DeathEvolution,
                        EffectLimitation::InstantLifetime,
                        EffectLimitation::CurveInterpolation,
                        EffectLimitation::OrientationPolicies,
                    ];
                    if layers_use_inverse {
                        limitations.push(EffectLimitation::InverseTransform);
                    }
                    limitations.extend(payload_policies);
                    if lowering
                        .samplers
                        .iter()
                        .any(|sampler| matches!(sampler, EffectSampler::Shape(_)))
                    {
                        limitations.extend([
                            EffectLimitation::ShapeDefaults,
                            EffectLimitation::ShapeCoordinates,
                            EffectLimitation::ShapeSampling,
                        ]);
                    }
                    limitations.into_boxed_slice()
                },
                duration: options.stream_duration,
                samplers: lowering.samplers.into_boxed_slice(),
                random: options.random,
            }),
        })
    }
}
impl Lowering<'_, '_> {
    fn attribute(&mut self, link: u32) -> Result<(), EffectCompileError> {
        let d = self.document;
        let object = d.declaration(link, b"CLayerCompileCacheAttrib")?;
        let (mut name, mut kind) = (None, None);
        let (mut float, mut integer) = (None, None);
        d.fields(object, |field, input| {
            match field {
                0 => name = Some(d.string(input)?),
                2 => kind = Some(NativeType(input.word()?)),
                3 | 4 => {
                    input.word()?;
                }
                5 | 6 | 8 | 9 => {
                    d.words(input)?;
                }
                7 => integer = Some(d.words(input)?),
                10 => float = Some(d.words(input)?),
                _ => return Err(input.error("unsupported compiled-attribute field")),
            }
            Ok(())
        })?;
        let name = d.required(object, name)?;
        let kind = d
            .required(object, kind)?
            .value_type()
            .ok_or_else(|| error("unsupported attribute value layout"))?;
        let default = attribute_default(kind, float, integer).map_err(error)?;
        if let Some(index) = self.attribute_slots.get(name) {
            if self.attributes[*index].default != default {
                return Err(error("same named attribute has conflicting layer defaults"));
            }
        } else {
            let index = self.attributes.len();
            self.attribute_slots.insert(name.to_vec(), index);
            self.attributes.push(EffectAttribute {
                name: name.strip_prefix(b"__a_").unwrap_or(name).into(),
                default,
            });
        }
        Ok(())
    }
    fn sampler(&mut self, link: u32, kind: i32) -> Result<usize, EffectCompileError> {
        let sampler = match kind {
            3 => {
                let curve = self.document.curve_sampler(link)?;
                if curve.interpolator.is_some_and(|value| value != 0) {
                    return Err(error("unsupported native curve interpolation selector"));
                }
                let inferred = self.curve_widths.get(&link).copied();
                if matches!((curve.value_type, inferred), (Some(native), Some(used)) if native != used)
                {
                    return Err(error(
                        "curve dimensionality differs from its checked sample return layout",
                    ));
                }
                let width = curve.value_type.or(inferred).ok_or_else(|| {
                    error("sparse native curve dimensionality has no typed sample use")
                })? as usize;
                let kind = match width {
                    1 => ValueType::Float,
                    2 => ValueType::Float2,
                    3 => ValueType::Float3,
                    4 => ValueType::Float4,
                    _ => return Err(error("unsupported native curve dimension")),
                };
                let times: Vec<_> = curve.times.iter().collect();
                if times.is_empty()
                    || times.iter().any(|time| !time.is_finite())
                    || times.windows(2).any(|window| window[0] >= window[1])
                    || curve.values.len() != times.len() * width
                    || (!curve.tangents.is_empty()
                        && curve.tangents.len() != times.len() * width * 2)
                {
                    return Err(error(
                        "curve keys/components/tangents have incompatible lengths or times",
                    ));
                }
                let pack =
                    |source: &PackedArray<'_, f32>| -> Result<Box<[[f32; 4]]>, EffectCompileError> {
                        let mut result = Vec::with_capacity(source.len() / width);
                        for index in 0..source.len() / width {
                            let mut words = [0.; 4];
                            for (component, word) in words.iter_mut().enumerate().take(width) {
                                let value = source.get(index * width + component).unwrap();
                                if !value.is_finite() {
                                    return Err(error("curve components must be finite"));
                                }
                                *word = value;
                            }
                            result.push(words);
                        }
                        Ok(result.into_boxed_slice())
                    };
                EffectSampler::Curve {
                    kind,
                    times: times.into_boxed_slice(),
                    values: pack(&curve.values)?,
                    tangents: pack(&curve.tangents)?,
                }
            }
            5 => {
                let native = self.document.shape_sampler(link)?;
                let primitive = match native.shape_type.unwrap_or(0) {
                    0 => Primitive::Box,
                    1 => Primitive::Sphere,
                    2 => Primitive::Ellipsoid,
                    3 => Primitive::Cylinder,
                    4 => Primitive::Capsule,
                    5 => Primitive::Cone,
                    _ => return Err(error("unsupported native shape primitive")),
                };
                let domain = match native.dimensionality.unwrap_or(2) {
                    2 => Domain::Surface,
                    3 => Domain::Volume,
                    _ => return Err(error("unsupported native shape sampling domain")),
                };
                // PKB editor properties use a distinct authoring frame. Convert once here.
                let author_to_native = Quat::from_rotation_x(FRAC_PI_2);
                let swap = |value: Vec3| Vec3::new(value.x, value.z, value.y);
                let shape = Shape {
                    primitive,
                    domain,
                    dimensions: swap(native.box_dimensions.unwrap_or(Vec3::splat(0.5))),
                    radius: native.radius.unwrap_or(1.),
                    inner_radius: native.inner_radius.unwrap_or(0.),
                    height: native.height.unwrap_or(0.),
                    hemisphere: native.hemisphere.unwrap_or(false),
                    scale: swap(native.nonuniform_scale.unwrap_or(Vec3::ONE)),
                    translation: author_to_native * native.position.unwrap_or(Vec3::ZERO),
                    rotation: author_to_native
                        * Shape::rotation(native.euler_degrees.unwrap_or(Vec3::ZERO))
                        * author_to_native.conjugate(),
                };
                shape.validate().map_err(error)?;
                EffectSampler::Shape(shape)
            }
            4 => {
                let stream = self.document.event_stream_sampler(link)?;
                let times: Vec<_> = stream.times.iter().collect();
                if times.iter().any(|time| !time.is_finite() || *time < 0.)
                    || times.windows(2).any(|window| window[0] > window[1])
                {
                    return Err(error(
                        "event-stream times must be finite, nonnegative and ordered",
                    ));
                }
                EffectSampler::Events(times.into_boxed_slice())
            }
            _ => return Err(error(format!("unsupported named sampler family {kind}"))),
        };
        let index = self.samplers.len();
        self.samplers.push(sampler);
        Ok(index)
    }
    fn bind(
        &self,
        blob: &CompilerBlob<'_>,
        fields: &[EffectField],
        field_slots: &HashMap<Vec<u8>, usize>,
        resources: &HashMap<Vec<u8>, External>,
    ) -> Result<BoundProgram, EffectCompileError> {
        let result = (|| {
            let script = blob.script()?;
            if blob.entrypoint != 0 {
                let entry = self.document.entrypoint(blob.entrypoint)?;
                let code_bytes = u32::from_le_bytes(blob.bytes[12..16].try_into().unwrap());
                if entry.begin != 0 || entry.end != code_bytes {
                    return Err(error(
                        "partial native script entrypoint execution is unsupported",
                    ));
                }
            }
            let mut externals = Vec::new();
            let mut types = Vec::new();
            for link in &blob.externals {
                let native = self.document.external_declaration(*link)?;
                let kind = native
                    .native_type
                    .value_type()
                    .ok_or_else(|| error("unsupported script external layout"))?;
                let binding = match native.name {
                    b"scene.time" => External::Time,
                    b"scene.dt" => External::Delta,
                    b"ParticleContextS" => External::ParticleContext,
                    b"ParticleContextI" => External::EffectContext,
                    b"RandContext" => External::RandomContext,
                    name => {
                        if let Some(index) = field_slots.get(name) {
                            if fields[*index].value_type != Some(kind) {
                                return Err(error(
                                    "script external and canonical field types differ",
                                ));
                            }
                            External::Field(*index)
                        } else if let Some(resource) = resources.get(name) {
                            *resource
                        } else if let Some(index) = self.attribute_slots.get(name) {
                            if self.attributes[*index].default.kind() != kind {
                                return Err(error(
                                    "script attribute layout differs from its default",
                                ));
                            }
                            External::Attribute(*index)
                        } else {
                            return Err(error(format!(
                                "unresolved script external {}",
                                String::from_utf8_lossy(name)
                            )));
                        }
                    }
                };
                let expected = match binding {
                    External::Time | External::Delta => ValueType::Float,
                    External::ParticleContext
                    | External::EffectContext
                    | External::RandomContext
                    | External::Event(_)
                    | External::Sampler(_) => ValueType::Context,
                    _ => kind,
                };
                if kind != expected {
                    return Err(error(
                        "built-in external declaration has incompatible layout",
                    ));
                }
                externals.push(binding);
                types.push(kind);
            }
            let mut callbacks = Vec::new();
            let mut signatures = Vec::new();
            for link in &blob.functions {
                let native = self.document.function_declaration(*link)?;
                let arguments = native
                    .arguments
                    .iter()
                    .map(|link| {
                        self.document
                            .function_argument(*link)?
                            .native_type
                            .value_type()
                            .ok_or(ReadError {
                                offset: 0,
                                message: "unsupported effect callback argument layout",
                            })
                    })
                    .collect::<Result<Vec<_>, ReadError>>()?;
                let receiver = if native.symbol_slot == u32::MAX {
                    None
                } else {
                    Some(u16::try_from(native.symbol_slot).map_err(|_| {
                        error("callback receiver exceeds native external-slot width")
                    })?)
                };
                let (callback, result) =
                    self.callback(native.name, &arguments, receiver, &externals)?;
                callbacks.push(callback);
                signatures.push(Signature {
                    receiver,
                    arguments,
                    result,
                });
            }
            let program = script
                .compile_with_limits(&types, &signatures, self.options.scalar_limits)
                .map_err(|failure| EffectCompileError {
                    layer: None,
                    stage: Some(blob.identifier),
                    offset: Some(failure.offset),
                    message: failure.message.into(),
                })?;
            Ok(BoundProgram {
                program,
                externals: externals.into_boxed_slice(),
                callbacks: callbacks.into_boxed_slice(),
            })
        })();
        result.map_err(|mut error| {
            error.stage = Some(blob.identifier);
            error
        })
    }
    fn callback(
        &self,
        name: &[u8],
        arguments: &[ValueType],
        receiver: Option<u16>,
        externals: &[External],
    ) -> Result<(Callback, Option<ValueType>), EffectCompileError> {
        use ValueType::{Bool, Context, Float, Float3, Float4, Int, Int2, Int3, Quaternion};
        let bound = receiver
            .map(|slot| {
                externals
                    .get(slot as usize)
                    .copied()
                    .ok_or_else(|| error("callback receiver exceeds external bindings"))
            })
            .transpose()?;
        let (callback, expected, result) = match name {
            b"scene.orientation_f_norm" => (Callback::Orientation, vec![Float3], Some(Quaternion)),
            b"orientation_axisSide" => (Callback::OrientationSide, vec![Quaternion], Some(Float3)),
            b"rotate" => (Callback::Rotate, vec![Float3, Quaternion], Some(Float3)),
            b"radians.rotate" => (
                Callback::RotateAbout,
                vec![Float3, Float3, Float],
                Some(Float3),
            ),
            b"effect.axisSide" => (Callback::EffectSide, vec![Context], Some(Float3)),
            b"effect.axisUp" => (Callback::EffectUp, vec![Context], Some(Float3)),
            b"hsv2rgb" => (Callback::HsvToRgb, vec![Float4], Some(Float4)),
            b"rgb2hsv" => (Callback::RgbToHsv, vec![Float4], Some(Float4)),
            b"effect.age" => (Callback::Age, vec![Context], Some(Float)),
            b"effect.isRunning" => (Callback::Running, vec![Context], Some(Bool)),
            b"duration" => {
                let Some(External::Sampler(index)) = bound else {
                    return Err(error("duration requires a bound event-stream sampler"));
                };
                if !matches!(self.samplers[index], EffectSampler::Events(_)) {
                    return Err(error("duration receiver must be an event stream"));
                }
                (Callback::StreamDuration, vec![Context], Some(Float))
            }
            b"trigger" => {
                if bound.is_some() {
                    return Err(error("trigger requires a free function"));
                }
                (Callback::Trigger, vec![Bool, Float, Context], Some(Int3))
            }
            b"generate" => {
                if let Some(bound) = bound {
                    if !matches!(bound,External::Sampler(index) if matches!(self.samplers[index],EffectSampler::Events(_)))
                    {
                        return Err(error("bound generate requires an event-stream sampler"));
                    }
                }
                (
                    Callback::Generate,
                    vec![Int2, Float, Float, Float, Float, Context],
                    Some(Int3),
                )
            }
            b"buildPayloadElement" => {
                if bound.is_some()
                    || arguments.len() != 5
                    || arguments[0] != Int3
                    || arguments[1] != arguments[2]
                    || !matches!(arguments[1], Float | Float3 | Float4 | Quaternion | Int)
                    || arguments[3] != Int
                    || arguments[4] != Context
                {
                    return Err(error("payload element requires free batch, matching typed endpoints, semantic and context"));
                }
                (Callback::BuildElement, arguments.to_vec(), Some(Int))
            }
            b"appendPayload" => {
                if !matches!(bound, Some(External::Event(_))) {
                    return Err(error("payload append requires an owned event receiver"));
                }
                (Callback::Append, vec![Int, Int, Int, Context], Some(Int))
            }
            b"extractPayloadElementF1"
            | b"extractPayloadElementF3"
            | b"extractPayloadElementF4"
            | b"extractPayloadElementI1"
            | b"extractPayloadElementO" => {
                let kind = match name {
                    b"extractPayloadElementF1" => Float,
                    b"extractPayloadElementF3" => Float3,
                    b"extractPayloadElementF4" => Float4,
                    b"extractPayloadElementI1" => Int,
                    _ => Quaternion,
                };
                (Callback::Extract(kind), vec![Int, Context], Some(kind))
            }
            b"initPayload" => {
                if !matches!(bound, Some(External::Event(_))) {
                    return Err(error("initPayload requires an owned graph event"));
                }
                (Callback::InitPayload, vec![Int, Int3, Context], Some(Int))
            }
            b"kick" => {
                if !matches!(bound, Some(External::Event(_))) {
                    return Err(error("kick requires an owned graph event"));
                }
                (Callback::Kick, vec![Int, Context, Context], None)
            }
            b"self.kill" => (
                Callback::Kill,
                if arguments.len() == 1 {
                    vec![Context]
                } else {
                    vec![Bool, Context]
                },
                None,
            ),
            b"vrand" => {
                let expected = if arguments.len() == 1 {
                    vec![Context]
                } else {
                    vec![Float, Float, Context]
                };
                (Callback::VRand, expected, Some(Float3))
            }
            b"rand" => {
                let kind = arguments
                    .first()
                    .copied()
                    .ok_or_else(|| error("rand requires an interval"))?;
                if !matches!(kind, Float | ValueType::Float2 | Float3 | ValueType::Float4) {
                    return Err(error("rand supports floating-point interval layouts"));
                }
                (Callback::Rand, vec![kind, kind, Context], Some(kind))
            }
            b"xform_w2l_f_masked" => (
                Callback::InverseTransform,
                vec![Float3, Int, Context],
                Some(Float3),
            ),
            b"xform_w2l_d_masked" => (
                Callback::InverseDirection,
                vec![Float3, Int, Context],
                Some(Float3),
            ),
            b"xform_l2w_f_masked" => (
                Callback::Transform,
                vec![Float3, Int, Context],
                Some(Float3),
            ),
            b"xform_l2w_d_masked" => (
                Callback::TransformDirection,
                vec![Float3, Int, Context],
                Some(Float3),
            ),
            b"samplePosition" => {
                let Some(External::Sampler(index)) = bound else {
                    return Err(error("samplePosition requires a bound shape sampler"));
                };
                if !matches!(self.samplers[index], EffectSampler::Shape(_)) {
                    return Err(error("samplePosition requires a shape sampler"));
                }
                (Callback::SamplePosition, vec![Context], Some(Float3))
            }
            b"sample" => {
                let Some(External::Sampler(index)) = bound else {
                    return Err(error("sample requires a bound sampler"));
                };
                let EffectSampler::Curve { kind, .. } = self.samplers[index] else {
                    return Err(error("sample requires a supported curve"));
                };
                (Callback::Sample, vec![Float], Some(kind))
            }
            _ => {
                return Err(error(format!(
                    "unsupported effect callback {}",
                    String::from_utf8_lossy(name)
                )))
            }
        };
        if arguments != expected {
            return Err(error(format!(
                "unsupported signature for callback {}",
                String::from_utf8_lossy(name)
            )));
        }
        if matches!(
            callback,
            Callback::Age
                | Callback::Extract(_)
                | Callback::Running
                | Callback::Kill
                | Callback::Rand
                | Callback::VRand
                | Callback::Transform
                | Callback::TransformDirection
                | Callback::InverseTransform
                | Callback::InverseDirection
                | Callback::HsvToRgb
                | Callback::RgbToHsv
                | Callback::Orientation
                | Callback::OrientationSide
                | Callback::Rotate
                | Callback::RotateAbout
                | Callback::EffectSide
                | Callback::EffectUp
        ) && bound.is_some()
        {
            return Err(error(
                "free effect callback unexpectedly has a bound receiver",
            ));
        }
        Ok((callback, result))
    }
}

#[cfg(test)]
#[path = "effect_compile_tests.rs"]
mod tests;
