//! Packed fields are produced from the current logical attributes and influences.
use super::packing::{unorm, UvTransform};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::records::VertexFlags;
use crate::model::profiles::Profile;
use crate::model::{Influence, Mesh};

pub(super) struct Layout {
    pub uv_mask: u8,
    pub color: bool,
}
impl Layout {
    pub fn stride(&self) -> usize {
        28 + 4 * self.uv_mask.count_ones() as usize + usize::from(self.color) * 4
    }
    pub fn flags(&self) -> u32 {
        let uv = u32::from(self.uv_mask);
        let mut flags = VertexFlags::default();
        flags.set_position(true);
        flags.set_skin0(true);
        flags.set_skin1(true);
        flags.set_normal(true);
        flags.set_tangent(true);
        flags.set_color(self.color);
        flags.bits() | ((uv & 15) << 17) | ((uv & 16) << 26)
    }
    pub fn row<P: Profile>(
        &self,
        mesh: &Mesh<'_, P>,
        index: usize,
        uv: UvTransform,
        context: &mut Context,
    ) -> Result<Vec<u8>, Error> {
        let mut out = Vec::with_capacity(self.stride());
        for value in mesh
            .positions()
            .get(index)
            .ok_or_else(|| Error::new("m3.vertex", "position is missing"))?
        {
            out.extend(value.to_le_bytes());
        }
        let influences = mesh.influences().joint_weights(index).unwrap_or_default();
        let (weights, joints) = weights(&influences, context)?;
        out.extend(weights);
        out.extend(joints);
        if let Some(normal) = mesh.normals().and_then(|values| values.get(index)) {
            for value in normal {
                out.push(unorm(value, true, "m3.vertex.normal", context)?);
            }
        } else {
            out.extend([128, 128, 255]);
        }
        out.push(0);
        if self.color {
            let color = mesh
                .colors()
                .and_then(|values| values.get(index))
                .unwrap_or([1.; 4]);
            for component in [2, 1, 0, 3] {
                out.push(unorm(color[component], false, "m3.vertex.color", context)?);
            }
        }
        for channel in 0..5 {
            if self.uv_mask & (1 << channel) == 0 {
                continue;
            }
            let coordinate = mesh
                .texture_coordinates()
                .get(channel)
                .and_then(|set| set.get(index))
                .unwrap_or([0.; 2]);
            for value in coordinate {
                out.extend(uv.encode(value, context)?.to_le_bytes());
            }
        }
        if let Some(tangent) = mesh.tangents().and_then(|values| values.get(index)) {
            for value in tangent {
                out.push(unorm(value, true, "m3.vertex.tangent", context)?);
            }
        } else {
            out.extend([255, 128, 128, 255]);
        }
        Ok(out)
    }
}
fn weights(influences: &[Influence], context: &mut Context) -> Result<([u8; 4], [u8; 4]), Error> {
    if influences.len() > 4 {
        return Err(Error::new(
            "m3.vertex.skin",
            "M3 packed vertices support at most four influence lanes",
        ));
    }
    let mut weights = [0u8; 4];
    let mut joints = [0u8; 4];
    let total: f64 = influences.iter().map(|x| f64::from(x.weight)).sum();
    if influences.iter().all(|influence| influence.weight == 0.) {
        for (lane, influence) in influences.iter().enumerate() {
            joints[lane] = u8::try_from(influence.joint)
                .map_err(|_| Error::new("m3.vertex.skin", "joint palette exceeds byte capacity"))?;
        }
        return Ok((weights, joints));
    }
    if total <= 0. || !total.is_finite() {
        return Err(Error::new(
            "m3.vertex.skin",
            "positive finite influence sum is required",
        ));
    }
    let mut fractions = Vec::new();
    let mut sum = 0u16;
    for (lane, influence) in influences.iter().enumerate() {
        joints[lane] = u8::try_from(influence.joint).map_err(|_| {
            Error::new(
                "m3.vertex.skin",
                "joint palette exceeds byte index capacity",
            )
        })?;
        let exact = f64::from(influence.weight) / total * 255.;
        if !exact.is_finite() || exact < 0. {
            return Err(Error::new(
                "m3.vertex.skin",
                "influences must be finite and nonnegative",
            ));
        }
        weights[lane] = exact.floor() as u8;
        sum += u16::from(weights[lane]);
        fractions.push((lane, exact - exact.floor()));
    }
    fractions.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    for (lane, _) in fractions.iter().take(usize::from(255 - sum)) {
        weights[*lane] += 1;
    }
    let decoded_total: f32 = weights.iter().map(|x| f32::from(*x)).sum();
    for (lane, influence) in influences.iter().enumerate() {
        context.approximate(
            "m3.vertex.skin",
            (f32::from(weights[lane]) / decoded_total - influence.weight).abs(),
            "packing an influence changes its normalized weight",
        )?;
    }
    Ok((weights, joints))
}
