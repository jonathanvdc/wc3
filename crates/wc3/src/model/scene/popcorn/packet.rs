//! Private instruction-major evolution for a small checked pure subset.
use super::curve::sample;
use super::execution::float_arithmetic;
use super::graph::{BoundProgram, Callback, EffectLayer, EffectSampler, External};
use super::runtime::FieldPlanes;
use super::{Address, BinaryOperation, Destination, Instruction, Source, ValueType};
use super::{
    SpecializationInstruction, SpecializationReason as Reason,
    SpecializationRejection as Rejection, SpecializationResource as Resource, SpecializationStatus,
};
use std::mem::size_of;

// Full tiles keep natural low-population effects on the cheap scalar path.
pub(super) const PARTICLE_BATCH_SIZE: usize = 16;
const VALUE_COMPONENTS: usize = 4;
type Planes<const BATCH: usize> = [[u32; BATCH]; VALUE_COMPONENTS];
#[derive(Clone, Copy, Debug)]
struct Register {
    slot: usize,
    width: usize,
}
#[derive(Clone, Copy, Debug)]
enum Location {
    Constant([u32; VALUE_COMPONENTS]),
    Register(usize),
}
#[derive(Clone, Copy, Debug)]
struct Operand {
    location: Location,
    width: usize,
}
impl Operand {
    #[inline]
    fn word<const BATCH: usize>(
        self,
        registers: &[Planes<BATCH>],
        lane: usize,
        component: usize,
    ) -> u32 {
        let component = component.min(self.width - 1);
        match self.location {
            Location::Constant(words) => words[component],
            Location::Register(slot) => registers[slot][component][lane],
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum Input {
    Field(usize),
    Time,
    Delta,
}
#[derive(Clone, Copy, Debug)]
enum InstructionPlan {
    Load {
        output: Register,
        input: Input,
    },
    Store {
        field: usize,
        input: Operand,
    },
    Binary {
        output: Register,
        left: Operand,
        right: Operand,
        operation: BinaryOperation,
    },
    Curve {
        output: Register,
        input: Operand,
        sampler: usize,
    },
}
#[derive(Clone, Debug)]
struct Operation {
    offset: usize,
    instruction: InstructionPlan,
}
#[derive(Clone, Debug)]
struct Field {
    canonical: usize,
    written: bool,
    read_before_write: bool,
    components: usize,
}
#[derive(Clone, Debug)]
pub(crate) struct PureEvolution {
    operations: Box<[Operation]>,
    fields: Box<[Field]>,
    registers: usize,
    locals: usize,
    #[cfg(test)]
    pub persistent_writes: Box<[(usize, usize)]>,
}
// Quaternion belongs to the VM float family but is intentionally outside
// this component-wise packet subset.
fn ordinary_float(kind: ValueType) -> bool {
    matches!(
        kind,
        ValueType::Float | ValueType::Float2 | ValueType::Float3 | ValueType::Float4
    )
}
fn supported(kind: ValueType) -> bool {
    ordinary_float(kind) || matches!(kind, ValueType::Bool | ValueType::Bool3)
}
fn local(scope: usize, slot: usize, counts: [usize; 4]) -> Option<usize> {
    if slot >= *counts.get(scope)? {
        return None;
    }
    counts[..scope]
        .iter()
        .try_fold(slot, |offset, count| offset.checked_add(*count))
}
fn readable(
    operand: Operand,
    defined: &[bool],
    counts: [usize; 4],
    offset: usize,
) -> Result<(), Rejection> {
    if let Location::Register(slot) = operand.location {
        if !defined[slot] {
            let mut local = slot;
            for (scope, count) in counts.into_iter().enumerate() {
                if local < count {
                    return Err(
                        Rejection::new(Reason::PersistentInput { scope, slot: local }).at(offset),
                    );
                }
                local -= count;
            }
            return Err(Rejection::new(Reason::InvalidBinding).at(offset));
        }
    }
    Ok(())
}
fn output(
    destination: Destination,
    counts: [usize; 4],
    offset: usize,
) -> Result<Register, Rejection> {
    if !supported(destination.kind) {
        return Err(Rejection::new(Reason::Layout(destination.kind)).at(offset));
    }
    Ok(Register {
        slot: local(destination.scope, destination.slot, counts)
            .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?,
        width: destination.kind.components(),
    })
}
fn operand(source: Source, counts: [usize; 4], offset: usize) -> Result<Operand, Rejection> {
    if !supported(source.kind) {
        return Err(Rejection::new(Reason::Layout(source.kind)).at(offset));
    }
    Ok(Operand {
        width: source.kind.components(),
        location: match source.address {
            Address::Constant(words) => Location::Constant(words),
            Address::Slot { scope, slot } => Location::Register(
                local(scope, slot, counts)
                    .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?,
            ),
        },
    })
}
fn field(fields: &mut Vec<Field>, canonical: usize, components: usize, written: bool) -> usize {
    if let Some(index) = fields.iter().position(|field| field.canonical == canonical) {
        let field = &mut fields[index];
        debug_assert_eq!(field.components, components);
        field.read_before_write |= !written && !field.written;
        field.written |= written;
        index
    } else {
        let index = fields.len();
        fields.push(Field {
            canonical,
            written,
            read_before_write: !written,
            components,
        });
        index
    }
}
impl PureEvolution {
    #[cfg(test)]
    fn prepare(
        layer: &EffectLayer,
        bound: &BoundProgram,
        samplers: &[EffectSampler],
    ) -> Option<Self> {
        Self::analyze(layer, bound, samplers).ok()
    }
    pub(super) fn analyze(
        layer: &EffectLayer,
        bound: &BoundProgram,
        samplers: &[EffectSampler],
    ) -> Result<Self, Rejection> {
        if layer.spawner {
            return Err(Rejection::new(Reason::Spawner));
        }
        if !layer.rendered {
            return Err(Rejection::new(Reason::NotRendered));
        }
        let counts = bound.program.counts;
        let address_limit = || {
            Rejection::new(Reason::Limit {
                resource: Resource::RegisterAddress,
                limit: isize::MAX as usize,
            })
        };
        let registers = counts
            .iter()
            .try_fold(0_usize, |total, count| total.checked_add(*count))
            .ok_or_else(address_limit)?;
        let locals = counts[0].checked_add(counts[1]).ok_or_else(address_limit)?;
        let mut defined = vec![false; registers];
        defined[..locals].fill(true);
        #[cfg(test)]
        let mut persistent_writes = Vec::new();
        let mut fields = Vec::new();
        let mut operations = Vec::with_capacity(bound.program.operations.len());
        for operation in &bound.program.operations {
            let offset = operation.offset;
            let instruction = match &operation.instruction {
                Instruction::Load {
                    destination,
                    external,
                } => {
                    let input = match bound
                        .externals
                        .get(usize::from(*external))
                        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?
                    {
                        External::Field(index) => {
                            if layer
                                .fields
                                .get(*index)
                                .ok_or_else(|| {
                                    Rejection::new(Reason::InvalidBinding)
                                        .at(offset)
                                        .field(*index)
                                })?
                                .value_type
                                != Some(destination.kind)
                            {
                                return Err(Rejection::new(Reason::InvalidBinding)
                                    .at(offset)
                                    .field(*index));
                            }
                            Input::Field(field(
                                &mut fields,
                                *index,
                                destination.kind.components(),
                                false,
                            ))
                        }
                        External::Time if destination.kind == ValueType::Float => Input::Time,
                        External::Delta if destination.kind == ValueType::Float => Input::Delta,
                        binding => return Err(Rejection::external(*external, binding).at(offset)),
                    };
                    InstructionPlan::Load {
                        output: output(*destination, counts, offset)?,
                        input,
                    }
                }
                Instruction::Store { source, external } => {
                    let binding = bound
                        .externals
                        .get(usize::from(*external))
                        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?;
                    let External::Field(index) = binding else {
                        return Err(Rejection::external(*external, binding).at(offset));
                    };
                    if *index == layer.life || *index == layer.inverse_life {
                        return Err(Rejection::new(Reason::LifecycleWrite)
                            .at(offset)
                            .field(*index));
                    }
                    if layer
                        .fields
                        .get(*index)
                        .ok_or_else(|| {
                            Rejection::new(Reason::InvalidBinding)
                                .at(offset)
                                .field(*index)
                        })?
                        .value_type
                        != Some(source.kind)
                    {
                        return Err(Rejection::new(Reason::InvalidBinding)
                            .at(offset)
                            .field(*index));
                    }
                    InstructionPlan::Store {
                        field: field(&mut fields, *index, source.kind.components(), true),
                        input: operand(*source, counts, offset)?,
                    }
                }
                Instruction::Binary {
                    destination,
                    left,
                    right,
                    operation,
                } => {
                    if !matches!(
                        operation,
                        BinaryOperation::Add
                            | BinaryOperation::Subtract
                            | BinaryOperation::Multiply
                    ) {
                        return Err(Rejection::new(Reason::Instruction(
                            SpecializationInstruction::Arithmetic,
                        ))
                        .at(offset));
                    }
                    for kind in [destination.kind, left.kind, right.kind] {
                        if !ordinary_float(kind) {
                            return Err(Rejection::new(Reason::Layout(kind)).at(offset));
                        }
                    }
                    InstructionPlan::Binary {
                        output: output(*destination, counts, offset)?,
                        left: operand(*left, counts, offset)?,
                        right: operand(*right, counts, offset)?,
                        operation: *operation,
                    }
                }
                Instruction::Call {
                    destination: Some(destination),
                    result_kind: Some(ValueType::Float4),
                    receiver: Some(receiver),
                    function,
                    arguments,
                    ..
                } if destination.kind == ValueType::Float4
                    && arguments.len() == 1
                    && arguments[0].kind == ValueType::Float =>
                {
                    let callback = bound
                        .callbacks
                        .get(usize::from(*function))
                        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?;
                    if callback != &Callback::Sample {
                        return Err(Rejection::new(Reason::Instruction(
                            SpecializationInstruction::Call,
                        ))
                        .at(offset));
                    }
                    let binding = bound
                        .externals
                        .get(usize::from(*receiver))
                        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?;
                    let External::Sampler(index) = binding else {
                        return Err(Rejection::external(*receiver, binding).at(offset));
                    };
                    if !matches!(
                        samplers
                            .get(*index)
                            .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?,
                        EffectSampler::Curve {
                            kind: ValueType::Float4,
                            ..
                        }
                    ) {
                        return Err(
                            Rejection::new(Reason::CurveResource { sampler: *index }).at(offset)
                        );
                    }
                    InstructionPlan::Curve {
                        output: output(*destination, counts, offset)?,
                        input: operand(arguments[0], counts, offset)?,
                        sampler: *index,
                    }
                }
                instruction => {
                    return Err(Rejection::new(Reason::Instruction(instruction.into())).at(offset))
                }
            };
            // Read all operands before defining the output. Persistent slots
            // must be definitely assigned within this selected evolution;
            // initializer writes cannot satisfy this proof on the next frame.
            let output = match instruction {
                InstructionPlan::Load { output, .. } => Some(output),
                InstructionPlan::Store { input, .. } => {
                    readable(input, &defined, counts, offset)?;
                    None
                }
                InstructionPlan::Binary {
                    output,
                    left,
                    right,
                    ..
                } => {
                    readable(left, &defined, counts, offset)?;
                    readable(right, &defined, counts, offset)?;
                    Some(output)
                }
                InstructionPlan::Curve { output, input, .. } => {
                    readable(input, &defined, counts, offset)?;
                    Some(output)
                }
            };
            if let Some(output) = output {
                defined[output.slot] = true;
                #[cfg(test)]
                if output.slot >= locals {
                    let scope = if output.slot < locals + counts[2] {
                        2
                    } else {
                        3
                    };
                    let slot = output.slot
                        - if scope == 2 {
                            locals
                        } else {
                            locals + counts[2]
                        };
                    if !persistent_writes.contains(&(scope, slot)) {
                        persistent_writes.push((scope, slot));
                    }
                }
            }
            operations.push(Operation {
                offset: operation.offset,
                instruction,
            });
        }
        Ok(Self {
            operations: operations.into_boxed_slice(),
            fields: fields.into_boxed_slice(),
            registers,
            locals,
            #[cfg(test)]
            persistent_writes: persistent_writes.into_boxed_slice(),
        })
    }
}
pub(crate) fn prepare(layers: &mut [EffectLayer], samplers: &[EffectSampler]) {
    for layer in layers {
        let Some(bound) = layer.evolve.as_ref() else {
            layer.pure_evolution = None;
            layer.pure_evolution_status = SpecializationStatus::NotApplicable;
            continue;
        };
        match PureEvolution::analyze(layer, bound, samplers) {
            Ok(plan) => {
                layer.pure_evolution = Some(Box::new(plan));
                layer.pure_evolution_status = SpecializationStatus::Accepted;
            }
            Err(reason) => {
                layer.pure_evolution = None;
                layer.pure_evolution_status = SpecializationStatus::Rejected(reason);
            }
        }
    }
}
#[derive(Clone, Copy, Default)]
pub(super) struct Lane {
    pub slot: usize,
    pub delta: f64,
    pub life: f32,
}
pub(super) type Scratch = BatchScratch<PARTICLE_BATCH_SIZE>;

/// Private fixed-size storage. The runtime uses only the default Scratch alias.
pub(super) struct BatchScratch<const BATCH: usize> {
    registers: Vec<Planes<BATCH>>,
    fields: Vec<Planes<BATCH>>,
    pub lanes: [Lane; BATCH],
    pub count: usize,
    #[cfg(test)]
    pub attempts: usize,
    #[cfg(test)]
    pub committed: usize,
    #[cfg(test)]
    pub fallbacks: usize,
}
impl<const BATCH: usize> BatchScratch<BATCH> {
    pub fn shape(layers: &[EffectLayer]) -> Option<(usize, usize)> {
        let mut result = None;
        for plan in layers
            .iter()
            .filter_map(|layer| layer.evolve.as_ref().and(layer.pure_evolution.as_deref()))
        {
            let shape = result.get_or_insert((0usize, 0usize));
            shape.0 = shape.0.max(plan.registers);
            shape.1 = shape.1.max(plan.fields.len());
        }
        result
    }
    pub fn bytes((registers, fields): (usize, usize)) -> Option<usize> {
        if BATCH == 0 {
            return None;
        }
        registers
            .checked_add(fields)?
            .checked_mul(size_of::<Planes<BATCH>>())?
            .checked_add(size_of::<Self>())
    }
    pub fn new((registers, fields): (usize, usize)) -> Box<Self> {
        assert!(BATCH > 0, "particle batch must not be empty");
        Box::new(Self {
            registers: vec![[[0; BATCH]; VALUE_COMPONENTS]; registers],
            fields: vec![[[0; BATCH]; VALUE_COMPONENTS]; fields],
            lanes: [Lane::default(); BATCH],
            count: 0,
            #[cfg(test)]
            attempts: 0,
            #[cfg(test)]
            committed: 0,
            #[cfg(test)]
            fallbacks: 0,
        })
    }
    pub fn stage(&mut self, plan: &PureEvolution, fields: &FieldPlanes, life_field: usize) {
        // One range proof per batch, shared by every gathered component plane.
        let contiguous = self.lanes[..self.count].first().and_then(|first| {
            let end = first.slot.checked_add(self.count)?;
            self.lanes[..self.count]
                .iter()
                .enumerate()
                .all(|(index, lane)| lane.slot == first.slot + index)
                .then_some((first.slot, end))
        });
        for (index, binding) in plan
            .fields
            .iter()
            .enumerate()
            .filter(|(_, binding)| binding.read_before_write)
        {
            if binding.canonical == life_field {
                for (word, lane) in self.fields[index][0][..self.count]
                    .iter_mut()
                    .zip(&self.lanes[..self.count])
                {
                    *word = lane.life.to_bits();
                }
                continue;
            }
            for component in 0..binding.components {
                let input = fields
                    .component(binding.canonical, component)
                    .expect("prepared packet component has physical storage");
                let output = &mut self.fields[index][component][..self.count];
                if let Some((start, end)) = contiguous {
                    output.copy_from_slice(&input[start..end]);
                } else {
                    for (word, lane) in output.iter_mut().zip(&self.lanes[..self.count]) {
                        *word = input[lane.slot];
                    }
                }
            }
        }
    }
    pub fn execute(
        &mut self,
        plan: &PureEvolution,
        samplers: &[EffectSampler],
        time: f64,
    ) -> Result<(), usize> {
        // Only the Rust evaluator uses retained register planes. Native SIMD
        // initializes its invocation locals in SSA and needs no scratch clearing.
        // Persistent reads are proven to follow a write within this invocation.
        self.registers[..plan.locals].fill([[0; BATCH]; VALUE_COMPONENTS]);
        #[cfg(test)]
        {
            self.attempts += 1;
        }
        for operation in &plan.operations {
            match operation.instruction {
                InstructionPlan::Load { output, input } => {
                    let mut result = [[0; BATCH]; VALUE_COMPONENTS];
                    for lane in 0..self.count {
                        match input {
                            Input::Field(index) => {
                                for (component, words) in
                                    result.iter_mut().enumerate().take(output.width)
                                {
                                    words[lane] = self.fields[index][component][lane];
                                }
                            }
                            Input::Time | Input::Delta => {
                                let value = if matches!(input, Input::Time) {
                                    time as f32
                                } else {
                                    self.lanes[lane].delta as f32
                                };
                                if !value.is_finite() {
                                    return Err(operation.offset);
                                }
                                result[0][lane] = value.to_bits();
                            }
                        }
                    }
                    self.registers[output.slot] = result;
                }
                InstructionPlan::Store { field, input } => {
                    let mut result = [[0; BATCH]; VALUE_COMPONENTS];
                    for (component, words) in result.iter_mut().enumerate().take(input.width) {
                        for (lane, word) in words.iter_mut().enumerate().take(self.count) {
                            *word = input.word(&self.registers, lane, component);
                        }
                    }
                    self.fields[field] = result;
                }
                InstructionPlan::Binary {
                    output,
                    left,
                    right,
                    operation,
                } => {
                    let mut result = [[0; BATCH]; VALUE_COMPONENTS];
                    for (component, words) in result.iter_mut().enumerate().take(output.width) {
                        for (lane, word) in words.iter_mut().enumerate().take(self.count) {
                            *word = float_arithmetic(
                                operation,
                                left.word(&self.registers, lane, component),
                                right.word(&self.registers, lane, component),
                            );
                        }
                    }
                    self.registers[output.slot] = result;
                }
                InstructionPlan::Curve {
                    output,
                    input,
                    sampler,
                } => {
                    let mut result = [[0; BATCH]; VALUE_COMPONENTS];
                    for (lane, _) in self.lanes.iter().enumerate().take(self.count) {
                        let value = sample(
                            &samplers[sampler],
                            f32::from_bits(input.word(&self.registers, lane, 0)),
                        )
                        .map_err(|_| operation.offset)?;
                        for (component, word) in value.words().into_iter().enumerate() {
                            result[component][lane] = word;
                        }
                    }
                    self.registers[output.slot] = result;
                }
            }
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn poison(&mut self) {
        self.registers.fill([[0xdeadbeef; BATCH]; VALUE_COMPONENTS]);
        self.fields.fill([[0x7fc00001; BATCH]; VALUE_COMPONENTS]);
    }
    pub fn commit(&self, plan: &PureEvolution, lane: usize, fields: &mut FieldPlanes) {
        for (index, binding) in plan
            .fields
            .iter()
            .enumerate()
            .filter(|(_, binding)| binding.written)
        {
            fields.write(
                binding.canonical,
                self.lanes[lane].slot,
                self.fields[index].map(|plane| plane[lane]),
            );
        }
    }
}

#[cfg(test)]
#[path = "packet_tests.rs"]
pub(super) mod tests;

#[cfg(all(
    feature = "popcorn-jit",
    any(unix, windows),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub(crate) mod simd;
