//! Optional closed age expressions derived from the checked selected evolution.
//! This module performs no admission, scheduling, canonical writes or GPU work.
use super::curve::sample;
use super::execution::float_arithmetic;
use super::graph::{Callback, EffectLayer, EffectSampler, External};
use super::{Address, BinaryOperation, Destination, Instruction, Source, Value, ValueType};
use super::{
    SpecializationInstruction, SpecializationReason as Reason,
    SpecializationRejection as Rejection, SpecializationResource as Resource,
};
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    operation: BinaryOperation,
}
struct Builder<'a> {
    symbols: &'a [SymbolNode],
    layer: &'a EffectLayer,
    written: &'a [bool],
    invariant: &'a [bool],
    phases: &'a [Phase],
    captures: Vec<Capture>,
    nodes: Vec<Node>,
    node_requests: usize,
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
) -> Result<usize, Rejection> {
    if !supported(kind) {
        return Err(Rejection::new(Reason::Layout(kind)).at(offset));
    }
    if symbols.len() >= MAX_NODES {
        return Err(Rejection::new(Reason::Limit {
            resource: Resource::Nodes,
            limit: MAX_NODES,
        })
        .at(offset));
    }
    let result = symbols.len();
    symbols.push(SymbolNode {
        symbol,
        kind,
        offset,
    });
    Ok(result)
}
fn source(
    symbols: &mut Vec<SymbolNode>,
    registers: &[Option<usize>],
    counts: [usize; 4],
    source: Source,
    offset: usize,
) -> Result<usize, Rejection> {
    if !supported(source.kind) {
        return Err(Rejection::new(Reason::Layout(source.kind)).at(offset));
    }
    match source.address {
        Address::Constant(words) => push(
            symbols,
            Symbol::Constant(Value::from_words(source.kind, words)),
            source.kind,
            offset,
        ),
        Address::Slot { scope, slot } => {
            let ordinal = register(scope, slot, counts)
                .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?;
            let index = registers[ordinal].ok_or_else(|| {
                Rejection::new(Reason::PersistentInput { scope, slot }).at(offset)
            })?;
            if symbols[index].kind == source.kind {
                Ok(index)
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
    offset: usize,
) -> Result<(), Rejection> {
    if !supported(destination.kind) {
        return Err(Rejection::new(Reason::Layout(destination.kind)).at(offset));
    }
    let slot = register(destination.scope, destination.slot, counts)
        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))?;
    *registers
        .get_mut(slot)
        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(offset))? = Some(value);
    Ok(())
}
fn old_field(symbols: &[SymbolNode], index: usize) -> Option<usize> {
    if let Symbol::Field(field) = symbols[index].symbol {
        Some(field)
    } else {
        None
    }
}
fn arithmetic(kind: ValueType) -> bool {
    matches!(
        kind,
        ValueType::Float | ValueType::Float3 | ValueType::Float4
    )
}
fn arithmetic_value(
    kind: ValueType,
    operation: BinaryOperation,
    left: Value,
    right: Value,
) -> Value {
    let mut words = [0; 4];
    for (component, out) in words.iter_mut().enumerate().take(kind.components()) {
        *out = float_arithmetic(
            operation,
            left.words()[component.min(left.kind().components() - 1)],
            right.words()[component.min(right.kind().components() - 1)],
        );
    }
    Value::from_words(kind, words)
}
fn invariant_rates(symbols: &[SymbolNode], written: &[bool], life: usize) -> Vec<bool> {
    // Symbols are topological. Compute once, avoiding exponential walks when
    // several operations reuse the same immutable expression.
    let mut rates = Vec::with_capacity(symbols.len());
    for node in symbols {
        let invariant = arithmetic(node.kind)
            && match node.symbol {
                Symbol::Constant(_) => true,
                Symbol::Field(field) => field != life && !written[field],
                Symbol::Words { input, .. } => rates[input],
                Symbol::Binary { left, right, .. } => rates[left] && rates[right],
                Symbol::Time | Symbol::Curve { .. } => false,
            };
        rates.push(invariant);
    }
    rates
}
fn phase(
    symbols: &[SymbolNode],
    expression: usize,
    field: usize,
    final_fields: &[Option<usize>],
    invariant_rates: &[bool],
) -> Option<Phase> {
    // An initialized affine accumulator and a paired last-time update. The
    // rate may be a pure initialized expression, never lifecycle or feedback.
    let Symbol::Binary {
        operation,
        left,
        right,
    } = symbols[expression].symbol
    else {
        return None;
    };
    if !matches!(operation, BinaryOperation::Add | BinaryOperation::Subtract) {
        return None;
    }
    let candidates = [(left, right), (right, left)];
    let candidates = if operation == BinaryOperation::Add {
        &candidates[..]
    } else {
        &candidates[..1]
    };
    for &(previous, increment) in candidates {
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
        // Reusing the scalar delta is valid only when multiplication has the
        // same layout as the final accumulator (no intermediate truncation).
        if symbols[increment].kind != symbols[expression].kind {
            continue;
        }
        for (rate, delta) in [(left, right), (right, left)] {
            if !invariant_rates[rate] {
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
            if symbols[delta].kind != ValueType::Float
                || !matches!(symbols[left].symbol, Symbol::Time)
            {
                continue;
            }
            let Some(clock) = old_field(symbols, right) else {
                continue;
            };
            if clock == field {
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
                    operation,
                });
            }
        }
    }
    None
}
impl Builder<'_> {
    fn node(
        &mut self,
        expression: Expression,
        kind: ValueType,
        offset: usize,
    ) -> Result<NodeId, Rejection> {
        // Preserve the original construction budget even when several source
        // expressions share one value node after normalization.
        if self.node_requests >= MAX_NODES {
            return Err(Rejection::new(Reason::Limit {
                resource: Resource::Nodes,
                limit: MAX_NODES,
            })
            .at(offset));
        }
        self.node_requests += 1;
        let expression = self.fold(expression, kind);
        // A curve call remains a distinct checked guard at its source offset,
        // including calls whose results are not consumed by an output.
        if !matches!(expression, Expression::Curve { .. }) {
            if let Some((index, node)) = self
                .nodes
                .iter_mut()
                .enumerate()
                .find(|(_, node)| node.kind == kind && node.expression == expression)
            {
                node.offset = node.offset.min(offset);
                return Ok(NodeId(index));
            }
        }
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            expression,
            kind,
            offset,
        });
        Ok(id)
    }
    fn fold(&self, expression: Expression, kind: ValueType) -> Expression {
        let finite = |value: Value| {
            !arithmetic(value.kind())
                || value.words()[..value.kind().components()]
                    .iter()
                    .all(|word| f32::from_bits(*word).is_finite())
        };
        let value = match expression {
            Expression::Words { input, kind } => {
                let Expression::Constant(value) = self.nodes[input.0].expression else {
                    return expression;
                };
                if !finite(value) {
                    return expression;
                }
                Value::from_words(kind, value.words())
            }
            Expression::Binary {
                operation,
                left,
                right,
            } => {
                let (Expression::Constant(left), Expression::Constant(right)) = (
                    self.nodes[left.0].expression,
                    self.nodes[right.0].expression,
                ) else {
                    return expression;
                };
                if !finite(left) || !finite(right) {
                    return expression;
                }
                arithmetic_value(kind, operation, left, right)
            }
            _ => return expression,
        };
        // Do not erase nonfinite intermediate nodes observed by the cold
        // capture guard. No algebraic identities, reassociation or DCE apply.
        if finite(value) {
            Expression::Constant(value)
        } else {
            expression
        }
    }
    fn capture(&mut self, field: usize, offset: usize) -> Result<NodeId, Rejection> {
        let kind = self
            .layer
            .fields
            .get(field)
            .and_then(|field| field.value_type)
            .ok_or_else(|| {
                Rejection::new(Reason::InvalidBinding)
                    .at(offset)
                    .field(field)
            })?;
        if !supported(kind) {
            return Err(Rejection::new(Reason::Layout(kind)).at(offset).field(field));
        }
        let index = if let Some(index) = self
            .captures
            .iter()
            .position(|capture| capture.field == field)
        {
            index
        } else {
            if self.captures.len() >= MAX_CAPTURES {
                return Err(Rejection::new(Reason::Limit {
                    resource: Resource::Captures,
                    limit: MAX_CAPTURES,
                })
                .at(offset)
                .field(field));
            }
            let index = self.captures.len();
            self.captures.push(Capture { field, kind });
            index
        };
        self.node(Expression::Capture(index), kind, offset)
    }
    fn lower(&mut self, index: usize, allow_time: bool) -> Result<NodeId, Rejection> {
        // Time is permitted only inside the certified phase and as its paired
        // clock store. Never memoize an otherwise disallowed Time use.
        let input = self.symbols[index];
        if matches!(input.symbol, Symbol::Time) {
            if !allow_time {
                return Err(Rejection::new(Reason::UncertifiedTime).at(input.offset));
            }
            return self.node(Expression::Time, ValueType::Float, input.offset);
        }
        if let Some(result) = self.memo[index] {
            return Ok(result);
        }
        if let Some(phase) = self
            .phases
            .iter()
            .find(|phase| phase.expression == index)
            .copied()
        {
            let initial = self.capture(phase.phase, input.offset)?;
            let rate = self.lower(phase.rate, false)?;
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
                input.kind,
                input.offset,
            )?;
            let result = self.node(
                Expression::Binary {
                    operation: phase.operation,
                    left: initial,
                    right: increment,
                },
                input.kind,
                input.offset,
            )?;
            self.memo[index] = Some(result);
            return Ok(result);
        }
        let expression = match input.symbol {
            Symbol::Constant(value) => Expression::Constant(value),
            Symbol::Field(field) if field == self.layer.life => Expression::Life,
            Symbol::Field(field) if !self.written[field] || self.invariant[field] => {
                let result = self.capture(field, input.offset)?;
                self.memo[index] = Some(result);
                return Ok(result);
            }
            Symbol::Field(field) => {
                return Err(Rejection::new(Reason::Feedback)
                    .at(input.offset)
                    .field(field));
            }
            Symbol::Time => return Err(Rejection::new(Reason::UncertifiedTime).at(input.offset)),
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
        Ok(result)
    }
}
impl AgePlan {
    /// Keep unused samples in the same finite-domain certificate as outputs.
    fn certify_curves(&self, samplers: &[EffectSampler]) -> Result<(), Rejection> {
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
        for node in &self.nodes {
            if let Expression::Curve { sampler, input } = node.expression {
                if !life(self, input) {
                    return Err(Rejection::new(Reason::CurveCoordinate { sampler }).at(node.offset));
                }
                if !matches!(samplers.get(sampler), Some(EffectSampler::Curve { times, values, tangents, .. })
                    if !times.is_empty() && values.len() == times.len()
                    && (tangents.is_empty() || tangents.len() == 2 * times.len())
                    && times.iter().all(|t| t.is_finite())
                    && times.windows(2).all(|t| t[0] < t[1]))
                {
                    return Err(Rejection::new(Reason::CurveResource { sampler }).at(node.offset));
                }
            }
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn total_life_curves(&self, samplers: &[EffectSampler]) -> bool {
        self.certify_curves(samplers).is_ok()
    }
    pub fn analyze(layer: &EffectLayer, samplers: &[EffectSampler]) -> Result<Self, Rejection> {
        let plan = Self::build(layer, samplers)?;
        plan.certify_curves(samplers)?;
        Ok(plan)
    }
    #[cfg(test)]
    pub fn prepare(layer: &EffectLayer, samplers: &[EffectSampler]) -> Option<Self> {
        // Private evaluator tests also exercise deliberately uncertified curves.
        Self::build(layer, samplers).ok()
    }
    /// Validate initialized arithmetic once, before captures become immutable.
    /// Finite fields alone do not prevent an overflowing derived rate. Dynamic
    /// life/time expressions and checked curves retain their ordinary guards.
    pub fn captures_permitted(&self, mut read: impl FnMut(usize) -> Value) -> bool {
        let mut values = [Value::from_words(ValueType::Float, [0; 4]); MAX_NODES];
        let mut initialized = [false; MAX_NODES];
        for (index, node) in self.nodes.iter().enumerate() {
            let value = match node.expression {
                Expression::Constant(value) => value,
                Expression::Capture(capture) => read(self.captures[capture].field),
                Expression::Words { input, kind } if initialized[input.0] => {
                    Value::from_words(kind, values[input.0].words())
                }
                Expression::Binary {
                    operation,
                    left,
                    right,
                } if initialized[left.0] && initialized[right.0] => {
                    let left = values[left.0];
                    let right = values[right.0];
                    arithmetic_value(node.kind, operation, left, right)
                }
                _ => continue,
            };
            if value.kind() != node.kind
                || (arithmetic(node.kind)
                    && value.words()[..node.kind.components()]
                        .iter()
                        .any(|word| !f32::from_bits(*word).is_finite()))
            {
                return false;
            }
            values[index] = value;
            initialized[index] = true;
        }
        true
    }
    pub fn permitted(&self, sample: AgeSample) -> bool {
        // The renderer's analytic life also uses an f32 time anchor, even when
        // the extracted source contains no explicit Time node.
        sample.life.is_finite() && (sample.effect_time as f32).is_finite()
    }
    fn build(layer: &EffectLayer, samplers: &[EffectSampler]) -> Result<Self, Rejection> {
        if layer.spawner {
            return Err(Rejection::new(Reason::Spawner));
        }
        if !layer.rendered {
            return Err(Rejection::new(Reason::NotRendered));
        }
        if layer.evolve_on_death {
            return Err(Rejection::new(Reason::EvolutionOnDeath));
        }
        let bound = layer
            .evolve
            .as_ref()
            .ok_or_else(|| Rejection::new(Reason::InvalidBinding))?;
        let counts = bound.program.counts;
        let address_limit = || {
            Rejection::new(Reason::Limit {
                resource: Resource::RegisterAddress,
                limit: isize::MAX as usize,
            })
        };
        let total = counts
            .iter()
            .try_fold(0usize, |a, b| a.checked_add(*b))
            .ok_or_else(address_limit)?;
        let locals = counts[0].checked_add(counts[1]).ok_or_else(address_limit)?;
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
                    let value = match bound
                        .externals
                        .get(usize::from(*external))
                        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(op.offset))?
                    {
                        External::Field(field) => {
                            if layer
                                .fields
                                .get(*field)
                                .ok_or_else(|| {
                                    Rejection::new(Reason::InvalidBinding)
                                        .at(op.offset)
                                        .field(*field)
                                })?
                                .value_type
                                != Some(destination.kind)
                            {
                                return Err(Rejection::new(Reason::InvalidBinding)
                                    .at(op.offset)
                                    .field(*field));
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
                        _ => return Err(Rejection::new(Reason::External(*external)).at(op.offset)),
                    };
                    set(&mut registers, counts, *destination, value, op.offset)?;
                }
                Instruction::Store {
                    source: input,
                    external,
                } => {
                    let External::Field(field) = bound
                        .externals
                        .get(usize::from(*external))
                        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(op.offset))?
                    else {
                        return Err(Rejection::new(Reason::External(*external)).at(op.offset));
                    };
                    if *field == layer.life || *field == layer.inverse_life {
                        return Err(Rejection::new(Reason::LifecycleWrite)
                            .at(op.offset)
                            .field(*field));
                    }
                    if layer
                        .fields
                        .get(*field)
                        .ok_or_else(|| {
                            Rejection::new(Reason::InvalidBinding)
                                .at(op.offset)
                                .field(*field)
                        })?
                        .value_type
                        != Some(input.kind)
                    {
                        return Err(Rejection::new(Reason::InvalidBinding)
                            .at(op.offset)
                            .field(*field));
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
                    ) {
                        return Err(Rejection::new(Reason::Instruction(
                            SpecializationInstruction::Arithmetic,
                        ))
                        .at(op.offset));
                    }
                    for kind in [destination.kind, left.kind, right.kind] {
                        if !arithmetic(kind) {
                            return Err(Rejection::new(Reason::Layout(kind)).at(op.offset));
                        }
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
                    set(&mut registers, counts, *destination, value, op.offset)?;
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
                        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(op.offset))?;
                    if callback != &Callback::Sample {
                        return Err(Rejection::new(Reason::Instruction(
                            SpecializationInstruction::Call,
                        ))
                        .at(op.offset));
                    }
                    let External::Sampler(sampler) = bound
                        .externals
                        .get(usize::from(*receiver))
                        .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(op.offset))?
                    else {
                        return Err(Rejection::new(Reason::External(*receiver)).at(op.offset));
                    };
                    if !matches!(
                        samplers
                            .get(*sampler)
                            .ok_or_else(|| Rejection::new(Reason::InvalidBinding).at(op.offset))?,
                        EffectSampler::Curve {
                            kind: ValueType::Float4,
                            ..
                        }
                    ) {
                        return Err(Rejection::new(Reason::CurveResource { sampler: *sampler })
                            .at(op.offset));
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
                    set(&mut registers, counts, *destination, value, op.offset)?;
                }
                instruction => {
                    return Err(
                        Rejection::new(Reason::Instruction(instruction.into())).at(op.offset)
                    )
                }
            }
        }
        let invariant_rates = invariant_rates(&symbols, &written, layer.life);
        let phases: Vec<_> = fields
            .iter()
            .enumerate()
            .filter(|(field, _)| written[*field])
            .filter_map(|(field, value)| {
                phase(&symbols, (*value)?, field, &fields, &invariant_rates)
            })
            .collect();
        // Clocks are scalar, while an accumulator may use the same checked
        // componentwise Float3/Float4 arithmetic as the source evaluator.
        for phase in &phases {
            if layer.fields[phase.clock].value_type != Some(ValueType::Float)
                || !layer.fields[phase.phase].value_type.is_some_and(arithmetic)
            {
                return Err(Rejection::new(Reason::InvalidBinding)
                    .at(symbols[phase.expression].offset)
                    .field(phase.phase));
            }
        }
        if phases.is_empty()
            && symbols
                .iter()
                .any(|node| matches!(node.symbol, Symbol::Time))
        {
            let offset = symbols
                .iter()
                .find(|node| matches!(node.symbol, Symbol::Time))
                .expect("time reference was detected")
                .offset;
            return Err(Rejection::new(Reason::UncertifiedTime).at(offset));
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
            node_requests: 0,
            memo: vec![None; symbols.len()],
        };
        let mut outputs = Vec::new();
        for (field, value) in fields
            .iter()
            .enumerate()
            .filter(|(field, _)| written[*field])
        {
            if outputs.len() >= MAX_OUTPUTS {
                return Err(Rejection::new(Reason::Limit {
                    resource: Resource::Outputs,
                    limit: MAX_OUTPUTS,
                })
                .field(field));
            }
            let allow_time = phases.iter().any(|phase| phase.clock == field);
            outputs.push(Output {
                field,
                value: builder.lower(
                    (*value).ok_or_else(|| Rejection::new(Reason::InvalidBinding).field(field))?,
                    allow_time,
                )?,
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
            return Err(Rejection::new(Reason::NoOutputs));
        }
        Ok(Self {
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
                } => arithmetic_value(node.kind, operation, scratch[left.0], scratch[right.0]),
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

#[cfg(test)]
#[path = "age_normalization_tests.rs"]
mod normalization_tests;

impl Display for AgeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(offset) = self.offset {
            write!(f, "{} at byte {offset}", self.message)
        } else {
            f.write_str(self.message)
        }
    }
}
