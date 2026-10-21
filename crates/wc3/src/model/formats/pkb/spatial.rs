//! Borrowed native spatial resource declarations, separate from live snapshots.
use super::{Document, NativeType, ReadError};
/// One layer-local binding to a shared authored spatial resource.
#[derive(Clone, Debug, PartialEq)]
pub struct SpatialLayer<'a> {
    /// Full authored resource identity, independent of its layer-local alias.
    pub name: &'a [u8],
    /// Script external binding name within this layer.
    pub local_name: &'a [u8],
    /// Native controls; absence is retained without inferred behavior.
    pub flags: Option<u32>,
    /// Authored cell width; absence is retained without an invented default.
    pub cell_size: Option<f32>,
    /// Ordered named payload declarations.
    pub payload: Vec<SpatialPayload<'a>>,
}
/// A named spatial value; write/read flags are independent of its identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpatialPayload<'a> {
    /// Authored payload name.
    pub name: &'a [u8],
    /// Lossless native value layout selector.
    pub native_type: NativeType,
    /// Native controls with sparse zero default.
    pub flags: u32,
}
impl<'a> Document<'a> {
    /// Read an authored declaration; this allocates no live spatial store.
    pub fn spatial_layer(&self, link: u32) -> Result<SpatialLayer<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheSpatialLayer")?;
        let (mut name, mut local_name, mut flags, mut cell_size) = (None, None, None, None);
        let mut payload = Vec::new();
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => local_name = Some(self.string(input)?),
                2 => flags = Some(input.word()?),
                3 => cell_size = Some(f32::from_bits(input.word()?)),
                4 => {
                    payload = self
                        .links(input)?
                        .into_iter()
                        .map(|link| self.spatial_payload(link))
                        .collect::<Result<_, _>>()?;
                }
                _ => return Err(input.error("unsupported spatial declaration field")),
            }
            Ok(())
        })?;
        Ok(SpatialLayer {
            name: self.required(object, name)?,
            local_name: self.required(object, local_name)?,
            flags,
            cell_size,
            payload,
        })
    }
    fn spatial_payload(&self, link: u32) -> Result<SpatialPayload<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerCompileCacheSpatialLayerPayload")?;
        let (mut name, mut native_type) = (None, None);
        let mut flags = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = Some(self.string(input)?),
                1 => native_type = Some(NativeType(input.word()?)),
                2 => flags = input.word()?,
                _ => return Err(input.error("unsupported spatial payload declaration field")),
            }
            Ok(())
        })?;
        Ok(SpatialPayload {
            name: self.required(object, name)?,
            native_type: self.required(object, native_type)?,
            flags,
        })
    }
}
