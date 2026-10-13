//! Geometry attributes, indices, and skinning mappings.
use super::*;
pub(super) fn influences<V: ModelVersion>(
    mesh: &native::Geoset<'_, V>,
) -> Result<Vec<Vec<(u32, f32)>>, String> {
    if let Ok(Some(rows)) = mesh.try_skin_weights() {
        return Ok(rows
            .iter()
            .map(|row| {
                row.bone_indices()
                    .into_iter()
                    .zip(row.weights())
                    .filter(|(_, weight)| *weight != 0)
                    .map(|(bone, weight)| (u32::from(bone), f32::from(weight) / 255.0))
                    .collect()
            })
            .collect());
    }
    let mut groups = Vec::new();
    let indices: Vec<_> = mesh.matrix_indices().iter().collect();
    let mut offset = 0usize;
    for size in mesh.matrix_group_sizes().iter() {
        let end = offset
            .checked_add(size as usize)
            .ok_or("matrix-group length overflow")?;
        let group = indices
            .get(offset..end)
            .ok_or("matrix-group exceeds matrix table")?;
        groups.push(
            group
                .iter()
                .copied()
                .map(|id| (id, 1.0 / size as f32))
                .collect::<Vec<_>>(),
        );
        offset = end;
    }
    if offset != indices.len() {
        return Err("matrix table has unclaimed entries".into());
    }
    mesh.vertex_groups()
        .iter()
        .map(|index| {
            groups
                .get(usize::from(index))
                .cloned()
                .ok_or_else(|| "vertex matrix-group index is out of range".into())
        })
        .collect()
}

pub(super) fn extent(value: native::GeosetExtent) -> wem::records::Extent {
    wem::records::Extent {
        minimum: value.minimum,
        maximum: value.maximum,
        sphere_radius: value.bounds_radius,
    }
}

pub(super) fn native_extent(value: wem::records::Extent) -> native::GeosetExtent {
    native::GeosetExtent {
        minimum: value.minimum,
        maximum: value.maximum,
        bounds_radius: value.sphere_radius,
    }
}
