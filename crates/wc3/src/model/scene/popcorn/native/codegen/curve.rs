//! Scalar curve interval selection; interpolation algebra is shared with SIMD.
use super::Lower;
use crate::model::scene::popcorn::graph::EffectSampler;
use crate::model::scene::popcorn::native::interpolation;
use crate::model::scene::popcorn::Destination;
use cranelift_codegen::ir::condcodes::FloatCC;
use cranelift_codegen::ir::{types, Block, InstBuilder, Value};

impl Lower<'_, '_> {
    fn float_constant(&mut self, value: f32) -> Value {
        interpolation::constant(self.builder, types::F32, value)
    }
    pub(super) fn curve(
        &mut self,
        destination: Destination,
        coordinate: Value,
        sampler: &EffectSampler,
    ) {
        let EffectSampler::Curve {
            times,
            values,
            tangents,
            ..
        } = sampler
        else {
            unreachable!("validated curve binding")
        };
        let time = self.float(coordinate);
        let done = self.builder.create_block();
        let first = self.builder.create_block();
        let after_first = self.builder.create_block();
        let key = self.float_constant(times[0]);
        let before = self.builder.ins().fcmp(FloatCC::LessThanOrEqual, time, key);
        self.builder
            .ins()
            .brif(before, first, &[], after_first, &[]);
        self.builder.switch_to_block(first);
        self.curve_endpoint(destination, values[0]);
        self.builder.ins().jump(done, &[]);
        self.builder.switch_to_block(after_first);
        if times.len() == 1 {
            self.curve_endpoint(destination, values[0]);
            self.builder.ins().jump(done, &[]);
        } else {
            let last = self.builder.create_block();
            let interior = self.builder.create_block();
            let key = self.float_constant(times[times.len() - 1]);
            let after = self
                .builder
                .ins()
                .fcmp(FloatCC::GreaterThanOrEqual, time, key);
            self.builder.ins().brif(after, last, &[], interior, &[]);
            self.builder.switch_to_block(last);
            self.curve_endpoint(destination, values[values.len() - 1]);
            self.builder.ins().jump(done, &[]);
            self.builder.switch_to_block(interior);
            self.curve_intervals(
                destination,
                time,
                times,
                values,
                tangents,
                0,
                times.len() - 1,
                done,
            );
        }
        self.builder.switch_to_block(done);
    }
    fn curve_endpoint(&mut self, destination: Destination, value: [f32; 4]) {
        let words: Vec<_> = value
            .iter()
            .take(destination.kind.components())
            .map(|v| {
                self.builder
                    .ins()
                    .iconst(types::I32, i64::from(v.to_bits()))
            })
            .collect();
        self.write(destination, &words);
    }
    #[allow(clippy::too_many_arguments)]
    fn curve_intervals(
        &mut self,
        destination: Destination,
        time: Value,
        times: &[f32],
        values: &[[f32; 4]],
        tangents: &[[f32; 4]],
        start: usize,
        end: usize,
        done: Block,
    ) {
        if end - start == 1 {
            self.curve_interval(destination, time, times, values, tangents, start);
            self.builder.ins().jump(done, &[]);
            return;
        }
        let middle = (start + end) / 2;
        let key = self.float_constant(times[middle]);
        let before = self.builder.ins().fcmp(FloatCC::LessThan, time, key);
        let left = self.builder.create_block();
        let right = self.builder.create_block();
        self.builder.ins().brif(before, left, &[], right, &[]);
        self.builder.switch_to_block(left);
        self.curve_intervals(
            destination,
            time,
            times,
            values,
            tangents,
            start,
            middle,
            done,
        );
        self.builder.switch_to_block(right);
        self.curve_intervals(
            destination,
            time,
            times,
            values,
            tangents,
            middle,
            end,
            done,
        );
    }
    fn curve_interval(
        &mut self,
        destination: Destination,
        time: Value,
        times: &[f32],
        values: &[[f32; 4]],
        tangents: &[[f32; 4]],
        index: usize,
    ) {
        // Keep shared curve::sample's explicit scalar f32 operation order.
        // Constants are bit-exact values, never pre-expanded cubic coefficients.
        let key0 = self.float_constant(times[index]);
        let key1 = self.float_constant(times[index + 1]);
        let t = interpolation::coordinate(self.builder, time, key0, key1);
        let weights = if tangents.is_empty() {
            None
        } else {
            Some(interpolation::hermite_weights(self.builder, t))
        };
        let mut words = Vec::new();
        for component in 0..destination.kind.components() {
            let a = self.float_constant(values[index][component]);
            let b = self.float_constant(values[index + 1][component]);
            let result = if let Some(weights) = weights {
                let out = self.float_constant(tangents[index * 2 + 1][component]);
                let input = self.float_constant(tangents[(index + 1) * 2][component]);
                interpolation::hermite(self.builder, [a, out, b, input], weights)
            } else {
                interpolation::linear(self.builder, a, b, t)
            };
            words.push(self.bits(result));
        }
        self.write(destination, &words);
    }
}
