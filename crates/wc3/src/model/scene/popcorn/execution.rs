use super::{
    Address, BinaryOperation, Destination, Family, Instruction, Program, Source, UnaryOperation,
    Value,
};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::mem::swap;

/// Resource and side-effect boundary for one particle invocation.
///
/// Bindings are resolved before execution. A host may request retirement or
/// enqueue events without interrupting the remaining instruction stream.
pub trait Host {
    /// Application-specific binding or resource failure.
    type Error;
    /// Read a typed field or opaque capability by its compiled external slot.
    fn load(&mut self, external: u16) -> Result<Value, Self::Error>;
    /// Write a field by its compiled external slot.
    fn store(&mut self, external: u16, value: Value) -> Result<(), Self::Error>;
    /// Invoke a resolved function with read arguments and preserved native controls.
    fn call(
        &mut self,
        function: u16,
        receiver: Option<u16>,
        arguments: &[Value],
        controls: &[u8],
        flags: u8,
    ) -> Result<Option<Value>, Self::Error>;
}

/// Reusable storage for one particle, including allocation-free call scratch.
#[derive(Clone, Debug)]
pub struct State {
    scopes: [Vec<[u32; 4]>; 4],
    arguments: Vec<Value>,
}
impl State {
    /// Allocate the maximum storage required by a layer's alternative programmes.
    pub fn for_programs<'a>(programs: impl IntoIterator<Item = &'a Program>) -> Self {
        Self::allocate(programs, [true; 4], true)
    }
    fn allocate<'a>(
        programs: impl IntoIterator<Item = &'a Program>,
        enabled: [bool; 4],
        calls: bool,
    ) -> Self {
        let mut counts = [0; 4];
        let mut arguments = 0;
        for program in programs {
            for ((count, required), enabled) in counts.iter_mut().zip(program.counts).zip(enabled) {
                if enabled {
                    *count = (*count).max(required);
                }
            }
            if calls {
                arguments = arguments.max(program.max_arguments);
            }
        }
        Self {
            scopes: counts.map(|count| vec![[0; 4]; count]),
            arguments: Vec::with_capacity(arguments),
        }
    }
    pub(crate) fn persistent_for_programs<'a>(
        programs: impl IntoIterator<Item = &'a Program>,
    ) -> Self {
        Self::allocate(programs, [false, false, true, true], false)
    }
    pub(crate) fn scratch_for_programs<'a>(
        programs: impl IntoIterator<Item = &'a Program>,
    ) -> Self {
        Self::allocate(programs, [true, true, false, false], true)
    }
    /// Clear every persistent value when reusing storage for a new particle.
    pub fn clear(&mut self) {
        for scope in &mut self.scopes {
            scope.fill([0; 4]);
        }
        self.arguments.clear();
    }
    #[cfg(test)]
    pub(super) fn same_observable(
        &self,
        other: &Self,
        dead: impl Fn(usize, usize) -> bool,
    ) -> bool {
        self.arguments == other.arguments
            && self
                .scopes
                .iter()
                .zip(&other.scopes)
                .enumerate()
                .all(|(scope, (a, b))| {
                    a.len() == b.len()
                        && a.iter()
                            .zip(b)
                            .enumerate()
                            .all(|(slot, (a, b))| dead(scope, slot) || a == b)
                })
    }
    fn read(&self, source: Source) -> Value {
        let words = match source.address {
            Address::Constant(words) => words,
            Address::Slot { scope, slot } => self.scopes[scope][slot],
        };
        Value::from_words(source.kind, words)
    }
    fn write(&mut self, destination: Destination, words: [u32; 4]) {
        self.scopes[destination.scope][destination.slot] =
            Value::from_words(destination.kind, words).words();
    }
}

/// Execution failure with the instruction's original byte location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionError<E> {
    /// Byte position within the compiled instruction stream.
    pub offset: usize,
    /// Runtime failure at this instruction.
    pub kind: Failure<E>,
}
/// Runtime failures kept distinct from host errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure<E> {
    /// State was not allocated for this programme or its layer.
    IncompatibleState,
    /// Host returned a value with an incompatible layout or missing return.
    HostType,
    /// Integer division by zero, whose native behavior is unspecified.
    IntegerDivisionByZero,
    /// Host binding or resource failure.
    Host(E),
}
impl<E: Display> Display for ExecutionError<E> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "particle programme at byte {}: ", self.offset)?;
        match &self.kind {
            Failure::IncompatibleState => formatter.write_str("incompatible particle storage"),
            Failure::HostType => {
                formatter.write_str("host value does not match the compiled signature")
            }
            Failure::IntegerDivisionByZero => formatter.write_str("integer division by zero"),
            Failure::Host(error) => Display::fmt(error, formatter),
        }
    }
}
impl<E: Error + 'static> Error for ExecutionError<E> {}

impl Program {
    /// Execute once, resetting local scratch while preserving input and stream storage.
    /// Errors leave earlier host writes and particle-state changes observable.
    pub fn execute<H: Host>(
        &self,
        state: &mut State,
        host: &mut H,
    ) -> Result<(), ExecutionError<H::Error>> {
        if state.arguments.capacity() < self.max_arguments
            || state
                .scopes
                .iter()
                .zip(self.counts)
                .any(|(storage, required)| storage.len() < required)
        {
            return Err(ExecutionError {
                offset: 0,
                kind: Failure::IncompatibleState,
            });
        }
        state.scopes[0][..self.counts[0]].fill([0; 4]);
        state.scopes[1][..self.counts[1]].fill([0; 4]);
        for operation in &self.operations {
            self.execute_instruction(&operation.instruction, state, host)
                .map_err(|kind| ExecutionError {
                    offset: operation.offset,
                    kind,
                })?;
        }
        Ok(())
    }
    pub(crate) fn execute_with_scratch<H: Host>(
        &self,
        state: &mut State,
        scratch: &mut State,
        host: &mut H,
    ) -> Result<(), ExecutionError<H::Error>> {
        for scope in 0..2 {
            swap(&mut state.scopes[scope], &mut scratch.scopes[scope]);
        }
        swap(&mut state.arguments, &mut scratch.arguments);
        let result = self.execute(state, host);
        for scope in 0..2 {
            swap(&mut state.scopes[scope], &mut scratch.scopes[scope]);
        }
        swap(&mut state.arguments, &mut scratch.arguments);
        result
    }
    fn execute_instruction<H: Host>(
        &self,
        instruction: &Instruction,
        state: &mut State,
        host: &mut H,
    ) -> Result<(), Failure<H::Error>> {
        match instruction {
            Instruction::Load {
                destination,
                external,
            } => {
                let value = host.load(*external).map_err(Failure::Host)?;
                if value.kind() != destination.kind {
                    return Err(Failure::HostType);
                }
                state.write(*destination, value.words());
            }
            Instruction::Store { source, external } => host
                .store(*external, state.read(*source))
                .map_err(Failure::Host)?,
            Instruction::Reinterpret {
                destination,
                source,
            } => state.write(*destination, state.read(*source).words()),
            Instruction::Convert {
                destination,
                source,
            } => {
                let value = state.read(*source);
                let mut words = [0; 4];
                for (index, word) in words
                    .iter_mut()
                    .enumerate()
                    .take(destination.kind.components())
                {
                    let input = value.words()[index.min(source.kind.components() - 1)];
                    *word = convert(input, source.kind.family(), destination.kind.family());
                }
                state.write(*destination, words);
            }
            Instruction::Construct {
                destination,
                sources,
            } => {
                let mut words = [0; 4];
                let mut next = 0;
                for source in sources {
                    let value = state.read(*source);
                    for component in &value.words()[..source.kind.components()] {
                        words[next] = *component;
                        next += 1;
                    }
                }
                if next == 1 {
                    let scalar = words[0];
                    words.fill(scalar);
                }
                state.write(*destination, words);
            }
            Instruction::Swizzle {
                destination,
                source,
                lanes,
            } => {
                let input = state.read(*source).words();
                let mut words = [0; 4];
                for index in 0..destination.kind.components() {
                    words[index] = match lanes[index] {
                        0..=3 => input[usize::from(lanes[index])],
                        4 => 0,
                        5 => one(destination.kind.family()),
                        _ => unreachable!(),
                    };
                }
                state.write(*destination, words);
            }
            Instruction::Binary {
                destination,
                left,
                right,
                operation,
            } => {
                let left_words = state.read(*left).words();
                let right_words = state.read(*right).words();
                let mut words = [0; 4];
                if *operation == BinaryOperation::Dot {
                    words[0] = (0..left.kind.components())
                        .map(|index| {
                            f32::from_bits(left_words[index]) * f32::from_bits(right_words[index])
                        })
                        .sum::<f32>()
                        .to_bits();
                } else {
                    for (index, word) in words
                        .iter_mut()
                        .enumerate()
                        .take(destination.kind.components())
                    {
                        let lhs = left_words[index.min(left.kind.components() - 1)];
                        let rhs = right_words[index.min(right.kind.components() - 1)];
                        *word = binary(*operation, lhs, rhs, left.kind.family())
                            .ok_or(Failure::IntegerDivisionByZero)?;
                    }
                }
                state.write(*destination, words);
            }
            Instruction::Unary {
                destination,
                source,
                operation,
            } => {
                let input = state.read(*source).words();
                let mut words = [0; 4];
                if *operation == UnaryOperation::SineCosine {
                    let (sin, cos) = f32::from_bits(input[0]).sin_cos();
                    words[0] = sin.to_bits();
                    words[1] = cos.to_bits();
                } else {
                    for (index, word) in words
                        .iter_mut()
                        .enumerate()
                        .take(destination.kind.components())
                    {
                        let input = input[index.min(source.kind.components() - 1)];
                        *word = unary(*operation, input, source.kind.family());
                    }
                }
                state.write(*destination, words);
            }
            Instruction::Select {
                destination,
                if_false,
                if_true,
                condition,
            } => {
                let conditions = state.read(*condition).words();
                let a = state.read(*if_false).words();
                let b = state.read(*if_true).words();
                let mut words = [0; 4];
                for index in 0..destination.kind.components() {
                    words[index] = if conditions[index.min(condition.kind.components() - 1)] != 0 {
                        b[index]
                    } else {
                        a[index]
                    };
                }
                state.write(*destination, words);
            }
            Instruction::Call {
                destination,
                result_kind,
                receiver,
                function,
                arguments,
                controls,
                flags,
            } => {
                state.arguments.clear();
                for source in arguments {
                    let value = state.read(*source);
                    state.arguments.push(value);
                }
                let result = host
                    .call(*function, *receiver, &state.arguments, controls, *flags)
                    .map_err(Failure::Host)?;
                if result.map(Value::kind) != *result_kind {
                    return Err(Failure::HostType);
                }
                if let (Some(destination), Some(value)) = (destination, result) {
                    state.write(*destination, value.words());
                }
            }
        }
        Ok(())
    }
}

fn truth(value: bool) -> u32 {
    if value {
        u32::MAX
    } else {
        0
    }
}
fn one(family: Family) -> u32 {
    match family {
        Family::Float => 1.0f32.to_bits(),
        Family::Bool => u32::MAX,
        _ => 1,
    }
}
fn convert(word: u32, from: Family, to: Family) -> u32 {
    if from == to {
        return word;
    }
    match (from, to) {
        (Family::Float, Family::Int) => (f32::from_bits(word) as i32) as u32,
        (Family::Float, Family::Bool) => truth(f32::from_bits(word) != 0.0),
        (Family::Int, Family::Float) => (word as i32 as f32).to_bits(),
        (Family::Int, Family::Bool) => truth(word != 0),
        (Family::Bool, Family::Float) => (if word != 0 { 1.0f32 } else { 0.0 }).to_bits(),
        (Family::Bool, Family::Int) => u32::from(word != 0),
        _ => unreachable!(),
    }
}
/// Exact scalar word arithmetic shared with the private pure-evolution plan.
#[inline]
pub(super) fn float_arithmetic(operation: BinaryOperation, left: u32, right: u32) -> u32 {
    let a = f32::from_bits(left);
    let b = f32::from_bits(right);
    match operation {
        BinaryOperation::Add => (a + b).to_bits(),
        BinaryOperation::Subtract => (a - b).to_bits(),
        BinaryOperation::Multiply => (a * b).to_bits(),
        _ => unreachable!(),
    }
}
fn unary(operation: UnaryOperation, word: u32, family: Family) -> u32 {
    let value = f32::from_bits(word);
    match operation {
        UnaryOperation::Negate if family == Family::Float => (-value).to_bits(),
        UnaryOperation::Negate => (word as i32).wrapping_neg() as u32,
        UnaryOperation::LogicalNot => truth(word == 0),
        UnaryOperation::BitNot => !word,
        UnaryOperation::SquareRoot => value.sqrt().to_bits(),
        UnaryOperation::ReciprocalSquareRoot => value.sqrt().recip().to_bits(),
        UnaryOperation::Exponential => value.exp().to_bits(),
        UnaryOperation::Reciprocal => value.recip().to_bits(),
        UnaryOperation::SignedFraction => value.fract().to_bits(),
        UnaryOperation::Saturate => value.clamp(0.0, 1.0).to_bits(),
        UnaryOperation::IsFinite => truth(value.is_finite()),
        UnaryOperation::SineCosine => unreachable!("pair result is handled by the instruction"),
    }
}
fn binary(operation: BinaryOperation, left: u32, right: u32, family: Family) -> Option<u32> {
    let result = if family == Family::Float {
        let a = f32::from_bits(left);
        let b = f32::from_bits(right);
        match operation {
            BinaryOperation::Add | BinaryOperation::Subtract | BinaryOperation::Multiply => {
                float_arithmetic(operation, left, right)
            }
            BinaryOperation::Divide => (a / b).to_bits(),
            BinaryOperation::Remainder => (a % b).to_bits(),
            BinaryOperation::Less => truth(a < b),
            BinaryOperation::LessEqual => truth(a <= b),
            BinaryOperation::Greater => truth(a > b),
            BinaryOperation::GreaterEqual => truth(a >= b),
            BinaryOperation::Equal => truth(a == b),
            BinaryOperation::Minimum => a.min(b).to_bits(),
            BinaryOperation::Maximum => a.max(b).to_bits(),
            _ => unreachable!(),
        }
    } else if family == Family::Bool {
        match operation {
            BinaryOperation::LogicalAnd => truth(left != 0 && right != 0),
            BinaryOperation::LogicalOr => truth(left != 0 || right != 0),
            BinaryOperation::Equal => truth((left != 0) == (right != 0)),
            _ => unreachable!(),
        }
    } else {
        let a = left as i32;
        let b = right as i32;
        match operation {
            BinaryOperation::Add => a.wrapping_add(b) as u32,
            BinaryOperation::Subtract => a.wrapping_sub(b) as u32,
            BinaryOperation::Multiply => a.wrapping_mul(b) as u32,
            BinaryOperation::Divide if b != 0 => a.wrapping_div(b) as u32,
            BinaryOperation::Remainder if b != 0 => a.wrapping_rem(b) as u32,
            BinaryOperation::Divide | BinaryOperation::Remainder => return None,
            BinaryOperation::BitAnd => left & right,
            BinaryOperation::BitOr => left | right,
            BinaryOperation::Less => truth(a < b),
            BinaryOperation::LessEqual => truth(a <= b),
            BinaryOperation::Greater => truth(a > b),
            BinaryOperation::GreaterEqual => truth(a >= b),
            BinaryOperation::Equal => truth(a == b),
            _ => unreachable!(),
        }
    };
    Some(result)
}

#[cfg(test)]
#[path = "execution_tests.rs"]
mod tests;
