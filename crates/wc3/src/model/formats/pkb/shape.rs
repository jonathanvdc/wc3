//! Native primitive shape sampler declarations, preserving authored omissions.
use super::{Cursor, Document, ReadError};
use glam::Vec3;

/// Native primitive sampler parameters before simulation defaults or coordinate conversion.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeSampler<'a> {
    /// Localized editor names as locale identifiers and borrowed strings.
    pub names: Vec<(u32, &'a [u8])>,
    /// Editor workspace coordinates, without simulation meaning.
    pub workspace: Option<[i32; 2]>,
    /// Native sampling domain selector; omission remains explicit.
    pub dimensionality: Option<u32>,
    /// Native primitive selector; omission remains explicit.
    pub shape_type: Option<i32>,
    /// Authored native-space translation.
    pub position: Option<Vec3>,
    /// Authored XYZ Euler angles in degrees.
    pub euler_degrees: Option<Vec3>,
    /// Authored complete box dimensions.
    pub box_dimensions: Option<Vec3>,
    /// Authored outer primitive radius.
    pub radius: Option<f32>,
    /// Authored inner primitive radius.
    pub inner_radius: Option<f32>,
    /// Authored complete axial height.
    pub height: Option<f32>,
    /// Authored hemisphere control, independent of sparse omission.
    pub hemisphere: Option<bool>,
    /// Authored scale for the complex ellipsoid primitive.
    pub nonuniform_scale: Option<Vec3>,
}
impl<'a> Document<'a> {
    /// Read shape parameters without guessing absent native defaults or sampling a point.
    pub fn shape_sampler(&self, link: u32) -> Result<ShapeSampler<'a>, ReadError> {
        let object = self.declaration(link, b"CParticleNodeSamplerData_Shape")?;
        let mut result = ShapeSampler {
            names: Vec::new(),
            workspace: None,
            dimensionality: None,
            shape_type: None,
            position: None,
            euler_degrees: None,
            box_dimensions: None,
            radius: None,
            inner_radius: None,
            height: None,
            hemisphere: None,
            nonuniform_scale: None,
        };
        self.fields(object, |field, input| {
            let vector = |input: &mut Cursor<'_>| -> Result<Vec3, ReadError> {
                Ok(Vec3::new(
                    f32::from_bits(input.word()?),
                    f32::from_bits(input.word()?),
                    f32::from_bits(input.word()?),
                ))
            };
            match field {
                0 => {
                    let count = input.count(8)?;
                    result.names.reserve(count);
                    for _ in 0..count {
                        result.names.push((input.word()?, self.string(input)?));
                    }
                }
                7 => result.workspace = Some([input.word()? as i32, input.word()? as i32]),
                9 => result.dimensionality = Some(input.word()?),
                13 => result.position = Some(vector(input)?),
                14 => result.euler_degrees = Some(vector(input)?),
                15 => result.shape_type = Some(input.word()? as i32),
                16 => result.box_dimensions = Some(vector(input)?),
                17 => result.radius = Some(f32::from_bits(input.word()?)),
                18 => result.inner_radius = Some(f32::from_bits(input.word()?)),
                20 => result.height = Some(f32::from_bits(input.word()?)),
                21 => {
                    result.hemisphere = Some(match input.byte()? {
                        0 => false,
                        1 => true,
                        _ => {
                            return Err(input.error("native shape boolean is neither zero nor one"))
                        }
                    })
                }
                22 => result.nonuniform_scale = Some(vector(input)?),
                _ => return Err(input.error("unsupported native shape-sampler field")),
            }
            Ok(())
        })?;
        Ok(result)
    }
}
