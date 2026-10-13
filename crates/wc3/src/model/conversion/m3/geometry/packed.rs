//! Native packed vertex decoding with explicit layout checks.
use crate::model::m3 as native;
use native::Envelope;
#[derive(Clone, Debug)]
pub(super) struct PackedVertex {
    pub(super) position: [f32; 3],
    pub(super) normal: [f32; 3],
    pub(super) tangent: [f32; 4],
    pub(super) uv: [Option<[f32; 2]>; 5],
    pub(super) color: Option<[f32; 4]>,
    pub(super) weights: [u8; 4],
    pub(super) bones: [u8; 4],
}
pub(super) fn packed_vertices<E: Envelope>(
    model: &native::Model<'_, E>,
) -> Result<Vec<PackedVertex>, native::ReadError> {
    let root = model.root_record()?;
    let view = model.vertices()?;
    let count = view
        .vertex_count()
        .ok_or_else(|| native::ReadError::invalid(0, "unresolved packed vertex layout"))?;
    if count == 0 {
        return Ok(Vec::new());
    }
    let bytes = view.as_bytes();
    let stride = bytes.len() / count;
    let color = root.vertex_flags().color();
    let uv_mask = ((root.vertex_flags().bits() >> 17) & 15) as u8
        | (((root.vertex_flags().bits() >> 30) & 1) as u8) << 4;
    let mut output = Vec::with_capacity(count);
    for row in bytes.chunks_exact(stride) {
        let position = [0, 4, 8].map(|at| f32::from_le_bytes(row[at..at + 4].try_into().unwrap()));
        let mut at = 24 + usize::from(color) * 4;
        let mut uv = [None; 5];
        for (channel, value) in uv.iter_mut().enumerate() {
            if uv_mask & (1 << channel) != 0 {
                *value = Some([0, 2].map(|offset| {
                    f32::from(i16::from_le_bytes(
                        row[at + offset..at + offset + 2].try_into().unwrap(),
                    )) / 2048.0
                }));
                at += 4;
            }
        }
        output.push(PackedVertex {
            position,
            normal: [20, 21, 22].map(|at| f32::from(row[at]) / 255.0 * 2.0 - 1.0),
            tangent: [stride - 4, stride - 3, stride - 2, stride - 1]
                .map(|at| f32::from(row[at]) / 255.0 * 2.0 - 1.0),
            uv,
            color: color.then(|| [26, 25, 24, 27].map(|at| f32::from(row[at]) / 255.0)),
            weights: row[12..16].try_into().unwrap(),
            bones: row[16..20].try_into().unwrap(),
        });
    }
    Ok(output)
}
