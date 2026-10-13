//! Common geometry encoded into typed native draw buffers.
use super::*;
pub(in crate::model::conversion::wmo) fn export<D: Dialect>(
    mesh: &s::Mesh<'_>,
    flags: &[u32],
    context: &mut Context,
    path: &str,
) -> Result<Group<'static, D>, Error> {
    let mut group = Group::<D>::new();
    group.header_mut().bounds_minimum = mesh.bounds.minimum;
    group.header_mut().bounds_maximum = mesh.bounds.maximum;
    let indices = mesh.triangles.iter().flatten().copied().collect::<Vec<_>>();
    if mesh.positions.len() > 65536 {
        group
            .chunks_mut()
            .push(GroupChunk::Indices32(array(&indices)?));
    } else {
        let indices = indices
            .into_iter()
            .map(|index| u16::try_from(index).map_err(|error| failure(path, error)))
            .collect::<Result<Vec<_>, _>>()?;
        group
            .chunks_mut()
            .push(GroupChunk::Indices(array(&indices)?));
    }
    group
        .chunks_mut()
        .push(GroupChunk::Positions(array(&mesh.positions)?));
    if !mesh.normals.is_empty() {
        group
            .chunks_mut()
            .push(GroupChunk::Normals(array(&mesh.normals)?));
    }
    for uv in &mesh.uv_sets {
        group
            .chunks_mut()
            .push(GroupChunk::TextureCoordinates(array(uv)?));
    }
    if !mesh.influences.is_empty() {
        context.unsupported(format!("{path}.skin"), "WMO geometry cannot be skinned")?;
    }
    if !mesh.colors.is_empty() {
        let mut colors = Vec::new();
        for (index, color) in mesh.colors.iter().enumerate() {
            let mut value = [0u8; 4];
            for (channel, source) in [color[2], color[1], color[0], color[3]]
                .into_iter()
                .enumerate()
            {
                if !source.is_finite() || !(0.0..=1.0).contains(&source) {
                    return Err(Error::new(
                        format!("{path}.colors[{index}]"),
                        "color component outside normalized range",
                    ));
                }
                let byte = (source * 255.).round() as u8;
                context.approximate(
                    format!("{path}.colors[{index}][{channel}]"),
                    (source - f32::from(byte) / 255.).abs(),
                    "Vertex color quantized to one byte",
                )?;
                value[channel] = byte;
            }
            colors.push(value);
        }
        group
            .chunks_mut()
            .push(GroupChunk::VertexColors(array(&colors)?));
    }
    let material = if mesh.material_slot == u32::MAX {
        255
    } else {
        mesh.material_slot
    };
    if material > 65535 {
        return Err(Error::new(path, "WMO material index exceeds sixteen bits"));
    }
    if material > 254 || flags.iter().any(|flag| *flag > 255) {
        let rows = mesh
            .triangles
            .iter()
            .enumerate()
            .map(|(index, _)| nr::Triangle2 {
                flags: nr::TriangleFlags2(flags.get(index).copied().unwrap_or(0) as u16),
                material: material as u16,
            })
            .collect::<Vec<_>>();
        group
            .chunks_mut()
            .push(GroupChunk::Triangles2(array(&rows)?));
    } else {
        let rows = mesh
            .triangles
            .iter()
            .enumerate()
            .map(|(index, _)| nr::Triangle {
                flags: nr::TriangleFlags(flags.get(index).copied().unwrap_or(0) as u8),
                material: material as u8,
            })
            .collect::<Vec<_>>();
        group
            .chunks_mut()
            .push(GroupChunk::Triangles(array(&rows)?));
    }
    if material != 255 && !mesh.triangles.is_empty() {
        let count = mesh
            .triangles
            .len()
            .checked_mul(3)
            .ok_or_else(|| Error::new(path, "index count overflow"))?;
        let mut batches = Vec::new();
        for start in (0..count).step_by(65535) {
            let length = (count - start).min(65535);
            let mut bytes = [0u8; 12];
            let mut probe = [0u8; 12];
            probe[0] = 1;
            let extended = Decoder::new(&probe)
                .read::<D::BatchPrefix>()
                .map_err(|error| failure(path, error))?
                .vertex_range(nr::BatchFlags(4), 0, 0)
                .0
                == 65536;
            let maximum = mesh.positions.len().saturating_sub(1) as u32;
            let mut batch_flags = nr::BatchFlags::default();
            batch_flags.set_large_material(material > 255);
            if extended {
                bytes[10..12].copy_from_slice(&(material as u16).to_le_bytes());
                if maximum > 65535 {
                    bytes[1] = u8::try_from(maximum >> 16).map_err(|error| failure(path, error))?;
                    batch_flags.set_extended_vertices(true);
                }
            } else {
                for axis in 0..3 {
                    let minimum = mesh
                        .positions
                        .iter()
                        .map(|p| p[axis])
                        .fold(f32::INFINITY, f32::min)
                        .floor();
                    let maximum = mesh
                        .positions
                        .iter()
                        .map(|p| p[axis])
                        .fold(f32::NEG_INFINITY, f32::max)
                        .ceil();
                    if minimum < i16::MIN as f32 || maximum > i16::MAX as f32 {
                        return Err(Error::new(
                            path,
                            "legacy batch culling bounds exceed signed sixteen-bit coordinates",
                        ));
                    }
                    bytes[axis * 2..axis * 2 + 2].copy_from_slice(&(minimum as i16).to_le_bytes());
                    bytes[6 + axis * 2..8 + axis * 2]
                        .copy_from_slice(&(maximum as i16).to_le_bytes());
                }
                if maximum > 65535 {
                    return Err(Error::new(
                        path,
                        "legacy batch vertex range exceeds sixteen bits",
                    ));
                }
            }
            let prefix = Decoder::new(&bytes)
                .read::<D::BatchPrefix>()
                .map_err(|error| failure(path, error))?;
            if material > 255 && prefix.material(nr::BatchFlags(2), 0) != material as u16 {
                return Err(Error::new(
                    path,
                    "selected WMO batch dialect cannot address this material",
                ));
            }
            batches.push(nr::Batch::<D> {
                prefix,
                start_index: start as u32,
                index_count: length as u16,
                minimum_vertex: 0,
                maximum_vertex: maximum as u16,
                flags: batch_flags,
                material: material.min(255) as u8,
            });
        }
        group.header_mut().exterior_batch_count =
            u16::try_from(batches.len()).map_err(|error| failure(path, error))?;
        group
            .chunks_mut()
            .push(GroupChunk::Batches(array(&batches)?));
    }
    Ok(group)
}
