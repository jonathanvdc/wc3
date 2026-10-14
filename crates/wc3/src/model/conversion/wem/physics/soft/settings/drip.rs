use super::super::super::codec::{lookup, mapped, read_vectors, read_words, vectors, words};
use super::super::leaf::{node, node_index, optional, real_records, reals};
use super::{ReadContext, WriteContext};
use crate::model::conversion::Error;
use crate::model::semantic::{
    profiles::wem::physics::soft::{Drip, DripPreset, DripSite, NeckLaw, Strand, SurfacePoint},
    Values,
};
use crate::model::wem::{records, Bool};
use std::array::from_fn;
pub(super) fn read<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::SoftDrip,
) -> Result<Drip<'a>, Error> {
    let meshes = read_words(context.document, value.site_mesh)?;
    let faces = read_words(context.document, value.site_face)?;
    let weights = read_vectors(context.document, value.site_weights)?;
    let thickness = reals(context.document, value.site_thickness)?;
    let len = meshes.len();
    if [faces.len(), weights.len(), thickness.len()]
        .into_iter()
        .any(|n| n != len)
    {
        return Err(Error::new(
            "wem.soft.drip.sites",
            "parallel site streams have incompatible lengths",
        ));
    }
    for id in meshes.iter() {
        lookup(context.meshes, id, "soft drip mesh")?;
    }
    let mesh_targets = context.meshes.to_vec();
    let authored_sites = Values::view(len, move |index| DripSite {
        place: SurfacePoint {
            mesh: mesh_targets[meshes.get(index).expect("checked drip site") as usize],
            face: faces.get(index).expect("checked drip site"),
            weights: weights.get(index).expect("checked drip site"),
            radius: 0.,
        },
        thickness: thickness.get(index).expect("checked drip site"),
    });
    let nodes = read_words(context.document, value.strand_nodes)?;
    let positions = read_vectors(context.document, value.strand_at)?;
    let widths = reals(context.document, value.strand_width)?;
    let drops = read_words(context.document, value.drop_nodes)?;
    let len = positions.len();
    if nodes.len()
        != len
            .checked_mul(3)
            .ok_or_else(|| Error::new("wem.soft.drip.strands", "strand length overflows"))?
        || widths.len() != len
        || drops.len() != len
    {
        return Err(Error::new(
            "wem.soft.drip.strands",
            "parallel strand streams have incompatible lengths",
        ));
    }
    for id in nodes.iter() {
        lookup(context.nodes, id, "soft strand node")?;
    }
    for id in drops.iter() {
        node(id, context.nodes)?;
    }
    let node_targets = context.nodes.to_vec();
    let strands = Values::view(len, move |index| Strand {
        nodes: from_fn(|part| {
            node_targets[nodes.get(index * 3 + part).expect("checked strand") as usize]
        }),
        position: positions.get(index).expect("checked strand"),
        width: widths.get(index).expect("checked strand"),
        drop: match drops.get(index).expect("checked strand") {
            u32::MAX => None,
            id => Some(node_targets[id as usize]),
        },
    });
    Ok(Drip {
        preset: DripPreset::from_raw(value.preset.raw()),
        neck: NeckLaw::from_raw(value.neck.raw()),
        sites_by_hand: value.sites_by_hand.get(),
        sites: value.sites,
        overhang: value.overhang,
        spacing: value.spacing,
        to_ground: value.to_ground.get(),
        length: value.length,
        speed: value.speed,
        thickness: value.thickness,
        neck_width: value.neck_width,
        tip: value.tip,
        stagger: value.stagger,
        sway: value.sway,
        pull: value.pull,
        pinch_every: value.pinch_every,
        material: optional(context.slots, value.material, "soft drip material")?,
        authored_sites,
        strands,
    })
}
pub(super) fn write(
    context: &mut WriteContext<'_, '_>,
    value: &Drip<'_>,
) -> Result<records::SoftDrip, Error> {
    let sites: Vec<_> = value.authored_sites.iter().collect();
    let strands: Vec<_> = value.strands.iter().collect();
    if sites.iter().any(|v| v.place.radius != 0.) {
        return Err(Error::new(
            "wem.soft.drip.radius",
            "drip sites have no radius field",
        ));
    }
    let site_mesh = sites
        .iter()
        .map(|v| mapped(context.meshes, v.place.mesh, "soft drip mesh"))
        .collect::<Result<Vec<_>, _>>()?;
    let strand_nodes = strands
        .iter()
        .flat_map(|v| v.nodes)
        .map(|v| mapped(context.nodes, v, "soft strand node"))
        .collect::<Result<Vec<_>, _>>()?;
    let drop_nodes = strands
        .iter()
        .map(|v| node_index(v.drop, context.nodes))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(records::SoftDrip {
        preset: records::SoftDripPreset::from_raw(value.preset.raw()),
        neck: records::SoftNeckLaw::from_raw(value.neck.raw()),
        sites_by_hand: Bool::from(value.sites_by_hand),
        sites: value.sites,
        overhang: value.overhang,
        spacing: value.spacing,
        to_ground: Bool::from(value.to_ground),
        length: value.length,
        speed: value.speed,
        thickness: value.thickness,
        neck_width: value.neck_width,
        tip: value.tip,
        stagger: value.stagger,
        sway: value.sway,
        pull: value.pull,
        pinch_every: value.pinch_every,
        material: value
            .material
            .map(|v| mapped(context.slots, v, "soft drip material"))
            .transpose()?
            .unwrap_or(u32::MAX),
        site_mesh: words(context.document, site_mesh)?,
        site_face: words(context.document, sites.iter().map(|v| v.place.face))?,
        site_weights: vectors(
            context.document,
            &Values::owned(sites.iter().map(|v| v.place.weights).collect::<Vec<_>>()),
        )?,
        site_thickness: real_records(context.document, sites.iter().map(|v| v.thickness))?,
        strand_nodes: words(context.document, strand_nodes)?,
        strand_at: vectors(
            context.document,
            &Values::owned(strands.iter().map(|v| v.position).collect::<Vec<_>>()),
        )?,
        strand_width: real_records(context.document, strands.iter().map(|v| v.width))?,
        drop_nodes: words(context.document, drop_nodes)?,
    })
}
