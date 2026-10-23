//! Backend-independent numeric operations after native decoding and type checks.
//!
//! These are semantic operations, not native selector values. Operand/result
//! layouts remain attached to the checked instruction; no backend decodes PKB
//! operation or mathematical-function numbers.

/// One checked binary operation, including elementwise and reducing functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    LogicalAnd,
    LogicalOr,
    BitAnd,
    BitOr,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    Minimum,
    Maximum,
    Dot,
}

/// One checked unary operation; sine/cosine returns a two-component pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UnaryOperation {
    Negate,
    LogicalNot,
    BitNot,
    SquareRoot,
    ReciprocalSquareRoot,
    Exponential,
    Reciprocal,
    SignedFraction,
    Saturate,
    IsFinite,
    SineCosine,
}
