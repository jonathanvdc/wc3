//! Quaternion components for native animation conversion.
use super::*;
use crate::model::error::Location;

/// Converts the layout-selected quaternion components to editable floats.
pub trait RotationComponents: Value {
    /// Decodes the native four quaternion components.
    fn floats(self) -> [f32; 4];
    /// Encodes components, reporting any quantization.
    fn encode(value: [f32; 4], context: &mut Context, path: &Location) -> Result<Self, Error>;
}

impl RotationComponents for [f32; 4] {
    fn floats(self) -> [f32; 4] {
        self
    }
    fn encode(value: [f32; 4], _: &mut Context, _: &Location) -> Result<Self, Error> {
        Ok(value)
    }
}

impl RotationComponents for [i16; 4] {
    fn floats(self) -> [f32; 4] {
        self.map(|n| if n<0 {f32::from(n)+32768.} else {f32::from(n)-32767.} /32767.)
    }
    fn encode(value: [f32; 4], context: &mut Context, path: &Location) -> Result<Self, Error> {
        if value.iter().any(|v| !v.is_finite() || v.abs() > 1.) {
            return Err(Error::requirement(
                path.clone(),
                "quaternion components exceed the compressed M2 range",
            ));
        }
        let encoded = value.map(|n| {
            let n = (n * 32767.).round() as i32;
            if n < 0 {
                (n + 32767) as i16
            } else {
                (n - 32768) as i16
            }
        });
        let error = value
            .into_iter()
            .zip(encoded.floats())
            .map(|(a, b)| (a - b).abs())
            .fold(0., f32::max);
        context.approximate(path.clone(), error, "quaternion component quantization")?;
        Ok(encoded)
    }
}
