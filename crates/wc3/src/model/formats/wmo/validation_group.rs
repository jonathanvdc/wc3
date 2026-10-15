//! Geometry, collision and root-reference relationships for native group documents.
use super::codec::Array;
use super::group::{GroupChunk, Liquid};
use super::root::RootChunk;
use super::{Bundle, Dialect, Group, ReadError, RecordLayout, Validate};
use std::collections::BTreeSet;

fn span(first: usize, count: usize, length: usize, field: &'static str) -> Result<(), ReadError> {
    if first.checked_add(count).is_none_or(|end| end > length) {
        Err(ReadError::invalid(0, field))
    } else {
        Ok(())
    }
}
fn references<T>(values: &Array<'_, T>, limit: usize, field: &'static str) -> Result<(), ReadError>
where
    T: RecordLayout + Into<usize>,
    for<'view> T: super::Read<'view>,
{
    for value in values.iter() {
        if value?.into() >= limit {
            return Err(ReadError::invalid(0, field));
        }
    }
    Ok(())
}
impl<D: Dialect> Validate for Group<'_, D> {
    fn validate(&self) -> Result<(), ReadError> {
        self.header().validate()?;
        let chunks = self.chunks();
        let mut seen = BTreeSet::new();
        for chunk in chunks {
            if !matches!(
                chunk,
                GroupChunk::Unknown { .. }
                    | GroupChunk::TextureCoordinates(_)
                    | GroupChunk::VertexColors(_)
            ) && !seen.insert(chunk.tag())
            {
                return Err(ReadError::invalid(0, "duplicate group table"));
            }
            if matches!(&chunk.tag(), b"MVER" | b"MOHD" | b"MOGP") {
                return Err(ReadError::invalid(0, "reserved group header tag"));
            }
        }
        macro_rules! table {
            ($name:ident) => {
                chunks.iter().find_map(|chunk| match chunk {
                    GroupChunk::$name(values) => Some(values),
                    _ => None,
                })
            };
        }
        let positions = table!(Positions);
        let vertices = positions.map_or(0, Array::len);
        let narrow_indices = table!(Indices);
        let wide_indices = table!(Indices32);
        let indices = wide_indices
            .map(Array::len)
            .or_else(|| narrow_indices.map(Array::len))
            .unwrap_or(0);
        if !indices.is_multiple_of(3) {
            return Err(ReadError::invalid(0, "triangle index cardinality"));
        }
        if let Some(values) = narrow_indices {
            references(values, vertices, "triangle vertex index")?;
        }
        if let Some(values) = wide_indices {
            for value in values.iter() {
                if value? as usize >= vertices {
                    return Err(ReadError::invalid(0, "wide triangle vertex index"));
                }
            }
        }
        let triangles = table!(Triangles2)
            .map(Array::len)
            .or_else(|| table!(Triangles).map(Array::len))
            .unwrap_or(indices / 3);
        if triangles != indices / 3 {
            return Err(ReadError::invalid(0, "triangle metadata cardinality"));
        }
        for chunk in chunks {
            match chunk {
                GroupChunk::Normals(values) if values.len() != vertices => {
                    return Err(ReadError::invalid(0, "normal vertex cardinality"));
                }
                GroupChunk::TextureCoordinates(values) if values.len() != vertices => {
                    return Err(ReadError::invalid(0, "texture coordinate cardinality"));
                }
                GroupChunk::VertexColors(values) | GroupChunk::VertexColors2(values)
                    if values.len() != vertices =>
                {
                    return Err(ReadError::invalid(0, "vertex color cardinality"));
                }
                GroupChunk::Batches(values) => {
                    for batch in values.iter() {
                        let batch = batch?;
                        span(
                            batch.start_index as usize,
                            batch.index_count as usize,
                            indices,
                            "draw batch index range",
                        )?;
                        let (first, last) = batch.vertex_range();
                        if batch.index_count != 0 && (first > last || last as usize >= vertices) {
                            return Err(ReadError::invalid(0, "draw batch vertex range"));
                        }
                    }
                }
                GroupChunk::ShadowBatches(values) => {
                    for batch in values.iter() {
                        let batch = batch?;
                        let first = usize::try_from(batch.start)
                            .map_err(|_| ReadError::invalid(0, "negative shadow batch start"))?;
                        let count = usize::try_from(batch.count)
                            .map_err(|_| ReadError::invalid(0, "negative shadow batch count"))?;
                        span(first, count, indices, "shadow batch range")?;
                    }
                }
                GroupChunk::Liquid(liquid) => {
                    liquid.validate()?;
                    validate_grid_shape(liquid)?;
                }
                _ => {}
            }
        }
        let batches = table!(Batches).map_or(0, Array::len);
        let header = self.header();
        let declared = usize::from(header.transition_batch_count)
            + usize::from(header.interior_batch_count)
            + usize::from(header.exterior_batch_count);
        if declared > batches {
            return Err(ReadError::invalid(0, "header batch counts"));
        }
        if let Some(overrides) = table!(BatchOverrides) {
            if overrides.len() != batches {
                return Err(ReadError::invalid(0, "strip batch override cardinality"));
            }
            let strips = table!(StripIndices).map_or(0, Array::len);
            for value in overrides.iter() {
                let value = value?;
                span(
                    value.start_index as usize,
                    value.index_count as usize,
                    strips,
                    "strip batch index range",
                )?;
            }
        }
        if let Some(tangents) = table!(Tangents) {
            if tangents.first_tangent.len() != batches {
                return Err(ReadError::invalid(0, "tangent batch map cardinality"));
            }
            for value in tangents.first_tangent.iter() {
                let value = value?;
                if value != u16::MAX && usize::from(value) >= tangents.tangents.len() {
                    return Err(ReadError::invalid(0, "batch tangent index"));
                }
            }
        }
        let bsp_faces = table!(BspFaces);
        if let Some(values) = bsp_faces {
            references(values, triangles, "BSP triangle index")?;
        }
        if let Some(nodes) = table!(BspNodes) {
            let mut children = Vec::with_capacity(nodes.len());
            for node in nodes.iter() {
                let node = node?;
                span(
                    node.first_face as usize,
                    usize::from(node.face_count),
                    bsp_faces.map_or(0, Array::len),
                    "BSP leaf face range",
                )?;
                for child in [node.negative_child, node.positive_child] {
                    if child < -1 || child >= 0 && child as usize >= nodes.len() {
                        return Err(ReadError::invalid(0, "BSP child index"));
                    }
                }
                children.push([node.negative_child, node.positive_child]);
            }
            let mut state = vec![0u8; nodes.len()];
            for start in 0..nodes.len() {
                let mut stack = vec![(start, false)];
                while let Some((index, exit)) = stack.pop() {
                    if exit {
                        state[index] = 2;
                        continue;
                    }
                    match state[index] {
                        1 => return Err(ReadError::invalid(0, "BSP child cycle")),
                        2 => continue,
                        _ => {}
                    }
                    state[index] = 1;
                    stack.push((index, true));
                    for child in children[index].into_iter().rev() {
                        if child >= 0 {
                            stack.push((child as usize, false));
                        }
                    }
                }
            }
        }
        macro_rules! sets {
            ($ranges:ident,$lights:ident) => {
                if let Some(ranges) = table!($ranges) {
                    let count = table!($lights).map_or(0, Array::len);
                    for range in ranges.iter() {
                        let range = range?;
                        span(
                            range.offset as usize,
                            range.count as usize,
                            count,
                            "group light set range",
                        )?;
                    }
                }
            };
        }
        sets!(PointLightSets, PointLights);
        sets!(SpotLightSets, SpotLights);
        sets!(PointAnimationSets, PointLightAnimations);
        sets!(SpotAnimationSets, SpotLightAnimations);
        Ok(())
    }
}
fn validate_grid_shape(liquid: &Liquid<'_>) -> Result<(), ReadError> {
    let vertex = liquid.header.vertex_dimensions;
    let tiles = liquid.header.tile_dimensions;
    if !liquid.vertices.is_empty()
        && (tiles[0].checked_add(1) != Some(vertex[0])
            || tiles[1].checked_add(1) != Some(vertex[1]))
    {
        return Err(ReadError::invalid(0, "liquid tile-to-vertex shape"));
    }
    Ok(())
}
impl<D: Dialect> Bundle<'_, D> {
    /// Checks a group's shared material, placement, portal, fog and lighting references.
    pub fn validate_group_references(&self, group: &Group<'_, D>) -> Result<(), ReadError> {
        let root: Vec<_> = self.root.chunks().collect();
        macro_rules! count {
            ($name:ident) => {
                root.iter()
                    .find_map(|chunk| match chunk {
                        RootChunk::$name(values) => Some(values.len()),
                        _ => None,
                    })
                    .unwrap_or(0)
            };
        }
        let materials = count!(Materials);
        // MOM3 supplies a separately encoded material table whose count is not established.
        let opaque_materials = root
            .iter()
            .any(|chunk| matches!(chunk, RootChunk::M3Materials(_)));
        let material = |index: usize| {
            if !opaque_materials && index >= materials {
                Err(ReadError::invalid(0, "group root material index"))
            } else {
                Ok(())
            }
        };
        let mut fog_refs = false;
        for chunk in group.chunks() {
            match chunk {
                GroupChunk::Triangles(values) => {
                    for value in values.iter() {
                        let value = value?;
                        if value.material != u8::MAX {
                            material(value.material.into())?;
                        }
                    }
                }
                GroupChunk::Triangles2(values) => {
                    for value in values.iter() {
                        let value = value?;
                        if value.material != u16::MAX {
                            material(value.material.into())?;
                        }
                    }
                }
                GroupChunk::Batches(values) => {
                    for value in values.iter() {
                        material(value?.material_index().into())?;
                    }
                }
                GroupChunk::ShadowBatches(values) => {
                    for value in values.iter() {
                        let value = value?;
                        material(
                            if value.flags.large_material() {
                                value.large_material
                            } else {
                                value.material.into()
                            }
                            .into(),
                        )?;
                    }
                }
                GroupChunk::LightReferences(values) => {
                    references(values, count!(Lights), "root light reference")?
                }
                GroupChunk::NewLightReferences(values) => {
                    references(values, count!(NewLights), "root modern light reference")?
                }
                GroupChunk::DoodadReferences(values) => {
                    references(values, count!(Doodads), "root doodad reference")?
                }
                GroupChunk::ParticulateReferences(values) => references(
                    values,
                    count!(ParticulateVolumes),
                    "root particulate reference",
                )?,
                GroupChunk::AmbientVolumeReferences(values) => references(
                    values,
                    count!(AmbientVolumes),
                    "root ambient volume reference",
                )?,
                GroupChunk::AmbientBoxReferences(values) => {
                    references(values, count!(AmbientBoxes), "root ambient box reference")?
                }
                GroupChunk::FogReferences(values) => {
                    fog_refs = true;
                    references(values, count!(Fogs), "root fog reference")?;
                }
                GroupChunk::FogBoxReferences(values) => {
                    references(values, count!(FogBoxes), "root fog box reference")?
                }
                _ => {}
            }
        }
        span(
            group.header().portal_start.into(),
            group.header().portal_count.into(),
            count!(PortalRefs),
            "group root portal reference range",
        )?;
        // An absent fog table means the legacy header slots have no active target.
        let fogs = count!(Fogs);
        if !fog_refs && fogs != 0 {
            for index in group.header().fog_indices {
                if index != u8::MAX && usize::from(index) >= fogs {
                    return Err(ReadError::invalid(0, "group header fog index"));
                }
            }
        }
        Ok(())
    }
}

use super::legacy::{
    Batch14, Batch16, Group14, Group16, LegacyGroupChunk, Root14, Root16, RootChunk14,
};
use super::Classic;

struct LegacyReferenceContext {
    material_count: usize,
    light_count: usize,
    portal_start: i64,
    portal_count: u64,
    fog_indices: [u8; 4],
    name_offsets: [u32; 2],
}
fn validate_legacy_references<B>(
    common: &[&RootChunk<'_, Classic>],
    chunks: &[LegacyGroupChunk<'_, B>],
    context: LegacyReferenceContext,
    batch_material: impl Fn(&B) -> u8,
) -> Result<(), ReadError>
where
    B: RecordLayout + for<'view> super::Read<'view>,
{
    let LegacyReferenceContext {
        material_count,
        light_count,
        portal_start,
        portal_count,
        fog_indices,
        name_offsets,
    } = context;

    macro_rules! count {
        ($name:ident) => {
            common
                .iter()
                .find_map(|chunk| match chunk {
                    RootChunk::$name(v) => Some(v.len()),
                    _ => None,
                })
                .unwrap_or(0)
        };
    }
    let material = |index: usize| {
        if index >= material_count {
            Err(ReadError::invalid(0, "legacy root material index"))
        } else {
            Ok(())
        }
    };
    let mut replacement_fogs = false;
    for chunk in chunks {
        match chunk {
            LegacyGroupChunk::Triangles(values) => {
                for value in values.iter() {
                    let value = value?;
                    if value.material != u8::MAX {
                        material(value.material.into())?;
                    }
                }
            }
            LegacyGroupChunk::Batches(values) => {
                for value in values.iter() {
                    material(batch_material(&value?).into())?;
                }
            }
            LegacyGroupChunk::Common(value) => match value {
                GroupChunk::LightReferences(v) => {
                    references(v, light_count, "legacy root light reference")?
                }
                GroupChunk::DoodadReferences(v) => {
                    references(v, count!(Doodads), "legacy root doodad reference")?
                }
                GroupChunk::NewLightReferences(v) => {
                    references(v, count!(NewLights), "legacy root modern light reference")?
                }
                GroupChunk::ParticulateReferences(v) => references(
                    v,
                    count!(ParticulateVolumes),
                    "legacy root particulate reference",
                )?,
                GroupChunk::AmbientVolumeReferences(v) => references(
                    v,
                    count!(AmbientVolumes),
                    "legacy root ambient volume reference",
                )?,
                GroupChunk::AmbientBoxReferences(v) => {
                    references(v, count!(AmbientBoxes), "legacy root ambient box reference")?
                }
                GroupChunk::FogReferences(v) => {
                    replacement_fogs = true;
                    references(v, count!(Fogs), "legacy root fog reference")?;
                }
                GroupChunk::FogBoxReferences(v) => {
                    references(v, count!(FogBoxes), "legacy root fog box reference")?
                }
                _ => {}
            },
            _ => {}
        }
    }
    if portal_count != 0 {
        let first = usize::try_from(portal_start)
            .map_err(|_| ReadError::invalid(0, "negative legacy portal start"))?;
        let count = usize::try_from(portal_count)
            .map_err(|_| ReadError::invalid(0, "legacy portal count overflow"))?;
        span(first, count, count!(PortalRefs), "legacy root portal range")?;
    }
    let fogs = count!(Fogs);
    if !replacement_fogs && fogs != 0 {
        for index in fog_indices {
            if index != u8::MAX && usize::from(index) >= fogs {
                return Err(ReadError::invalid(0, "legacy header fog index"));
            }
        }
    }
    if let Some(names) = common.iter().find_map(|chunk| match chunk {
        RootChunk::GroupNames(names) => Some(names),
        _ => None,
    }) {
        for offset in name_offsets {
            if offset != u32::MAX {
                let rest = names
                    .as_bytes()
                    .get(offset as usize..)
                    .ok_or_else(|| ReadError::invalid(0, "legacy group name offset"))?;
                if !rest.contains(&0) {
                    return Err(ReadError::invalid(0, "unterminated legacy group name"));
                }
            }
        }
    }
    Ok(())
}
impl Root14<'_> {
    /// Checks an embedded alpha group's references against native root tables.
    pub fn validate_group_references(&self, group: &Group14<'_>) -> Result<(), ReadError> {
        let materials = self
            .chunks()
            .find_map(|chunk| match chunk {
                RootChunk14::Materials(v) => Some(v.len()),
                _ => None,
            })
            .unwrap_or(0);
        let lights = self
            .chunks()
            .find_map(|chunk| match chunk {
                RootChunk14::Lights(v) => Some(v.len()),
                _ => None,
            })
            .unwrap_or(0);
        let common: Vec<_> = self
            .chunks()
            .filter_map(|chunk| match chunk {
                RootChunk14::Common(value) => Some(value),
                _ => None,
            })
            .collect();
        validate_legacy_references(
            &common,
            &group.chunks,
            LegacyReferenceContext {
                material_count: materials,
                light_count: lights,
                portal_start: i64::from(group.header.portal_start),
                portal_count: u64::from(group.header.portal_count),
                fog_indices: group.header.fog_indices,
                name_offsets: [
                    group.header.name_offset,
                    group.header.descriptive_name_offset,
                ],
            },
            |batch: &Batch14| batch.material,
        )
    }
}
impl Root16<'_> {
    /// Checks a standalone beta group's shared references against its owning root.
    pub fn validate_group_references(&self, group: &Group16<'_>) -> Result<(), ReadError> {
        let common: Vec<_> = self.chunks().collect();
        let materials = common
            .iter()
            .find_map(|chunk| match chunk {
                RootChunk::Materials(v) => Some(v.len()),
                _ => None,
            })
            .unwrap_or(0);
        let lights = common
            .iter()
            .find_map(|chunk| match chunk {
                RootChunk::Lights(v) => Some(v.len()),
                _ => None,
            })
            .unwrap_or(0);
        validate_legacy_references(
            &common,
            &group.chunks,
            LegacyReferenceContext {
                material_count: materials,
                light_count: lights,
                portal_start: i64::from(group.header.portal_start),
                portal_count: u64::from(group.header.portal_count),
                fog_indices: group.header.fog_indices,
                name_offsets: [
                    group.header.name_offset,
                    group.header.descriptive_name_offset,
                ],
            },
            |batch: &Batch16| batch.material,
        )
    }
}
