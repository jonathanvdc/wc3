//! Typed execution of compiled particle programmes, independent of rendering.
//!
//! A programme resolves storage and call signatures before simulation. The host
//! owns named fields, resources, random state and event effects; instructions
//! carry numeric bindings rather than looking up native names per particle.

use glam::{Quat, Vec2, Vec3, Vec4};

mod age;
mod immutable;
pub use immutable::{
    ImmutableBirth, ImmutableCapture, ImmutableCurve, ImmutableExpression, ImmutableNode,
    ImmutableNodeId, ImmutableOutput, ImmutableParticleView, ImmutableRecipe, ImmutableSample,
};

mod numeric;
pub(crate) use numeric::{BinaryOperation, UnaryOperation};

mod execution;
mod execution_plan;
mod execution_policy;
pub use execution_policy::{
    ExecutionBackend, ExecutionFallback, ExecutionOptions, ExecutionPreference, ExecutionReport,
    ExecutionStage, ExecutionStats, NumericPolicy,
};
#[cfg(all(
    feature = "popcorn-jit",
    any(unix, windows),
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "s390x"
    )
))]
mod native;
#[cfg(not(all(
    feature = "popcorn-jit",
    any(unix, windows),
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "s390x"
    )
)))]
#[path = "native_unavailable.rs"]
mod native;
pub use execution::{ExecutionError, Failure, Host, State};

/// Value layouts accepted by the scalar particle evaluator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    /// One boolean component.
    Bool,
    /// Three boolean components.
    Bool3,
    /// One signed integer or payload identifier.
    Int,
    /// Two signed integers.
    Int2,
    /// Three signed integers.
    Int3,
    /// One floating-point component.
    Float,
    /// Two floating-point components.
    Float2,
    /// Three floating-point components.
    Float3,
    /// Four floating-point components.
    Float4,
    /// Four floating-point quaternion components.
    Quaternion,
    /// Opaque host capability, represented by an external binding identity.
    Context,
}

impl ValueType {
    /// Number of component words in this layout.
    pub const fn components(self) -> usize {
        match self {
            Self::Bool | Self::Int | Self::Float | Self::Context => 1,
            Self::Int2 | Self::Float2 => 2,
            Self::Bool3 | Self::Int3 | Self::Float3 => 3,
            Self::Float4 | Self::Quaternion => 4,
        }
    }
    pub(crate) const fn family(self) -> Family {
        match self {
            Self::Bool | Self::Bool3 => Family::Bool,
            Self::Int | Self::Int2 | Self::Int3 => Family::Int,
            Self::Float | Self::Float2 | Self::Float3 | Self::Float4 | Self::Quaternion => {
                Family::Float
            }
            Self::Context => Family::Context,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Family {
    Bool,
    Int,
    Float,
    Context,
}

/// A typed value retaining the exact bits of each active component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Value {
    kind: ValueType,
    words: [u32; 4],
}
impl Value {
    /// Construct a bit-preserving value; inactive words are cleared.
    pub fn from_words(kind: ValueType, mut words: [u32; 4]) -> Self {
        words[kind.components()..].fill(0);
        Self { kind, words }
    }
    /// Layout of the active component words.
    pub fn kind(self) -> ValueType {
        self.kind
    }
    /// Component bits, with inactive words zeroed.
    pub fn words(self) -> [u32; 4] {
        self.words
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::from_words(ValueType::Float, [value.to_bits(), 0, 0, 0])
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::from_words(ValueType::Int, [value as u32, 0, 0, 0])
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::from_words(ValueType::Bool, [if value { u32::MAX } else { 0 }, 0, 0, 0])
    }
}
impl From<Vec2> for Value {
    fn from(value: Vec2) -> Self {
        Self::from_words(
            ValueType::Float2,
            [value.x.to_bits(), value.y.to_bits(), 0, 0],
        )
    }
}
impl From<Vec3> for Value {
    fn from(value: Vec3) -> Self {
        Self::from_words(
            ValueType::Float3,
            value.extend(0.).to_array().map(f32::to_bits),
        )
    }
}
impl From<Vec4> for Value {
    fn from(value: Vec4) -> Self {
        Self::from_words(ValueType::Float4, value.to_array().map(f32::to_bits))
    }
}
impl From<Quat> for Value {
    fn from(value: Quat) -> Self {
        Self::from_words(ValueType::Quaternion, value.to_array().map(f32::to_bits))
    }
}

/// Checked signature for a host function resolved at compilation time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    /// External binding used as receiver, or no bound receiver.
    pub receiver: Option<u16>,
    /// Ordered read argument layouts. Context mutation belongs to the host.
    pub arguments: Vec<ValueType>,
    /// Return layout, or no return value.
    pub result: Option<ValueType>,
}

/// Immutable, checked instructions sharing a single storage layout.
#[derive(Clone, Debug)]
pub struct Program {
    pub(crate) counts: [usize; 4],
    pub(crate) max_arguments: usize,
    pub(crate) operations: Vec<Operation>,
}
impl Program {
    /// Allocate cleared particle storage compatible with this programme.
    pub fn new_state(&self) -> State {
        State::for_programs([self])
    }
    /// Number of lowered instructions.
    pub fn instruction_count(&self) -> usize {
        self.operations.len()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Address {
    Constant([u32; 4]),
    Slot { scope: usize, slot: usize },
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Source {
    pub address: Address,
    pub kind: ValueType,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Destination {
    pub scope: usize,
    pub slot: usize,
    pub kind: ValueType,
}
#[derive(Clone, Debug)]
pub(crate) struct Operation {
    pub offset: usize,
    pub instruction: Instruction,
}
#[derive(Clone, Debug)]
pub(crate) enum Instruction {
    Load {
        destination: Destination,
        external: u16,
    },
    Store {
        source: Source,
        external: u16,
    },
    Reinterpret {
        destination: Destination,
        source: Source,
    },
    Convert {
        destination: Destination,
        source: Source,
    },
    Construct {
        destination: Destination,
        sources: Vec<Source>,
    },
    Swizzle {
        destination: Destination,
        source: Source,
        lanes: [u8; 4],
    },
    Binary {
        destination: Destination,
        left: Source,
        right: Source,
        operation: BinaryOperation,
    },
    Unary {
        destination: Destination,
        source: Source,
        operation: UnaryOperation,
    },
    Select {
        destination: Destination,
        if_false: Source,
        if_true: Source,
        condition: Source,
    },
    Call {
        destination: Option<Destination>,
        result_kind: Option<ValueType>,
        receiver: Option<u16>,
        function: u16,
        arguments: Vec<Source>,
        controls: Vec<u8>,
        flags: u8,
    },
}

mod callbacks;
mod color;
mod curve;
pub(crate) mod graph;
pub(crate) mod layout;
mod orientation;
pub(crate) mod packet;
mod runtime;
pub use graph::{
    EffectAttribute, EffectAttributeId, EffectEntry, EffectField, EffectFieldId, EffectLayer,
    EffectLayerId, EffectLimitation, EffectParticleId, EffectProgram, EffectRandom, EffectRenderer,
    EffectRendererInput, EffectRendererProperty, EventStreamDuration, MissingPayloadPolicy,
    SceneContext, SceneInputs,
};
pub use runtime::{EffectInstance, EffectLimits, EffectRuntimeError, EffectStats, ParticleView};

mod lifetime;
pub(crate) mod shapes;

pub(crate) mod payload;

mod random;

mod transforms;

mod spatial;

pub(crate) mod turbulence;
