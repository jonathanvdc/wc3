//! Liquid grid dimensions, surface heights, flow/UV union fields, and tile masks.
use super::*;
use native::group::{Liquid, LiquidHeader};
pub(super) fn fields<D: Dialect>(
    group: &Group<'_, D>,
) -> Result<Vec<(String, i64, String)>, Error> {
    let mut output = Vec::new();
    for chunk in group.chunks() {
        if let GroupChunk::Liquid(liquid) = chunk {
            if !output.is_empty() {
                return Err(Error::new(
                    "liquid",
                    "multiple liquid grids require independent grid selection",
                ));
            }
            output.push((
                "wmo.liquid.material".into(),
                i64::from(liquid.header.material),
                String::new(),
            ));
            output.push((
                "wmo.liquid.vertex_dimensions".into(),
                0,
                metadata::words(liquid.header.vertex_dimensions.map(|v| v as u32)),
            ));
            output.push((
                "wmo.liquid.tile_dimensions".into(),
                0,
                metadata::words(liquid.header.tile_dimensions.map(|v| v as u32)),
            ));
            output.push((
                "wmo.liquid.corner".into(),
                0,
                metadata::floats(liquid.header.corner),
            ));
            let rows = collect(
                &Array::<LiquidVertex>::decode(liquid.vertices.as_bytes())
                    .map_err(|e| failure("liquid", e))?,
                "liquid",
            )?;
            output.push((
                "wmo.liquid.heights".into(),
                0,
                metadata::floats(rows.iter().map(|v| v.height)),
            ));
            // These four union bytes are native values, independent of the source document layout.
            output.push((
                "wmo.liquid.vertex_union".into(),
                0,
                metadata::words(rows.iter().map(|v| u32::from_le_bytes(v.data))),
            ));
            output.push((
                "wmo.liquid.tiles".into(),
                0,
                metadata::words(
                    collect(&liquid.tiles, "liquid.tiles")?
                        .into_iter()
                        .map(u32::from),
                ),
            ));
            let kind = match &liquid.vertices {
                LiquidVertices::Unresolved(_) => 0,
                LiquidVertices::Water(_) => 1,
                LiquidVertices::Magma(_) => 2,
            };
            output.push(("wmo.liquid.interpretation".into(), kind, String::new()));
        }
    }
    Ok(output)
}
pub(super) fn restore<D: Dialect>(
    group: &mut Group<'static, D>,
    fields: &BTreeMap<String, (i64, String)>,
) -> Result<(), Error> {
    if !fields.contains_key("wmo.liquid.material") {
        return Ok(());
    }
    if group
        .chunks()
        .iter()
        .any(|chunk| matches!(chunk, GroupChunk::Liquid(_)))
    {
        return Ok(());
    }
    let words = |name: &str| {
        metadata::parse_words(fields.get(name).map(|v| v.1.as_str()).unwrap_or_default())
    };
    let floats = |name: &str| {
        metadata::parse_floats(fields.get(name).map(|v| v.1.as_str()).unwrap_or_default())
    };
    let dimensions = |name: &str| -> Result<[i32; 2], Error> {
        let rows = words(name)?;
        let values: [u32; 2] = rows
            .try_into()
            .map_err(|_| Error::new(name, "liquid dimensions require two components"))?;
        Ok(values.map(|v| v as i32))
    };
    let heights = floats("wmo.liquid.heights")?;
    let data = words("wmo.liquid.vertex_union")?;
    if heights.len() != data.len() {
        return Err(Error::new(
            "liquid.vertices",
            "heights and flow/UV values have different counts",
        ));
    }
    let rows = heights
        .into_iter()
        .zip(data)
        .map(|(height, data)| LiquidVertex {
            height,
            data: data.to_le_bytes(),
        })
        .collect::<Vec<_>>();
    let tiles = words("wmo.liquid.tiles")?
        .into_iter()
        .map(|v| u8::try_from(v).map_err(|e| failure("liquid.tile", e)))
        .collect::<Result<Vec<_>, _>>()?;
    let corner = floats("wmo.liquid.corner")?
        .try_into()
        .map_err(|_| Error::new("liquid.corner", "liquid origin requires three components"))?;
    let vertices = array(&rows)?;
    let vertices = match metadata::number(fields, "wmo.liquid.interpretation", 0)? {
        0 => LiquidVertices::Unresolved(vertices),
        1 => LiquidVertices::Water(
            Array::<native::group::WaterVertex>::decode(vertices.as_bytes())
                .map_err(|e| failure("liquid.water", e))?
                .into_owned(),
        ),
        2 => LiquidVertices::Magma(
            Array::<native::group::MagmaVertex>::decode(vertices.as_bytes())
                .map_err(|e| failure("liquid.magma", e))?
                .into_owned(),
        ),
        _ => {
            return Err(Error::new(
                "liquid.interpretation",
                "unknown water/magma interpretation",
            ));
        }
    };
    let liquid = Liquid {
        header: LiquidHeader {
            vertex_dimensions: dimensions("wmo.liquid.vertex_dimensions")?,
            tile_dimensions: dimensions("wmo.liquid.tile_dimensions")?,
            corner,
            material: u16::try_from(metadata::number(fields, "wmo.liquid.material", 0)?)
                .map_err(|e| failure("liquid.material", e))?,
        },
        vertices,
        tiles: array(&tiles)?,
    };
    liquid.encode().map_err(|e| failure("liquid", e))?;
    group.chunks_mut().push(GroupChunk::Liquid(liquid));
    Ok(())
}
