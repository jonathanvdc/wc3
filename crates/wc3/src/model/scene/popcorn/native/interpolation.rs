//! Ordered f32 interpolation algebra shared by scalar and SIMD native lowering.
//! Interval admission, branch/mask selection and effects stay with each caller.
use cranelift_codegen::ir::immediates::Ieee32;
use cranelift_codegen::ir::{types, InstBuilder, Type, Value};
use cranelift_frontend::FunctionBuilder;

/// Construct the same f32 bits, optionally broadcast across native SIMD lanes.
pub(crate) fn constant(builder: &mut FunctionBuilder<'_>, lane_type: Type, value: f32) -> Value {
    debug_assert!(lane_type.lane_type() == types::F32);
    let scalar = builder.ins().f32const(Ieee32::with_bits(value.to_bits()));
    if lane_type == types::F32 {
        scalar
    } else {
        builder.ins().splat(lane_type, scalar)
    }
}

/// Preserve `(time - first) / (last - first)` in the caller's f32 lane type.
pub(crate) fn coordinate(
    builder: &mut FunctionBuilder<'_>,
    time: Value,
    first: Value,
    last: Value,
) -> Value {
    let numerator = builder.ins().fsub(time, first);
    let denominator = builder.ins().fsub(last, first);
    builder.ins().fdiv(numerator, denominator)
}

/// Preserve `first + (last - first) * t` without explicit contraction.
pub(crate) fn linear(
    builder: &mut FunctionBuilder<'_>,
    first: Value,
    last: Value,
    t: Value,
) -> Value {
    let difference = builder.ins().fsub(last, first);
    let product = builder.ins().fmul(difference, t);
    builder.ins().fadd(first, product)
}

/// The same four Hermite basis terms as the checked Rust curve sampler.
pub(crate) fn hermite_weights(builder: &mut FunctionBuilder<'_>, t: Value) -> [Value; 4] {
    let lane_type = builder.func.dfg.value_type(t);
    let two = constant(builder, lane_type, 2.);
    let three = constant(builder, lane_type, 3.);
    let one = constant(builder, lane_type, 1.);
    let negative_two = constant(builder, lane_type, -2.);
    let t2 = builder.ins().fmul(t, t);
    let t3 = builder.ins().fmul(t2, t);
    let twice_t3 = builder.ins().fmul(two, t3);
    let thrice_t2 = builder.ins().fmul(three, t2);
    let difference = builder.ins().fsub(twice_t3, thrice_t2);
    let h00 = builder.ins().fadd(difference, one);
    let twice_t2 = builder.ins().fmul(two, t2);
    let difference = builder.ins().fsub(t3, twice_t2);
    let h10 = builder.ins().fadd(difference, t);
    let negative_twice_t3 = builder.ins().fmul(negative_two, t3);
    let h01 = builder.ins().fadd(negative_twice_t3, thrice_t2);
    let h11 = builder.ins().fsub(t3, t2);
    [h00, h10, h01, h11]
}

/// Apply `[first, outgoing tangent, last, incoming tangent]` in source sum order.
pub(crate) fn hermite(
    builder: &mut FunctionBuilder<'_>,
    terms: [Value; 4],
    weights: [Value; 4],
) -> Value {
    let first = builder.ins().fmul(weights[0], terms[0]);
    let outgoing = builder.ins().fmul(weights[1], terms[1]);
    let sum = builder.ins().fadd(first, outgoing);
    let last = builder.ins().fmul(weights[2], terms[2]);
    let sum = builder.ins().fadd(sum, last);
    let incoming = builder.ins().fmul(weights[3], terms[3]);
    builder.ins().fadd(sum, incoming)
}
