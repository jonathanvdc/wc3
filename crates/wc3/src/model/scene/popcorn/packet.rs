//! Private instruction-major evolution for a small checked pure subset.
use super::curve::sample;
use super::execution::float_arithmetic;
use super::graph::{BoundProgram, Callback, EffectLayer, EffectSampler, External};
use super::runtime::FieldPlanes;
use super::{Address, BinaryOperation, Destination, Instruction, Source, ValueType};
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
fn supported(kind: ValueType) -> bool {
    matches!(
        kind,
        ValueType::Bool
            | ValueType::Bool3
            | ValueType::Float
            | ValueType::Float3
            | ValueType::Float4
    )
}
fn local(scope: usize, slot: usize, counts: [usize; 4]) -> Option<usize> {
    if slot >= *counts.get(scope)? {
        return None;
    }
    counts[..scope]
        .iter()
        .try_fold(slot, |offset, count| offset.checked_add(*count))
}
fn readable(operand: Operand, defined: &[bool]) -> bool {
    match operand.location {
        Location::Constant(_) => true,
        Location::Register(slot) => defined[slot],
    }
}

fn output(destination: Destination, counts: [usize; 4]) -> Option<Register> {
    supported(destination.kind).then_some(())?;
    Some(Register {
        slot: local(destination.scope, destination.slot, counts)?,
        width: destination.kind.components(),
    })
}
fn operand(source: Source, counts: [usize; 4]) -> Option<Operand> {
    supported(source.kind).then_some(())?;
    Some(Operand {
        width: source.kind.components(),
        location: match source.address {
            Address::Constant(words) => Location::Constant(words),
            Address::Slot { scope, slot } => Location::Register(local(scope, slot, counts)?),
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
    fn prepare(
        layer: &EffectLayer,
        bound: &BoundProgram,
        samplers: &[EffectSampler],
    ) -> Option<Self> {
        if layer.spawner || !layer.rendered {
            return None;
        }
        let counts = bound.program.counts;
        let registers = counts
            .iter()
            .try_fold(0_usize, |total, count| total.checked_add(*count))?;
        let locals = counts[0].checked_add(counts[1])?;
        let mut defined = vec![false; registers];
        defined[..locals].fill(true);
        #[cfg(test)]
        let mut persistent_writes = Vec::new();
        let mut fields = Vec::new();
        let mut operations = Vec::with_capacity(bound.program.operations.len());
        for operation in &bound.program.operations {
            let instruction = match &operation.instruction {
                Instruction::Load {
                    destination,
                    external,
                } => {
                    let input = match bound.externals.get(usize::from(*external))? {
                        External::Field(index) => {
                            if layer.fields.get(*index)?.value_type != Some(destination.kind) {
                                return None;
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
                        _ => return None,
                    };
                    InstructionPlan::Load {
                        output: output(*destination, counts)?,
                        input,
                    }
                }
                Instruction::Store { source, external } => {
                    let External::Field(index) = bound.externals.get(usize::from(*external))?
                    else {
                        return None;
                    };
                    if *index == layer.life
                        || *index == layer.inverse_life
                        || layer.fields.get(*index)?.value_type != Some(source.kind)
                    {
                        return None;
                    }
                    InstructionPlan::Store {
                        field: field(&mut fields, *index, source.kind.components(), true),
                        input: operand(*source, counts)?,
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
                    ) || !matches!(destination.kind, ValueType::Float | ValueType::Float4)
                        || !matches!(left.kind, ValueType::Float | ValueType::Float4)
                        || !matches!(right.kind, ValueType::Float | ValueType::Float4)
                    {
                        return None;
                    }
                    InstructionPlan::Binary {
                        output: output(*destination, counts)?,
                        left: operand(*left, counts)?,
                        right: operand(*right, counts)?,
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
                    if bound.callbacks.get(usize::from(*function)) != Some(&Callback::Sample) {
                        return None;
                    }
                    let External::Sampler(index) = bound.externals.get(usize::from(*receiver))?
                    else {
                        return None;
                    };
                    if !matches!(
                        samplers.get(*index)?,
                        EffectSampler::Curve {
                            kind: ValueType::Float4,
                            ..
                        }
                    ) {
                        return None;
                    }
                    InstructionPlan::Curve {
                        output: output(*destination, counts)?,
                        input: operand(arguments[0], counts)?,
                        sampler: *index,
                    }
                }
                _ => return None,
            };
            // Read all operands before defining the output. Persistent slots
            // must be definitely assigned within this selected evolution;
            // initializer writes cannot satisfy this proof on the next frame.
            let output = match instruction {
                InstructionPlan::Load { output, .. } => Some(output),
                InstructionPlan::Store { input, .. } => {
                    if !readable(input, &defined) {
                        return None;
                    }
                    None
                }
                InstructionPlan::Binary {
                    output,
                    left,
                    right,
                    ..
                } => {
                    if !readable(left, &defined) || !readable(right, &defined) {
                        return None;
                    }
                    Some(output)
                }
                InstructionPlan::Curve { output, input, .. } => {
                    if !readable(input, &defined) {
                        return None;
                    }
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
        Some(Self {
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
        let plan = layer
            .evolve
            .as_ref()
            .and_then(|bound| PureEvolution::prepare(layer, bound, samplers));
        layer.pure_evolution = plan.map(Box::new);
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
