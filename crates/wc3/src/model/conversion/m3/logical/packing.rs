//! Quantization selects native packing from current semantic values.
use crate::model::conversion::{Context, Error};
use crate::model::semantic::profiles::m3::UvEncoding;
use crate::model::semantic::{TextureCoordinates, Values};

#[derive(Clone, Copy, Debug)]
pub(super) struct UvTransform {
    pub scale: f32,
    pub offset: f32,
}
impl UvTransform {
    pub fn for_mesh<'a, 'data: 'a>(
        preferred: Option<UvEncoding>,
        channels: impl Into<TextureCoordinates<'a, 'data>>,
    ) -> Result<Self, Error> {
        let channels = channels.into();
        if let Some(preferred) = preferred {
            if !preferred.scale.is_finite() || !preferred.offset.is_finite() {
                return Err(Error::new(
                    "m3.uv.encoding",
                    "preferred UV encoding must be finite",
                ));
            }
            let transform = Self {
                scale: preferred.scale,
                offset: preferred.offset,
            };
            if channels
                .iter()
                .flat_map(Values::iter)
                .flatten()
                .all(|value| {
                    transform.fits(value) && transform.decode(transform.pack(value)) == value
                })
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
        (packed >= f64::from(i16::MIN) && packed <= f64::from(i16::MAX))
            // Float rounding can make an authored endpoint invert one step beyond i16.
            // Accept that endpoint only when the bounded code decodes exactly to it.
            || self.decode(packed as i16) == value
    }
    fn pack(self, value: f32) -> i16 {
        if self.scale == 0. {
            0
        } else {
            ((f64::from(value) - f64::from(self.offset)) / (f64::from(self.scale) / 32768.)).round()
                as i16
        }
    }
    fn decode(self, packed: i16) -> f32 {
        f32::from(packed) * (self.scale / 32768.) + self.offset
    }
    pub fn encode(self, value: f32, context: &mut Context) -> Result<i16, Error> {
        if !self.fits(value) {
            return Err(Error::new("m3.uv", "UV coordinate exceeds affine encoding"));
        }
        let packed = self.pack(value);
        let decoded = self.decode(packed);
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
    use super::*;
    use crate::model::conversion::{ApproximationPolicy, ExportOptions};

    #[test]
    fn canonical_packing_preserves_representable_coordinates() {
        let channels = [Values::owned(vec![[0.5, 0.50097656], [1.5, -1.5]])];
        let transform = UvTransform::for_mesh(None, &channels).unwrap();
        assert_eq!(transform.scale, 16.);
        assert_eq!(transform.offset, 0.);
        let mut context = Context::export(&ExportOptions::default());
        for value in channels.iter().flat_map(Values::iter).flatten() {
            let packed = transform.encode(value, &mut context).unwrap();
            assert_eq!(
                f32::from(packed) * transform.scale / 32768. + transform.offset,
                value
            );
        }
    }
    #[test]
    fn wide_ranges_choose_affine_packing_without_clamping() {
        let channels = [Values::owned(vec![[-1000., 1000.]])];
        let transform = UvTransform::for_mesh(None, &channels).unwrap();
        assert_ne!(transform.scale, 16.);
        assert!(transform.fits(-1000.) && transform.fits(1000.));
        let mut strict = Context::export(&ExportOptions::default());
        assert!(transform.encode(1.2345, &mut strict).is_err());
        let mut context = Context::export(&ExportOptions {
            approximation: ApproximationPolicy::Allow { tolerance: 0.1 },
            ..Default::default()
        });
        for value in [-1000., 1000.] {
            assert!(transform.encode(value, &mut context).is_ok());
        }
    }
    #[test]
    fn out_of_range_constant_coordinates_use_a_constant_transform() {
        let channels = [Values::owned(vec![[1000., 1000.]])];
        let transform = UvTransform::for_mesh(None, &channels).unwrap();
        assert_eq!(transform.scale, 0.);
        assert_eq!(transform.offset, 1000.);
        let mut context = Context::export(&ExportOptions::default());
        assert_eq!(transform.encode(1000., &mut context).unwrap(), 0);
        assert!(transform.encode(1001., &mut context).is_err());
    }
}

#[cfg(test)]
mod preferred_encoding_tests {
    use super::*;
    use crate::model::conversion::ExportOptions;

    #[test]
    fn source_grids_preserve_exact_coordinates_including_negative_and_zero_scales() {
        for preferred in [
            UvEncoding {
                scale: 32.,
                offset: 0.1,
            },
            UvEncoding {
                scale: -16.,
                offset: 0.1,
            },
            UvEncoding {
                scale: 0.,
                offset: 0.1,
            },
            UvEncoding {
                scale: f32::from_bits(1),
                offset: 0.1,
            },
            UvEncoding {
                scale: 16.,
                offset: 10000.,
            },
            UvEncoding {
                scale: -16.,
                offset: -10000.,
            },
        ] {
            let original = UvTransform {
                scale: preferred.scale,
                offset: preferred.offset,
            };
            let channels = [Values::owned(
                (i16::MIN..=i16::MAX)
                    .map(|packed| {
                        let value = original.decode(packed);
                        [value, value]
                    })
                    .collect::<Vec<_>>(),
            )];
            let selected = UvTransform::for_mesh(Some(preferred), &channels).unwrap();
            assert_eq!(
                (selected.scale, selected.offset),
                (preferred.scale, preferred.offset)
            );
            let mut context = Context::export(&ExportOptions::default());
            for value in channels[0].iter().flatten() {
                assert_eq!(
                    selected.decode(selected.encode(value, &mut context).unwrap()),
                    value
                );
            }
            assert!(context.report.issues.is_empty());
        }
    }

    #[test]
    fn incompatible_edits_replace_preferred_grid_without_clamping() {
        let preferred = UvEncoding {
            scale: 32.,
            offset: 0.,
        };
        // This edit fits the preferred range, but needs a finer grid.
        let channels = [Values::owned(vec![[1. / 2048., 0.]])];
        let selected = UvTransform::for_mesh(Some(preferred), &channels).unwrap();
        assert_eq!((selected.scale, selected.offset), (16., 0.));
        let mut context = Context::export(&ExportOptions::default());
        assert_eq!(selected.encode(1. / 2048., &mut context).unwrap(), 1);
        let wide = [Values::owned(vec![[-1000., 1000.]])];
        let selected = UvTransform::for_mesh(Some(preferred), &wide).unwrap();
        assert!(selected.fits(-1000.) && selected.fits(1000.));
        assert_ne!(selected.scale, preferred.scale);
        let constant = UvEncoding {
            scale: 0.,
            offset: 0.1,
        };
        let selected = UvTransform::for_mesh(Some(constant), &channels).unwrap();
        assert_eq!(selected.scale, 16.);
    }

    #[test]
    fn invalid_preferences_and_coordinates_are_rejected() {
        let channels = [Values::owned(vec![[0.; 2]])];
        for preferred in [
            UvEncoding {
                scale: f32::NAN,
                offset: 0.,
            },
            UvEncoding {
                scale: 16.,
                offset: f32::INFINITY,
            },
        ] {
            assert!(UvTransform::for_mesh(Some(preferred), &channels).is_err());
        }
        let invalid = [Values::owned(vec![[f32::NAN, 0.]])];
        assert!(UvTransform::for_mesh(Some(UvEncoding::default()), &invalid).is_err());
    }
}
