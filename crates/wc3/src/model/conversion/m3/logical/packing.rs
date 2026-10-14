//! Quantization uses current graph values; optional layout hints never supply geometry.
use crate::model::conversion::{Context, EncodingHint, Error};
use crate::model::semantic::{MeshId, Values};

#[derive(Clone, Copy, Debug)]
pub(super) struct UvTransform {
    pub scale: f32,
    pub offset: f32,
}
impl UvTransform {
    pub fn for_mesh(
        mesh: MeshId,
        hints: &[EncodingHint<'_>],
        channels: &[Values<'_, [f32; 2]>],
    ) -> Result<Self, Error> {
        let mut selected = None;
        for hint in hints {
            if let EncodingHint::M3TextureCoordinates {
                mesh: owner,
                scale,
                offset,
            } = hint
            {
                if *owner != mesh {
                    continue;
                }
                if !scale.is_finite() || !offset.is_finite() {
                    return Err(Error::new("m3.uv.hint", "UV encoding hint must be finite"));
                }
                if let Some(previous) = selected {
                    let previous: Self = previous;
                    if previous.scale != *scale || previous.offset != *offset {
                        return Err(Error::new(
                            "m3.uv.hint",
                            "conflicting UV hints for one mesh",
                        ));
                    }
                }
                selected = Some(Self {
                    scale: *scale,
                    offset: *offset,
                });
            }
        }
        // Use a hint only when every current coordinate fits its affine encoding.
        if let Some(transform) = selected {
            if channels
                .iter()
                .flat_map(Values::iter)
                .flatten()
                .all(|value| transform.fits(value))
            {
                return Ok(transform);
            }
        }
        let canonical = Self {
            scale: 16.,
            offset: 0.,
        };
        if channels
            .iter()
            .flat_map(Values::iter)
            .flatten()
            .all(|value| canonical.fits(value))
        {
            return Ok(canonical);
        }
        // A common transform for every UV channel fits the authored range in i16.
        let mut minimum = f32::INFINITY;
        let mut maximum = f32::NEG_INFINITY;
        for value in channels.iter().flat_map(Values::iter).flatten() {
            if !value.is_finite() {
                return Err(Error::new("m3.uv", "UV coordinates must be finite"));
            }
            minimum = minimum.min(value);
            maximum = maximum.max(value);
        }
        if minimum == f32::INFINITY {
            return Ok(canonical);
        }
        if minimum == maximum {
            return Ok(Self {
                scale: 0.,
                offset: minimum,
            });
        }
        let offset = ((f64::from(minimum) + f64::from(maximum)) * 0.5) as f32;
        let half_range = (f64::from(maximum) - f64::from(minimum)) * 0.5;
        // Leave one integer step of headroom for rounded float endpoints.
        let scale = (half_range / 32766. * 32768.) as f32;
        let transform = Self { scale, offset };
        if !scale.is_finite()
            || scale == 0.
            || !channels
                .iter()
                .flat_map(Values::iter)
                .flatten()
                .all(|value| transform.fits(value))
        {
            return Err(Error::new(
                "m3.uv",
                "UV range cannot be packed with a finite affine transform",
            ));
        }
        Ok(transform)
    }
    fn fits(self, value: f32) -> bool {
        if !value.is_finite() {
            return false;
        }
        if self.scale == 0. {
            return value == self.offset;
        }
        let packed = ((f64::from(value) - f64::from(self.offset))
            / (f64::from(self.scale) / 32768.))
            .round();
        packed >= f64::from(i16::MIN) && packed <= f64::from(i16::MAX)
    }
    pub fn encode(self, value: f32, context: &mut Context) -> Result<i16, Error> {
        if !self.fits(value) {
            return Err(Error::new("m3.uv", "UV coordinate exceeds affine encoding"));
        }
        let packed = if self.scale == 0. {
            0
        } else {
            ((f64::from(value) - f64::from(self.offset)) / (f64::from(self.scale) / 32768.)).round()
                as i16
        };
        let decoded = f32::from(packed) * (self.scale / 32768.) + self.offset;
        context.approximate(
            "m3.uv",
            (decoded - value).abs(),
            "packing a texture coordinate changes its authored value",
        )?;
        Ok(packed)
    }
}
pub(super) fn unorm(
    value: f32,
    signed: bool,
    path: &str,
    context: &mut Context,
) -> Result<u8, Error> {
    if !value.is_finite() {
        return Err(Error::new(path, "packed attribute must be finite"));
    }
    let mapped = if signed { (value + 1.) * 0.5 } else { value };
    let packed = (mapped.clamp(0., 1.) * 255.).round() as u8;
    let decoded = if signed {
        f32::from(packed) / 255. * 2. - 1.
    } else {
        f32::from(packed) / 255.
    };
    context.approximate(
        path,
        (decoded - value).abs(),
        "packing an attribute changes its authored value",
    )?;
    Ok(packed)
}

#[cfg(test)]
mod tests {
    use super::super::GraphBuilder;
    use super::*;
    use crate::model::semantic::{M3Geometry, Mesh};
    fn mesh() -> MeshId {
        let mut builder = GraphBuilder::with_profile("UV packing");
        builder.add_mesh(Mesh::new(M3Geometry::empty())).unwrap()
    }
    #[test]
    fn existing_affine_hint_tracks_current_coordinates() {
        let id = mesh();
        let hints = [EncodingHint::M3TextureCoordinates {
            mesh: id,
            scale: 32.,
            offset: 0.5,
        }];
        let channels = [Values::owned(vec![[0.5, 0.50097656], [1.5, -1.5]])];
        let transform = UvTransform::for_mesh(id, &hints, &channels).unwrap();
        assert_eq!(transform.scale, 32.);
        assert_eq!(transform.offset, 0.5);
        assert!(channels
            .iter()
            .flat_map(Values::iter)
            .flatten()
            .all(|x| transform.fits(x)));
    }
    #[test]
    fn stale_hint_does_not_supply_or_clamp_current_uvs() {
        let id = mesh();
        let hints = [EncodingHint::M3TextureCoordinates {
            mesh: id,
            scale: 16.,
            offset: 0.,
        }];
        let channels = [Values::owned(vec![[-1000., 1000.]])];
        let transform = UvTransform::for_mesh(id, &hints, &channels).unwrap();
        assert_ne!(transform.scale, 16.);
        assert!(transform.fits(-1000.) && transform.fits(1000.));
    }
    #[test]
    fn constant_source_transform_and_conflicting_hints_are_checked() {
        let id = mesh();
        let hints = [EncodingHint::M3TextureCoordinates {
            mesh: id,
            scale: 0.,
            offset: 3.,
        }];
        let channels = [Values::owned(vec![[3., 3.]])];
        assert_eq!(
            UvTransform::for_mesh(id, &hints, &channels).unwrap().scale,
            0.
        );
        let mut conflicting = hints.to_vec();
        conflicting.push(EncodingHint::M3TextureCoordinates {
            mesh: id,
            scale: 16.,
            offset: 0.,
        });
        assert!(UvTransform::for_mesh(id, &conflicting, &channels).is_err());
    }
}
