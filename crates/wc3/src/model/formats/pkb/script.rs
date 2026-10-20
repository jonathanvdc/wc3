//! Checked baked-script storage and instruction decoding.
use super::{Cursor, ReadError};

/// A lossless register address, retaining native bank and scope aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Register(#[doc = "Unmodified native register word."] pub u32);
impl Register {
    /// Native no-result sentinel.
    pub const VOID: Self = Self(u32::MAX);
    /// Native type/bank byte, independently of external declaration type IDs.
    pub fn bank(self) -> u8 {
        (self.0 >> 24) as u8
    }
    /// Unmodified native storage-scope byte.
    pub fn scope(self) -> u8 {
        (self.0 >> 16) as u8
    }
    /// Storage slot, excluding the bank and scope bytes.
    pub fn slot(self) -> u16 {
        self.0 as u16
    }
}

/// A native external-call argument with its uninterpreted controls intact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Argument {
    /// Uninterpreted native argument controls.
    pub flags: u8,
    /// Argument register selected by the declaration.
    pub register: Register,
}

/// The decoded IR instruction family used by baked 2.5.1 effects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Load a declared external slot into a register.
    Load {
        /// Result register.
        destination: Register,
        /// Zero-based external declaration slot.
        external: u16,
    },
    /// Store a register into a declared external slot.
    Store {
        /// Source register.
        source: Register,
        /// Zero-based external declaration slot.
        external: u16,
    },
    /// Change the type view without changing component bits.
    Reinterpret {
        /// Result register.
        destination: Register,
        /// Source register.
        source: Register,
    },
    /// Convert numeric values to the destination's type.
    Convert {
        /// Result register.
        destination: Register,
        /// Source register.
        source: Register,
    },
    /// Construct a vector from an ordered list of source registers.
    Construct {
        /// Result register.
        destination: Register,
        /// Ordered source registers.
        sources: Vec<Register>,
    },
    /// Select components or literal zero/one; preserve all packed selector bytes.
    Swizzle {
        /// Unmodified packed component selectors.
        selectors: [u8; 3],
        /// Result register.
        destination: Register,
        /// Source register.
        source: Register,
    },
    /// Basic typed arithmetic or comparison.
    Binary {
        /// Native basic operation selector.
        operation: u8,
        /// Result register.
        destination: Register,
        /// First source register.
        left: Register,
        /// Second source register.
        right: Register,
    },
    /// A numbered unary mathematical function.
    UnaryFunction {
        /// Native function selector or declaration slot.
        function: u8,
        /// Result register.
        destination: Register,
        /// Source register.
        source: Register,
    },
    /// A numbered binary mathematical function.
    BinaryFunction {
        /// Native function selector or declaration slot.
        function: u8,
        /// Result register.
        destination: Register,
        /// First source register.
        left: Register,
        /// Second source register.
        right: Register,
    },
    /// Choose false or true value according to the condition register.
    Select {
        /// Result register.
        destination: Register,
        /// Value chosen when the condition is false.
        if_false: Register,
        /// Value chosen when the condition is true.
        if_true: Register,
        /// Boolean selection register.
        condition: Register,
    },
    /// Call a declared function with a signed external-declaration receiver slot.
    Call {
        /// Native call controls.
        flags: u8,
        /// External-declaration slot; minus one supplies no receiver override.
        receiver: i16,
        /// Native function selector or declaration slot.
        function: u16,
        /// Return register, including the native void sentinel.
        result: Register,
        /// Ordered native argument controls and registers.
        arguments: Vec<Argument>,
    },
}

/// One instruction's byte position within its declared stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedInstruction {
    /// Offset from the beginning of the instruction stream.
    pub offset: usize,
    /// Decoded instruction preserving native controls and register addresses.
    pub instruction: Instruction,
}

/// Baked 2.5.1 script storage with checked declared regions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Script<'a> {
    /// Uninterpreted leading header words.
    pub reserved: [u32; 2],
    /// Native five-word register allocation declaration.
    pub register_counts: [u32; 5],
    /// Exact 32-byte constant slots, including their trailing uninterpreted words.
    pub constants: &'a [u8],
    /// Decoded instructions with byte positions.
    pub instructions: Vec<LocatedInstruction>,
    /// Word-array alignment bytes after the declared stream.
    pub padding: &'a [u8],
}
impl<'a> Script<'a> {
    /// Read one compiled-script body, excluding its outer native array count.
    /// Unknown encodings are rejected instead of being executed as no-ops.
    pub fn read(bytes: &'a [u8]) -> Result<Self, ReadError> {
        let mut input = Cursor::new(bytes, 0);
        let reserved = [input.word()?, input.word()?];
        let constant_bytes = input.word()? as usize;
        let code_bytes = input.word()? as usize;
        let register_counts = [
            input.word()?,
            input.word()?,
            input.word()?,
            input.word()?,
            input.word()?,
        ];
        if !constant_bytes.is_multiple_of(32) {
            return Err(input.error("compiled constants do not occupy complete 32-byte slots"));
        }
        let constants = input.take(constant_bytes)?;
        let code_origin = input.offset;
        let code = input.take(code_bytes)?;
        if input.remaining() > 3 {
            return Err(input.error("compiled script has more than one word of alignment padding"));
        }
        let padding = input.take(input.remaining())?;
        let mut stream = Cursor::new(code, code_origin);
        let mut instructions = Vec::new();
        while stream.remaining() != 0 {
            let offset = stream.offset;
            let instruction = match stream.byte()? {
                0x43 => Instruction::Load {
                    destination: Register(stream.word()?),
                    external: stream.short()?,
                },
                0x44 => Instruction::Store {
                    source: Register(stream.word()?),
                    external: stream.short()?,
                },
                0x4a => Instruction::Reinterpret {
                    destination: Register(stream.word()?),
                    source: Register(stream.word()?),
                },
                0x4b => Instruction::Convert {
                    destination: Register(stream.word()?),
                    source: Register(stream.word()?),
                },
                0x4c => {
                    let count = usize::from(stream.byte()?) + 1;
                    if count > 4 {
                        return Err(stream.error("vector construction has more than four sources"));
                    }
                    let destination = Register(stream.word()?);
                    let mut sources = Vec::with_capacity(count);
                    for _ in 0..count {
                        sources.push(Register(stream.word()?));
                    }
                    Instruction::Construct {
                        destination,
                        sources,
                    }
                }
                0x4d => Instruction::Swizzle {
                    selectors: stream.take(3)?.try_into().unwrap(),
                    destination: Register(stream.word()?),
                    source: Register(stream.word()?),
                },
                0x4e => Instruction::Binary {
                    operation: stream.byte()?,
                    destination: Register(stream.word()?),
                    left: Register(stream.word()?),
                    right: Register(stream.word()?),
                },
                0x4f => Instruction::UnaryFunction {
                    function: stream.byte()?,
                    destination: Register(stream.word()?),
                    source: Register(stream.word()?),
                },
                0x50 => Instruction::BinaryFunction {
                    function: stream.byte()?,
                    destination: Register(stream.word()?),
                    left: Register(stream.word()?),
                    right: Register(stream.word()?),
                },
                0x52 => Instruction::Select {
                    destination: Register(stream.word()?),
                    if_false: Register(stream.word()?),
                    if_true: Register(stream.word()?),
                    condition: Register(stream.word()?),
                },
                0x53 => {
                    let flags = stream.byte()?;
                    let receiver = stream.short()? as i16;
                    let function = stream.short()?;
                    let count = usize::from(stream.byte()?);
                    let result = Register(stream.word()?);
                    if count > stream.remaining() / 5 {
                        return Err(
                            stream.error("function arguments exceed the instruction stream")
                        );
                    }
                    let mut arguments = Vec::with_capacity(count);
                    for _ in 0..count {
                        arguments.push(Argument {
                            flags: stream.byte()?,
                            register: Register(stream.word()?),
                        });
                    }
                    Instruction::Call {
                        flags,
                        receiver,
                        function,
                        result,
                        arguments,
                    }
                }
                _ => {
                    return Err(ReadError {
                        offset: code_origin + offset,
                        message: "unsupported compiled instruction opcode",
                    })
                }
            };
            instructions.push(LocatedInstruction {
                offset,
                instruction,
            });
        }
        Ok(Self {
            reserved,
            register_counts,
            constants,
            instructions,
            padding,
        })
    }
    /// Read the four component words of a constant without interpreting its type.
    pub fn constant(&self, slot: u16) -> Option<[u32; 4]> {
        let start = usize::from(slot) * 32;
        let bytes = self.constants.get(start..start + 16)?;
        Some(
            [0, 4, 8, 12]
                .map(|offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())),
        )
    }
}
