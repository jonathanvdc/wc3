//! Relationships that can be checked within one root document.
use super::codec::Array;
use super::root::RootChunk;
use super::{Dialect, ReadError, Root, Validate};
use std::collections::BTreeSet;

impl<'data, D: Dialect> Root<'data, D> {
    /// Group count from MOGI, then MGI2, or the declared header when neither exists.
    pub fn group_count(&self) -> usize {
        self.chunks()
            .find_map(|chunk| match chunk {
                RootChunk::Groups(values) => Some(values.len()),
                _ => None,
            })
            .or_else(|| {
                self.chunks().find_map(|chunk| match chunk {
                    RootChunk::Groups2(values) => Some(values.len()),
                    _ => None,
                })
            })
            .unwrap_or(self.header().group_count as usize)
    }
    /// GFID group-file identities in native group-order rows per LOD level.
    pub fn group_file_ids(&self) -> Option<&Array<'data, u32>> {
        self.chunks().find_map(|chunk| match chunk {
            RootChunk::GroupFileIds(values) => Some(values),
            _ => None,
        })
    }
}
impl<D: Dialect> Validate for Root<'_, D> {
    fn validate(&self) -> Result<(), ReadError> {
        self.header().validate()?;
        validate_tables(
            &self.chunks().collect::<Vec<_>>(),
            self.header().group_count as usize,
            self.header().portal_count as usize,
            self.header().doodad_set_count as usize,
            None,
        )
    }
}
/// Validates shared root tables using counts from their actual native header.
/// Alpha callers supply their separately encoded thirty-two-byte light count.
pub(crate) fn validate_tables<D: Dialect>(
    chunks: &[&RootChunk<'_, D>],
    group_count: usize,
    portal_count_declared: usize,
    doodad_set_count: usize,
    light_count: Option<usize>,
) -> Result<(), ReadError> {
    let mut seen = BTreeSet::new();
    for chunk in chunks.iter().copied() {
        if !matches!(chunk, RootChunk::Unknown { .. }) && !seen.insert(chunk.tag()) {
            return Err(ReadError::invalid(0, "duplicate root table"));
        }
        match chunk {
            RootChunk::Header(_) => return Err(ReadError::invalid(0, "root header in payload")),
            RootChunk::SkyboxFileId(value) => value.validate()?,
            RootChunk::Unknown {
                tag: [b'M', b'V', b'E', b'R'] | [b'M', b'O', b'H', b'D'] | [b'M', b'O', b'G', b'P'],
                ..
            } => return Err(ReadError::invalid(0, "reserved root header tag")),
            RootChunk::Unknown { .. } | RootChunk::M3Materials(_) | RootChunk::DetailDoodads(_) => {
            }
            RootChunk::TextureNames(values) => values.validate()?,
            RootChunk::Materials(values) => values.validate()?,
            RootChunk::UvAnimations(values) => values.validate()?,
            RootChunk::MaterialExtensions(values) => values.validate()?,
            RootChunk::M3TextureNames(values) => values.validate()?,
            RootChunk::GroupNames(values) => values.validate()?,
            RootChunk::Groups(values) => values.validate()?,
            RootChunk::Groups2(values) => values.validate()?,
            RootChunk::SkyboxName(values) => values.validate()?,
            RootChunk::PortalVertices(values) => values.validate()?,
            RootChunk::Portals(values) => values.validate()?,
            RootChunk::PortalRefs(values) => values.validate()?,
            RootChunk::PortalExtras(values) => values.validate()?,
            RootChunk::VisibleVertices(values) => values.validate()?,
            RootChunk::VisibleBlocks(values) => values.validate()?,
            RootChunk::Lights(values) => values.validate()?,
            RootChunk::LightExtensions(values) => values.validate()?,
            RootChunk::NewLights(values) => values.validate()?,
            RootChunk::DoodadSets(values) => values.validate()?,
            RootChunk::DoodadNames(values) => values.validate()?,
            RootChunk::DoodadFileIds(values) => values.validate()?,
            RootChunk::Doodads(values) => values.validate()?,
            RootChunk::DoodadColorMultipliers(values) => values.validate()?,
            RootChunk::Fogs(values) => values.validate()?,
            RootChunk::FogBoxes(values) => values.validate()?,
            RootChunk::FogExtras(values) => values.validate()?,
            RootChunk::GlobalAmbients(values) => values.validate()?,
            RootChunk::AmbientVolumes(values) => values.validate()?,
            RootChunk::AmbientBoxes(values) => values.validate()?,
            RootChunk::ParticulateVolumes(values) => values.validate()?,
            RootChunk::ConvexPlanes(values) => values.validate()?,
            RootChunk::GroupFileIds(values) => values.validate()?,
        }
    }
    macro_rules! table {
        ($variant:ident) => {
            chunks.iter().find_map(|chunk| match chunk {
                RootChunk::$variant(values) => Some(values),
                _ => None,
            })
        };
    }
    let groups = table!(Groups);
    let groups2 = table!(Groups2);
    let portals = table!(Portals);
    let vertices = table!(PortalVertices);
    let portal_refs = table!(PortalRefs);
    let portal_extras = table!(PortalExtras);
    let doodads = table!(Doodads);
    let doodad_sets = table!(DoodadSets);
    let group_names = table!(GroupNames);
    let doodad_names = table!(DoodadNames);
    let doodad_ids = table!(DoodadFileIds);
    let textures = table!(TextureNames);
    let materials = table!(Materials);
    if groups.is_some_and(|values| values.len() != group_count)
        || groups2.is_some_and(|values| values.len() != group_count)
    {
        return Err(ReadError::invalid(0, "root group count"));
    }
    if portals.is_some_and(|values| values.len() != portal_count_declared) {
        return Err(ReadError::invalid(0, "root portal count"));
    }
    if doodad_sets.is_some_and(|values| values.len() != doodad_set_count) {
        return Err(ReadError::invalid(0, "root doodad set count"));
    }
    if let Some(groups) = groups {
        for record in groups.iter() {
            let record = record?;
            if record.name_offset != -1 {
                let offset = usize::try_from(record.name_offset)
                    .map_err(|_| ReadError::invalid(0, "group name offset"))?;
                string(
                    group_names.map_or(&[], |bytes| bytes.as_bytes()),
                    offset,
                    "group name offset",
                )?;
            }
        }
    }
    let vertex_count = vertices.map_or(0, |values| values.len());
    let portal_count = portals.map_or(0, |values| values.len());
    if let Some(portals) = portals {
        for portal in portals.iter() {
            let portal = portal?;
            span(
                portal.start_vertex as usize,
                portal.vertex_count as usize,
                vertex_count,
                "portal vertex span",
            )?;
        }
    }
    if let Some(references) = portal_refs {
        for reference in references.iter() {
            let reference = reference?;
            index(
                reference.portal_index as usize,
                portal_count,
                "portal reference index",
            )?;
            index(
                reference.group_index as usize,
                group_count,
                "portal group index",
            )?;
        }
    }
    if let Some(extras) = portal_extras {
        for extra in extras.iter() {
            index(
                extra?.portal_index as usize,
                portal_count,
                "portal extra index",
            )?;
        }
    }
    if let Some(blocks) = table!(VisibleBlocks) {
        let count = table!(VisibleVertices).map_or(0, |values| values.len());
        for block in blocks.iter() {
            let block = block?;
            span(
                block.first_vertex as usize,
                block.count as usize,
                count,
                "visible block span",
            )?;
        }
    }
    let doodad_count = doodads.map_or(0, |values| values.len());
    if let Some(sets) = doodad_sets {
        for set in sets.iter() {
            let set = set?;
            span(
                set.start_index as usize,
                set.count as usize,
                doodad_count,
                "doodad set span",
            )?;
        }
    }
    if let Some(doodads) = doodads {
        for doodad in doodads.iter() {
            let doodad = doodad?;
            if let Some(names) = doodad_names {
                string(
                    names.as_bytes(),
                    doodad.name_index() as usize,
                    "doodad name offset",
                )?;
            } else {
                index(
                    doodad.name_index() as usize,
                    doodad_ids.map_or(0, |values| values.len()),
                    "doodad model index",
                )?;
            }
            if doodad.flags() & 0x4 != 0 {
                index(
                    doodad.color[3] as usize,
                    light_count.unwrap_or_else(|| table!(Lights).map_or(0, |values| values.len())),
                    "doodad light index",
                )?;
            }
        }
    }
    if let (Some(textures), Some(materials)) = (textures, materials) {
        for material in materials.iter() {
            let material = material?;
            let slots = material.textures();
            let count = material.texture_slots();
            for offset in &slots[..count] {
                string(
                    textures.as_bytes(),
                    *offset as usize,
                    "material texture offset",
                )?;
            }
        }
    }
    if let Some(extensions) = table!(LightExtensions) {
        let count = light_count.unwrap_or_else(|| table!(Lights).map_or(0, |values| values.len()));
        for extension in extensions.iter() {
            index(
                extension?.light_index as usize,
                count,
                "light extension index",
            )?;
        }
    }
    macro_rules! parallel {
        ($variant:ident,$count:expr,$field:literal) => {
            if let Some(values) = table!($variant) {
                if values.len() != $count {
                    return Err(ReadError::invalid(0, $field));
                }
            }
        };
    }
    let material_count = materials.map_or(0, |values| values.len());
    if materials.is_some() {
        parallel!(UvAnimations, material_count, "material UV table count");
        parallel!(
            MaterialExtensions,
            material_count,
            "material extension table count"
        );
    }
    parallel!(
        DoodadColorMultipliers,
        doodad_count,
        "doodad color table count"
    );
    parallel!(
        FogExtras,
        table!(Fogs).map_or(0, |values| values.len()),
        "fog extra table count"
    );
    if let Some(ids) = table!(GroupFileIds) {
        let stride = group_count;
        if (stride == 0 && !ids.is_empty()) || (stride != 0 && !ids.len().is_multiple_of(stride)) {
            return Err(ReadError::invalid(0, "group file ID row width"));
        }
    }
    Ok(())
}

fn span(start: usize, count: usize, length: usize, field: &'static str) -> Result<(), ReadError> {
    if start.checked_add(count).is_none_or(|end| end > length) {
        Err(ReadError::invalid(0, field))
    } else {
        Ok(())
    }
}
fn index(value: usize, length: usize, field: &'static str) -> Result<(), ReadError> {
    if value >= length {
        Err(ReadError::invalid(0, field))
    } else {
        Ok(())
    }
}
fn string(bytes: &[u8], offset: usize, field: &'static str) -> Result<(), ReadError> {
    if bytes
        .get(offset..)
        .is_none_or(|suffix| !suffix.contains(&0))
    {
        Err(ReadError::invalid(0, field))
    } else {
        Ok(())
    }
}
