use super::{Instruction as NativeInstruction, Register, Script};
use crate::model::scene::popcorn::{
    Address, Destination, Family, Function, Instruction, Operation, Program, Source, ValueType,
};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Allocation limits applied before compiling untrusted particle programmes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompileLimits {
    /// Total slots across the four storage scopes.
    pub storage_slots: usize,
    /// Maximum decoded instruction count.
    pub instructions: usize,
}
impl Default for CompileLimits {
    fn default() -> Self {
        Self {
            storage_slots: 65_536,
            instructions: 1_000_000,
        }
    }
}
/// Unsupported or invalid instruction with its native byte location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileError {
    /// Byte offset within the declared instruction stream.
    pub offset: usize,
    /// Reason compilation cannot establish a checked execution contract.
    pub message: &'static str,
}
impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "PKB instruction at byte {}: {}",
            self.offset, self.message
        )
    }
}
impl Error for CompileError {}

struct Compiler<'s, 'a> {
    script: &'s Script<'a>,
    counts: [usize; 4],
    externals: &'s [ValueType],
    functions: &'s [Function],
    offset: usize,
}
impl Script<'_> {
    /// Resolve register storage, layouts and host signatures before simulation.
    /// External and function arrays use this script's declaration-slot order.
    pub fn compile(
        &self,
        externals: &[ValueType],
        functions: &[Function],
    ) -> Result<Program, CompileError> {
        self.compile_with_limits(externals, functions, CompileLimits::default())
    }
    /// Compile with explicit limits for the application's trusted asset sizes.
    pub fn compile_with_limits(
        &self,
        externals: &[ValueType],
        functions: &[Function],
        limits: CompileLimits,
    ) -> Result<Program, CompileError> {
        let counts: [u32; 4] = self.register_counts[1..].try_into().unwrap();
        let counts: [usize; 4] = counts.map(|count: u32| count as usize);
        let mut compiler = Compiler {
            script: self,
            counts,
            externals,
            functions,
            offset: 0,
        };
        let total = counts
            .into_iter()
            .try_fold(0usize, |sum, count| sum.checked_add(count));
        if total.is_none_or(|total| total > limits.storage_slots) {
            return Err(compiler.error("register allocation exceeds the configured storage limit"));
        }
        if self.instructions.len() > limits.instructions {
            return Err(compiler.error("instruction count exceeds the configured limit"));
        }
        let mut operations = Vec::with_capacity(self.instructions.len());
        for located in &self.instructions {
            compiler.offset = located.offset;
            operations.push(Operation {
                offset: located.offset,
                instruction: compiler.instruction(&located.instruction)?,
            });
        }
        let max_arguments = operations
            .iter()
            .filter_map(|operation| match &operation.instruction {
                Instruction::Call { arguments, .. } => Some(arguments.len()),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        Ok(Program {
            counts,
            max_arguments,
            operations,
        })
    }
}
impl Compiler<'_, '_> {
    fn error(&self, message: &'static str) -> CompileError {
        CompileError {
            offset: self.offset,
            message,
        }
    }
    fn kind(&self, register: Register) -> Result<ValueType, CompileError> {
        Ok(match register.bank() {
            0x00 => ValueType::Context,
            0x02 => ValueType::Bool,
            0x04 => ValueType::Bool3,
            0x1a => ValueType::Int,
            0x1b => ValueType::Int2,
            0x1c => ValueType::Int3,
            0x20 => ValueType::Float,
            0x21 => ValueType::Float2,
            0x22 => ValueType::Float3,
            0x23 => ValueType::Float4,
            0x25 => ValueType::Quaternion,
            _ => return Err(self.error("unsupported register bank")),
        })
    }
    fn address(&self, register: Register) -> Result<Address, CompileError> {
        let scope = match register.scope() {
            0 | 0x10 => 0,
            1..=3 => usize::from(register.scope()),
            _ => return Err(self.error("unsupported register scope")),
        };
        if scope == 0 {
            if let Some(words) = self.script.constant(register.slot()) {
                return Ok(Address::Constant(words));
            }
        }
        let slot = usize::from(register.slot());
        if slot >= self.counts[scope] {
            return Err(self.error("register slot exceeds its declared scope"));
        }
        Ok(Address::Slot { scope, slot })
    }
    fn source(&self, register: Register) -> Result<Source, CompileError> {
        Ok(Source {
            kind: self.kind(register)?,
            address: self.address(register)?,
        })
    }
    fn destination(&self, register: Register) -> Result<Destination, CompileError> {
        let kind = self.kind(register)?;
        let Address::Slot { scope, slot } = self.address(register)? else {
            return Err(self.error("instruction writes to constant storage"));
        };
        Ok(Destination { scope, slot, kind })
    }
    fn external(&self, external: u16, kind: ValueType) -> Result<(), CompileError> {
        if self.externals.get(usize::from(external)) != Some(&kind) {
            return Err(self.error("external slot or value layout does not match its declaration"));
        }
        Ok(())
    }
    fn same(&self, actual: ValueType, expected: ValueType) -> Result<(), CompileError> {
        if actual != expected {
            return Err(self.error("incompatible instruction value layouts"));
        }
        Ok(())
    }
    fn shape(&self, input: ValueType, output: ValueType) -> Result<(), CompileError> {
        if input.components() != 1 && input.components() != output.components() {
            return Err(self.error("incompatible component counts"));
        }
        Ok(())
    }
    fn numeric(&self, kind: ValueType) -> Result<(), CompileError> {
        if !matches!(kind.family(), Family::Int | Family::Float) {
            return Err(self.error("instruction requires a numeric value"));
        }
        Ok(())
    }
    fn instruction(&self, native: &NativeInstruction) -> Result<Instruction, CompileError> {
        Ok(match native {
            NativeInstruction::Load {
                destination,
                external,
            } => {
                let destination = self.destination(*destination)?;
                self.external(*external, destination.kind)?;
                Instruction::Load {
                    destination,
                    external: *external,
                }
            }
            NativeInstruction::Store { source, external } => {
                let source = self.source(*source)?;
                self.external(*external, source.kind)?;
                Instruction::Store {
                    source,
                    external: *external,
                }
            }
            NativeInstruction::Reinterpret {
                destination,
                source,
            } => {
                let destination = self.destination(*destination)?;
                let source = self.source(*source)?;
                if destination.kind.components() != source.kind.components()
                    || (destination.kind.family() == Family::Context)
                        != (source.kind.family() == Family::Context)
                {
                    return Err(self.error("unsupported bit reinterpretation layout"));
                }
                Instruction::Reinterpret {
                    destination,
                    source,
                }
            }
            NativeInstruction::Convert {
                destination,
                source,
            } => {
                let destination = self.destination(*destination)?;
                let source = self.source(*source)?;
                self.shape(source.kind, destination.kind)?;
                if source.kind.family() == Family::Context
                    || destination.kind.family() == Family::Context
                {
                    return Err(self.error("opaque capabilities cannot be numerically converted"));
                }
                Instruction::Convert {
                    destination,
                    source,
                }
            }
            NativeInstruction::Construct {
                destination,
                sources,
            } => {
                let destination = self.destination(*destination)?;
                let sources = sources
                    .iter()
                    .map(|source| self.source(*source))
                    .collect::<Result<Vec<_>, _>>()?;
                if sources.is_empty()
                    || sources
                        .iter()
                        .any(|source| source.kind.family() != destination.kind.family())
                    || destination.kind.family() == Family::Context
                {
                    return Err(self.error("incompatible vector construction types"));
                }
                let count: usize = sources.iter().map(|source| source.kind.components()).sum();
                if count != destination.kind.components() && count != 1 {
                    return Err(self.error("vector construction does not fill the result"));
                }
                Instruction::Construct {
                    destination,
                    sources,
                }
            }
            NativeInstruction::Swizzle {
                destination,
                source,
                selectors,
            } => {
                let destination = self.destination(*destination)?;
                let source = self.source(*source)?;
                if source.kind.family() != destination.kind.family()
                    || source.kind.family() == Family::Context
                {
                    return Err(self.error("incompatible swizzle value types"));
                }
                let packed = u16::from(selectors[1]) | (u16::from(selectors[2] & 0xf0) << 4);
                let lanes = [0, 3, 6, 9].map(|shift| ((packed >> shift) & 7) as u8);
                if lanes[..destination.kind.components()].iter().any(|lane| {
                    *lane > 5 || (*lane < 4 && usize::from(*lane) >= source.kind.components())
                }) {
                    return Err(self.error("unsupported or out-of-range swizzle selector"));
                }
                Instruction::Swizzle {
                    destination,
                    source,
                    lanes,
                }
            }
            NativeInstruction::Binary {
                destination,
                left,
                right,
                operation,
            } => {
                let destination = self.destination(*destination)?;
                let left = self.source(*left)?;
                let right = if matches!(operation, 5 | 11) {
                    left
                } else {
                    self.source(*right)?
                };
                if left.kind.family() != right.kind.family() {
                    return Err(self.error("mixed basic-operation scalar families"));
                }
                self.shape(left.kind, destination.kind)?;
                self.shape(right.kind, destination.kind)?;
                match operation {
                    0..=5 => {
                        self.numeric(left.kind)?;
                        if destination.kind.family() != left.kind.family() {
                            return Err(self.error("arithmetic result changes scalar family"));
                        }
                    }
                    8 | 9 | 11 => {
                        if !matches!(left.kind.family(), Family::Int | Family::Bool)
                            || destination.kind.family() != left.kind.family()
                        {
                            return Err(self.error("unsupported logical or bitwise layout"));
                        }
                    }
                    12..=16 => {
                        if destination.kind.family() != Family::Bool
                            || left.kind.family() == Family::Context
                            || (left.kind.family() == Family::Bool && *operation != 16)
                        {
                            return Err(
                                self.error("comparison requires numeric inputs and boolean output")
                            );
                        }
                    }
                    _ => return Err(self.error("unsupported basic-operation selector")),
                }
                Instruction::Binary {
                    destination,
                    left,
                    right,
                    operation: *operation,
                }
            }
            NativeInstruction::UnaryFunction {
                destination,
                source,
                function,
            } => {
                let destination = self.destination(*destination)?;
                let source = self.source(*source)?;
                if source.kind.family() != Family::Float {
                    return Err(self.error("mathematical function requires floating-point input"));
                }
                match function {
                    7 => {
                        if source.kind != ValueType::Float || destination.kind != ValueType::Float2
                        {
                            return Err(self.error("unsupported sine/cosine pair layout"));
                        }
                    }
                    51 => {
                        if destination.kind.family() != Family::Bool {
                            return Err(self.error("finite-test requires boolean output"));
                        }
                        self.shape(source.kind, destination.kind)?;
                    }
                    0 | 1 | 13 | 17 | 23 | 24 => {
                        if destination.kind.family() != Family::Float {
                            return Err(
                                self.error("mathematical function requires floating-point output")
                            );
                        }
                        self.shape(source.kind, destination.kind)?;
                    }
                    _ => return Err(self.error("unsupported unary-function selector")),
                }
                Instruction::UnaryFunction {
                    destination,
                    source,
                    function: *function,
                }
            }
            NativeInstruction::BinaryFunction {
                destination,
                left,
                right,
                function,
            } => {
                let destination = self.destination(*destination)?;
                let left = self.source(*left)?;
                let right = self.source(*right)?;
                if [destination.kind, left.kind, right.kind]
                    .iter()
                    .any(|kind| kind.family() != Family::Float)
                {
                    return Err(self.error("mathematical function requires floating-point values"));
                }
                match function {
                    27 | 28 => {
                        self.shape(left.kind, destination.kind)?;
                        self.shape(right.kind, destination.kind)?;
                    }
                    29 => {
                        self.same(left.kind, right.kind)?;
                        self.same(destination.kind, ValueType::Float)?;
                    }
                    _ => return Err(self.error("unsupported binary-function selector")),
                }
                Instruction::BinaryFunction {
                    destination,
                    left,
                    right,
                    function: *function,
                }
            }
            NativeInstruction::Select {
                destination,
                if_false,
                if_true,
                condition,
            } => {
                let destination = self.destination(*destination)?;
                let if_false = self.source(*if_false)?;
                let if_true = self.source(*if_true)?;
                let condition = self.source(*condition)?;
                self.same(if_false.kind, destination.kind)?;
                self.same(if_true.kind, destination.kind)?;
                self.shape(condition.kind, destination.kind)?;
                if condition.kind.family() != Family::Bool {
                    return Err(self.error("selection condition must be boolean"));
                }
                Instruction::Select {
                    destination,
                    if_false,
                    if_true,
                    condition,
                }
            }
            NativeInstruction::Call {
                result,
                receiver,
                function,
                arguments,
                flags,
            } => {
                let signature = self.functions.get(usize::from(*function)).ok_or_else(|| {
                    self.error("call function slot is outside the declaration table")
                })?;
                if *receiver < -1 {
                    return Err(self.error("unsupported negative call receiver"));
                }
                if !matches!(flags, 0 | 2) {
                    return Err(self.error("unsupported call controls"));
                }
                let receiver = signature.receiver.or(if *receiver >= 0 {
                    Some(*receiver as u16)
                } else {
                    None
                });
                if receiver.is_some_and(|slot| usize::from(slot) >= self.externals.len()) {
                    return Err(
                        self.error("call receiver is outside the external declaration table")
                    );
                }
                if arguments.len() != signature.arguments.len() {
                    return Err(self.error("call argument count differs from its signature"));
                }
                let mut sources = Vec::with_capacity(arguments.len());
                let mut controls = Vec::with_capacity(arguments.len());
                for (argument, expected) in arguments.iter().zip(&signature.arguments) {
                    if !matches!(argument.flags, 0 | 8 | 10 | 24 | 26) {
                        return Err(self.error("unsupported call argument controls"));
                    }
                    let source = self.source(argument.register)?;
                    self.same(source.kind, *expected)?;
                    sources.push(source);
                    controls.push(argument.flags);
                }
                let destination = if *result == Register::VOID {
                    None
                } else {
                    let destination = self.destination(*result)?;
                    if Some(destination.kind) != signature.result {
                        return Err(self.error("call return register differs from its signature"));
                    }
                    Some(destination)
                };
                Instruction::Call {
                    destination,
                    result_kind: signature.result,
                    receiver,
                    function: *function,
                    arguments: sources,
                    controls,
                    flags: *flags,
                }
            }
        })
    }
}
