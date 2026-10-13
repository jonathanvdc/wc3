//! View mappings for effects.
use super::*;
type ActiveGraph = (
    Vec<wr::Node<wr::V19>>,
    Vec<(wr::MdxMaterial, wr::NativeSync)>,
);
pub(in crate::model::conversion::mdlx) fn active_graph(
    asset: &Asset,
    options: &ExportOptions,
) -> Result<ActiveGraph, Error> {
    let raw = asset.document.raw();
    let root = asset.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model = graph(raw, root.models, options.model as usize)?;
    let nodes = wem_view::nodes(raw, model.nodes.nodes)?;
    let sets = raw.resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?;
    let profile = options.profile.unwrap_or(root.default_profile);
    let mut materials = Vec::new();
    for set in sets.iter() {
        let set = set?;
        if set.profile != profile {
            continue;
        }
        for material in raw
            .resolve::<wr::MaterialRecord>(set.materials.cast())?
            .iter()
        {
            let material = material?;
            if let wr::NativeMaterial::Mdx(reference) = material.native {
                let record = raw
                    .resolve::<wr::MdxMaterial>(reference.cast())?
                    .get(0)
                    .transpose()?
                    .ok_or_else(|| Error::new("material.native", "missing MDLX material"))?;
                materials.push((record, material.sync));
            } else {
                return Ok((nodes, Vec::new()));
            }
        }
        break;
    }
    Ok((nodes, materials))
}
