//! Optional closed age expressions derived from the checked selected evolution.
//! This module performs no admission, scheduling, canonical writes or GPU work.
use super::curve::sample;
use super::execution::float_arithmetic;
use super::graph::{Callback, EffectLayer, EffectSampler, External};
use super::{Address, BinaryOperation, Destination, Instruction, Source, Value, ValueType};
use std::fmt::{Display, Formatter, Result as FmtResult};

pub(super) const MAX_NODES: usize = 128;
pub(super) const MAX_CAPTURES: usize = 64;
pub(super) const MAX_OUTPUTS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NodeId(pub(super) usize);
#[derive(Clone, Copy, Debug)]
pub(super) struct Capture {
    pub field: usize,
    pub kind: ValueType,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Expression {
    Constant(Value),
    Capture(usize),
    Life,
    Time,
    Words {
        input: NodeId,
        kind: ValueType,
    },
    Binary {
        operation: BinaryOperation,
        left: NodeId,
        right: NodeId,
    },
    Curve {
        sampler: usize,
        input: NodeId,
    },
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Node {
    pub expression: Expression,
    pub kind: ValueType,
    pub offset: usize,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Output {
    pub field: usize,
    pub value: NodeId,
}
#[derive(Debug)]
pub(super) struct AgePlan {
    pub captures: Box<[Capture]>,
    pub nodes: Box<[Node]>,
    pub outputs: Box<[Output]>,
    pub phases: usize,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct AgeSample {
    pub effect_time: f64,
    pub life: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AgeError {
    pub offset: Option<usize>,
    pub message: &'static str,
}
#[derive(Clone, Copy, Debug)]
enum Symbol {
    Constant(Value),
    Field(usize),
    Time,
    Words {
        input: usize,
        kind: ValueType,
    },
    Binary {
        operation: BinaryOperation,
        left: usize,
        right: usize,
    },
    Curve {
        sampler: usize,
        input: usize,
    },
}
#[derive(Clone, Copy, Debug)]
struct SymbolNode {
    symbol: Symbol,
    kind: ValueType,
    offset: usize,
}
#[derive(Clone, Copy, Debug)]
struct Phase {
    expression: usize,
    phase: usize,
    rate: usize,
    clock: usize,
    time: usize,
}
struct Builder<'a> {
    symbols: &'a [SymbolNode],
    layer: &'a EffectLayer,
    written: &'a [bool],
    invariant: &'a [bool],
    phases: &'a [Phase],
    captures: Vec<Capture>,
    nodes: Vec<Node>,
    memo: Vec<Option<NodeId>>,
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
fn register(scope: usize, slot: usize, counts: [usize; 4]) -> Option<usize> {
    (slot < *counts.get(scope)?).then_some(())?;
    counts[..scope]
        .iter()
        .try_fold(slot, |a, b| a.checked_add(*b))
}
fn push(
    symbols: &mut Vec<SymbolNode>,
    symbol: Symbol,
    kind: ValueType,
    offset: usize,
) -> Option<usize> {
    supported(kind).then_some(())?;
    if symbols.len() >= MAX_NODES {
        return None;
    }
    let result = symbols.len();
    symbols.push(SymbolNode {
        symbol,
        kind,
        offset,
    });
    Some(result)
}
fn source(
    symbols: &mut Vec<SymbolNode>,
    registers: &[Option<usize>],
    counts: [usize; 4],
    source: Source,
    offset: usize,
) -> Option<usize> {
    supported(source.kind).then_some(())?;
    match source.address {
        Address::Constant(words) => push(
            symbols,
            Symbol::Constant(Value::from_words(source.kind, words)),
            source.kind,
            offset,
        ),
        Address::Slot { scope, slot } => {
            let index = registers[register(scope, slot, counts)?]?;
            if symbols[index].kind == source.kind {
                Some(index)
            } else {
                push(
                    symbols,
                    Symbol::Words {
                        input: index,
                        kind: source.kind,
                    },
                    source.kind,
                    offset,
                )
            }
        }
    }
}
fn set(
    registers: &mut [Option<usize>],
    counts: [usize; 4],
    destination: Destination,
    value: usize,
) -> Option<()> {
    supported(destination.kind).then_some(())?;
    *registers.get_mut(register(destination.scope, destination.slot, counts)?)? = Some(value);
    Some(())
}
fn old_field(symbols: &[SymbolNode], index: usize) -> Option<usize> {
    if let Symbol::Field(field) = symbols[index].symbol {
        Some(field)
    } else {
        None
    }
}
fn phase(
    symbols: &[SymbolNode],
    expression: usize,
    field: usize,
    final_fields: &[Option<usize>],
    written: &[bool],
) -> Option<Phase> {
    // Exactly one immutable angular rate and a paired last-time update. No names,
    // native selectors, or asset identity participate in this certificate.
    let Symbol::Binary {
        operation: BinaryOperation::Add,
        left,
        right,
    } = symbols[expression].symbol
    else {
        return None;
    };
    for (previous, increment) in [(left, right), (right, left)] {
        if old_field(symbols, previous) != Some(field) {
            continue;
        }
        let Symbol::Binary {
            operation: BinaryOperation::Multiply,
            left,
            right,
        } = symbols[increment].symbol
        else {
            continue;
        };
        for (rate, delta) in [(left, right), (right, left)] {
            let Some(rate) = old_field(symbols, rate) else {
                continue;
            };
            if written[rate] {
                continue;
            }
            let Symbol::Binary {
                operation: BinaryOperation::Subtract,
                left,
                right,
            } = symbols[delta].symbol
            else {
                continue;
            };
            if !matches!(symbols[left].symbol, Symbol::Time) {
                continue;
            }
            let Some(clock) = old_field(symbols, right) else {
                continue;
            };
            if clock == field || clock == rate {
                continue;
            }
            if final_fields[clock]
                .is_some_and(|index| matches!(symbols[index].symbol, Symbol::Time))
            {
                return Some(Phase {
                    expression,
                    phase: field,
                    rate,
                    clock,
                    time: left,
                });
            }
        }
    }
    None
}
impl Builder<'_> {
    fn node(&mut self, expression: Expression, kind: ValueType, offset: usize) -> Option<NodeId> {
        if self.nodes.len() >= MAX_NODES {
            return None;
        }
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            expression,
            kind,
            offset,
        });
        Some(id)
    }
    fn capture(&mut self, field: usize, offset: usize) -> Option<NodeId> {
        let kind = self.layer.fields.get(field)?.value_type?;
        supported(kind).then_some(())?;
        let index = if let Some(index) = self
            .captures
            .iter()
            .position(|capture| capture.field == field)
        {
            index
        } else {
            if self.captures.len() >= MAX_CAPTURES {
                return None;
            }
            let index = self.captures.len();
            self.captures.push(Capture { field, kind });
            index
        };
        self.node(Expression::Capture(index), kind, offset)
    }
    fn lower(&mut self, index: usize, allow_time: bool) -> Option<NodeId> {
        // Time is permitted only inside the certified phase and as its paired
        // clock store. Never memoize an otherwise disallowed Time use.
        let input = self.symbols[index];
        if matches!(input.symbol, Symbol::Time) {
            return allow_time
                .then_some(())
                .and_then(|_| self.node(Expression::Time, ValueType::Float, input.offset));
        }
        if let Some(result) = self.memo[index] {
            return Some(result);
        }
        if let Some(phase) = self
            .phases
            .iter()
            .find(|phase| phase.expression == index)
            .copied()
        {
            let initial = self.capture(phase.phase, input.offset)?;
            let rate = self.capture(phase.rate, input.offset)?;
            let clock = self.capture(phase.clock, input.offset)?;
            let time = self.node(
                Expression::Time,
                ValueType::Float,
                self.symbols[phase.time].offset,
            )?;
            let delta = self.node(
                Expression::Binary {
                    operation: BinaryOperation::Subtract,
                    left: time,
                    right: clock,
                },
                ValueType::Float,
                input.offset,
            )?;
            let increment = self.node(
                Expression::Binary {
                    operation: BinaryOperation::Multiply,
                    left: rate,
                    right: delta,
                },
                ValueType::Float,
                input.offset,
            )?;
            let result = self.node(
                Expression::Binary {
                    operation: BinaryOperation::Add,
                    left: initial,
                    right: increment,
                },
                ValueType::Float,
                input.offset,
            )?;
            self.memo[index] = Some(result);
            return Some(result);
        }
        let expression = match input.symbol {
            Symbol::Constant(value) => Expression::Constant(value),
            Symbol::Field(field) if field == self.layer.life => Expression::Life,
            Symbol::Field(field) if !self.written[field] || self.invariant[field] => {
                let result = self.capture(field, input.offset)?;
                self.memo[index] = Some(result);
                return Some(result);
            }
            Symbol::Field(_) | Symbol::Time => return None,
            Symbol::Words { input, kind } => Expression::Words {
                input: self.lower(input, false)?,
                kind,
            },
            Symbol::Binary {
                operation,
                left,
                right,
            } => Expression::Binary {
                operation,
                left: self.lower(left, false)?,
                right: self.lower(right, false)?,
            },
            Symbol::Curve { sampler, input } => Expression::Curve {
                sampler,
                input: self.lower(input, false)?,
            },
        };
        let result = self.node(expression, input.kind, input.offset)?;
        self.memo[index] = Some(result);
        Some(result)
    }
}
impl AgePlan {
    /// First portable offload subset: every potentially failing sample uses
    /// finite lifecycle Life, including samples whose result is otherwise unused.
    pub fn total_life_curves(&self, samplers: &[EffectSampler]) -> bool {
        fn life(plan: &AgePlan, id: NodeId) -> bool {
            match plan.nodes[id.0].expression {
                Expression::Life => true,
                Expression::Words {
                    input,
                    kind: ValueType::Float,
                } => life(plan, input),
                _ => false,
            }
        }
        self.nodes.iter().all(|node| match node.expression {
            Expression::Curve { sampler, input } => life(self, input)
                && matches!(samplers.get(sampler), Some(EffectSampler::Curve { times, values, tangents, .. })
                    if !times.is_empty() && values.len() == times.len()
                    && (tangents.is_empty() || tangents.len() == 2 * times.len())
                    && times.iter().all(|t| t.is_finite())
                    && times.windows(2).all(|t| t[0] < t[1])),
            _ => true,
        })
    }
    pub fn permitted(&self, sample: AgeSample) -> bool {
        // The renderer's analytic life also uses an f32 time anchor, even when
        // the extracted source contains no explicit Time node.
        sample.life.is_finite() && (sample.effect_time as f32).is_finite()
    }
    pub fn prepare(layer: &EffectLayer, samplers: &[EffectSampler]) -> Option<Self> {
        if layer.spawner || !layer.rendered || layer.evolve_on_death {
            return None;
        }
        let bound = layer.evolve.as_ref()?;
        let counts = bound.program.counts;
        let total = counts.iter().try_fold(0usize, |a, b| a.checked_add(*b))?;
        let locals = counts[0].checked_add(counts[1])?;
        let mut symbols = Vec::new();
        let zero = push(
            &mut symbols,
            Symbol::Constant(Value::from_words(ValueType::Float4, [0; 4])),
            ValueType::Float4,
            0,
        )?;
        let mut registers = vec![None; total];
        registers[..locals].fill(Some(zero));
        let mut fields = vec![None; layer.fields.len()];
        let mut written = vec![false; layer.fields.len()];
        for op in &bound.program.operations {
            match &op.instruction {
                Instruction::Load {
                    destination,
                    external,
                } => {
                    let value = match bound.externals.get(usize::from(*external))? {
                        External::Field(field) => {
                            if layer.fields.get(*field)?.value_type != Some(destination.kind) {
                                return None;
                            }
                            if let Some(value) = fields[*field] {
                                value
                            } else {
                                let value = push(
                                    &mut symbols,
                                    Symbol::Field(*field),
                                    destination.kind,
                                    op.offset,
                                )?;
                                fields[*field] = Some(value);
                                value
                            }
                        }
                        External::Time if destination.kind == ValueType::Float => {
                            push(&mut symbols, Symbol::Time, ValueType::Float, op.offset)?
                        }
                        _ => return None,
                    };
                    set(&mut registers, counts, *destination, value)?;
                }
                Instruction::Store {
                    source: input,
                    external,
                } => {
                    let External::Field(field) = bound.externals.get(usize::from(*external))?
                    else {
                        return None;
                    };
                    if *field == layer.life
                        || *field == layer.inverse_life
                        || layer.fields.get(*field)?.value_type != Some(input.kind)
                    {
                        return None;
                    }
                    fields[*field] =
                        Some(source(&mut symbols, &registers, counts, *input, op.offset)?);
                    written[*field] = true;
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
                    let left = source(&mut symbols, &registers, counts, *left, op.offset)?;
                    let right = source(&mut symbols, &registers, counts, *right, op.offset)?;
                    let value = push(
                        &mut symbols,
                        Symbol::Binary {
                            operation: *operation,
                            left,
                            right,
                        },
                        destination.kind,
                        op.offset,
                    )?;
                    set(&mut registers, counts, *destination, value)?;
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
                    let External::Sampler(sampler) = bound.externals.get(usize::from(*receiver))?
                    else {
                        return None;
                    };
                    if !matches!(
                        samplers.get(*sampler)?,
                        EffectSampler::Curve {
                            kind: ValueType::Float4,
                            ..
                        }
                    ) {
                        return None;
                    }
                    let input = source(&mut symbols, &registers, counts, arguments[0], op.offset)?;
                    let value = push(
                        &mut symbols,
                        Symbol::Curve {
                            sampler: *sampler,
                            input,
                        },
                        ValueType::Float4,
                        op.offset,
                    )?;
                    set(&mut registers, counts, *destination, value)?;
                }
                _ => return None,
            }
        }
        let phases: Vec<_> = fields
            .iter()
            .enumerate()
            .filter(|(field, _)| written[*field])
            .filter_map(|(field, value)| phase(&symbols, (*value)?, field, &fields, &written))
            .collect();
        // A phase is scalar, and its own clock/rate/phase declarations must be
        // distinct Float fields. Captured state remains the initialized state.
        for phase in &phases {
            if phase.rate == layer.life {
                return None;
            }
            if [phase.phase, phase.rate, phase.clock]
                .iter()
                .any(|field| layer.fields[*field].value_type != Some(ValueType::Float))
            {
                return None;
            }
        }
        if phases.is_empty()
            && symbols
                .iter()
                .any(|node| matches!(node.symbol, Symbol::Time))
        {
            return None;
        }
        // Exact typed identity stores remain initialized values on every
        // invocation. Retain their output bindings to normalize inactive words;
        // arithmetic feedback and nontrivial word conversions stay unsupported.
        let invariant: Vec<_> = fields
            .iter()
            .enumerate()
            .map(|(field, value)| {
                value.is_some_and(|index| {
                    layer.fields[field].value_type.is_some_and(|kind| {
                    symbols[index].kind == kind
                        && matches!(symbols[index].symbol, Symbol::Field(source) if source == field)
                })
                })
            })
            .collect();
        let mut builder = Builder {
            symbols: &symbols,
            layer,
            written: &written,
            invariant: &invariant,
            phases: &phases,
            captures: Vec::new(),
            nodes: Vec::new(),
            memo: vec![None; symbols.len()],
        };
        let mut outputs = Vec::new();
        for (field, value) in fields
            .iter()
            .enumerate()
            .filter(|(field, _)| written[*field])
        {
            if outputs.len() >= MAX_OUTPUTS {
                return None;
            }
            let allow_time = phases.iter().any(|phase| phase.clock == field);
            outputs.push(Output {
                field,
                value: builder.lower((*value)?, allow_time)?,
            });
        }
        // An unused immutable sample still has checked failure semantics. Keep
        // its validation in the DAG instead of deleting a potentially failing
        // callback. Any error requires original scalar replay, which preserves
        // original first-error ordering and all earlier canonical stores.
        for (index, node) in symbols.iter().enumerate() {
            if matches!(node.symbol, Symbol::Curve { .. }) {
                builder.lower(index, false)?;
            }
        }
        if outputs.is_empty() {
            return None;
        }
        Some(Self {
            captures: builder.captures.into_boxed_slice(),
            nodes: builder.nodes.into_boxed_slice(),
            outputs: outputs.into_boxed_slice(),
            phases: phases.len(),
        })
    }
    #[cfg(test)]
    pub fn capture(
        &self,
        values: &mut [Value],
        mut read: impl FnMut(usize) -> Option<Value>,
    ) -> Result<(), AgeError> {
        if values.len() != self.captures.len() {
            return Err(AgeError {
                offset: None,
                message: "age capture storage does not match the plan",
            });
        }
        for (output, capture) in values.iter_mut().zip(&self.captures) {
            let value = read(capture.field).ok_or(AgeError {
                offset: None,
                message: "age birth field is absent",
            })?;
            if value.kind() != capture.kind {
                return Err(AgeError {
                    offset: None,
                    message: "age birth field has the wrong type",
                });
            }
            *output = value;
        }
        Ok(())
    }
    /// Evaluate the complete DAG once into reusable caller-owned storage. Output
    /// values are copied only after success; scalar replay owns error prefixes.
    pub fn evaluate(
        &self,
        captures: &[Value],
        samplers: &[EffectSampler],
        input: AgeSample,
        scratch: &mut [Value],
        outputs: &mut [Value],
    ) -> Result<(), AgeError> {
        if captures.len() != self.captures.len()
            || scratch.len() != self.nodes.len()
            || outputs.len() != self.outputs.len()
        {
            return Err(AgeError {
                offset: None,
                message: "age evaluation storage does not match the plan",
            });
        }
        for (index, node) in self.nodes.iter().enumerate() {
            let value = match node.expression {
                Expression::Constant(value) => value,
                Expression::Capture(capture) => captures[capture],
                Expression::Life => {
                    Value::from_words(ValueType::Float, [input.life.to_bits(), 0, 0, 0])
                }
                Expression::Time => {
                    let time = input.effect_time as f32;
                    if !time.is_finite() {
                        return Err(AgeError {
                            offset: Some(node.offset),
                            message: "effect callback produced a nonfinite scalar",
                        });
                    }
                    Value::from(time)
                }
                Expression::Words { input, kind } => {
                    Value::from_words(kind, scratch[input.0].words())
                }
                Expression::Binary {
                    operation,
                    left,
                    right,
                } => {
                    let kind = node.kind;
                    let left = scratch[left.0];
                    let right = scratch[right.0];
                    let a = left.words();
                    let b = right.words();
                    let mut words = [0; 4];
                    for (component, out) in words.iter_mut().enumerate().take(kind.components()) {
                        *out = float_arithmetic(
                            operation,
                            a[component.min(left.kind().components() - 1)],
                            b[component.min(right.kind().components() - 1)],
                        );
                    }
                    Value::from_words(kind, words)
                }
                Expression::Curve { sampler, input } => sample(
                    samplers.get(sampler).ok_or(AgeError {
                        offset: Some(node.offset),
                        message: "age curve resource is absent",
                    })?,
                    f32::from_bits(scratch[input.0].words()[0]),
                )
                .map_err(|message| AgeError {
                    offset: Some(node.offset),
                    message,
                })?,
            };
            scratch[index] = value;
        }
        for (output, binding) in outputs.iter_mut().zip(&self.outputs) {
            *output = scratch[binding.value.0];
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "age_tests.rs"]
mod tests;

impl Display for AgeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(offset) = self.offset {
            write!(f, "{} at byte {offset}", self.message)
        } else {
            f.write_str(self.message)
        }
    }
}
