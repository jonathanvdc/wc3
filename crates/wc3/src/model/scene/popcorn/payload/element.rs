//! Typed immutable endpoint descriptions; sampling policy is explicit and independent of transport.
use crate::model::scene::popcorn::{Value, ValueType};
use glam::Quat;
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Element {
    pub first: Value,
    pub second: Value,
    pub semantic: u32,
}
impl Element {
    pub fn new(first: Value, second: Value, semantic: u32) -> Result<Self> {
        let kind = first.kind();
        if second.kind() != kind {
            return Err("payload endpoint layouts differ");
        }
        let valid = match semantic {
            0 => matches!(
                kind,
                ValueType::Float
                    | ValueType::Float3
                    | ValueType::Float4
                    | ValueType::Quaternion
                    | ValueType::Int
            ),
            1 => matches!(kind, ValueType::Float | ValueType::Float4),
            256 | 258 => kind == ValueType::Float3,
            512 | 516 => kind == ValueType::Quaternion,
            768 => kind == ValueType::Int,
            _ => false,
        };
        if !valid {
            return Err("unsupported payload semantic or endpoint layout");
        }
        if semantic & 255 == 0 && first != second {
            return Err("constant payload endpoints differ");
        }
        if kind != ValueType::Int
            && [first, second].into_iter().any(|value| {
                value.words()[..kind.components()]
                    .iter()
                    .any(|word| !f32::from_bits(*word).is_finite())
            })
        {
            return Err("payload endpoints require finite floating components");
        }
        Ok(Self {
            first,
            second,
            semantic,
        })
    }
    pub fn sample(self, fraction: f32, emitted_index: u32) -> Result<Value> {
        if !fraction.is_finite() {
            return Err("payload interpolation fraction must be finite");
        }
        if self.semantic == 768 {
            if emitted_index > i32::MAX as u32 {
                return Err("payload emitted index exceeds signed native layout");
            }
            return Ok(Value::from_words(ValueType::Int, [emitted_index, 0, 0, 0]));
        }
        let interpolation = self.semantic & 255;
        if interpolation == 0 {
            return Ok(self.first);
        }
        let fraction = fraction.clamp(0., 1.);
        if interpolation == 4 {
            let normalized = |value: Value| {
                let quaternion = Quat::from_array(value.words().map(f32::from_bits));
                let square = quaternion.length_squared();
                if !square.is_finite() || square <= 1e-12 {
                    Err("interpolated payload orientation must have finite nonzero length")
                } else {
                    Ok(quaternion / square.sqrt())
                }
            };
            let first = normalized(self.first)?;
            let second = normalized(self.second)?;
            let result = first.slerp(second, fraction);
            if !result.is_finite() {
                return Err("payload orientation interpolation produced nonfinite components");
            }
            return Ok(Value::from(result));
        }
        let mut words = [0; 4];
        for (component, word) in words
            .iter_mut()
            .enumerate()
            .take(self.first.kind().components())
        {
            let first = f32::from_bits(self.first.words()[component]);
            let second = f32::from_bits(self.second.words()[component]);
            let result = first * (1. - fraction) + second * fraction;
            if !result.is_finite() {
                return Err("payload interpolation produced nonfinite components");
            }
            *word = result.to_bits();
        }
        Ok(Value::from_words(self.first.kind(), words))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec3, Vec4};
    #[test]
    fn constant_values_retain_bits_and_varying_samples_retain_hdr_and_alpha() {
        let constant = Value::from(Vec4::new(-0., 4., 2., 3.));
        assert_eq!(
            Element::new(constant, constant, 0)
                .unwrap()
                .sample(0.3, 0)
                .unwrap(),
            constant
        );
        let varying = Element::new(
            Value::from(Vec4::ZERO),
            Value::from(Vec4::new(4., 8., 2., 3.)),
            1,
        )
        .unwrap();
        assert_eq!(
            varying.sample(0.5, 0).unwrap(),
            Value::from(Vec4::new(2., 4., 1., 1.5))
        );
        let position = Element::new(
            Value::from(Vec3::ZERO),
            Value::from(Vec3::new(4., 8., 2.)),
            258,
        )
        .unwrap();
        assert_eq!(
            position.sample(0.25, 0).unwrap(),
            Value::from(Vec3::new(1., 2., 0.5))
        );
        assert!(position.sample(f32::NAN, 0).is_err());
    }
    #[test]
    fn orientation_uses_shortest_arc_and_index_uses_explicit_ordinal() {
        let rotation = Quat::from_rotation_z(1.);
        let orientation = Element::new(Value::from(rotation), Value::from(-rotation), 516).unwrap();
        let sampled = Quat::from_array(
            orientation
                .sample(0.5, 0)
                .unwrap()
                .words()
                .map(f32::from_bits),
        );
        assert!((sampled * Vec3::X - rotation * Vec3::X).length() < 1e-6);
        let index = Element::new(Value::from(0i32), Value::from(0i32), 768).unwrap();
        assert_eq!(index.sample(0.5, 13).unwrap(), Value::from(13i32));
        assert!(Element::new(Value::from(1.), Value::from(2.), 0).is_err());
        assert!(Element::new(Value::from(Vec3::ZERO), Value::from(Vec3::ZERO), 999).is_err());
    }
}
