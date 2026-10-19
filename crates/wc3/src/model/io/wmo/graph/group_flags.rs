//! Physical table declarations are regenerated from their logical owners.
use crate::model::formats::wmo::records::GroupFlags;
use crate::model::profiles::wmo::{Group, GroupOptions};
use crate::model::profiles::Wmo;
use crate::model::Model;

pub(super) fn read(flags: GroupFlags, supplemental: u32) -> GroupOptions {
    GroupOptions {
        exterior: flags.exterior(),
        interior: flags.interior(),
        exterior_lit: flags.exterior_lit(),
        unreachable: flags.unreachable(),
        sky: flags.show_exterior_sky(),
        baked_lighting: flags.has_lightmap(),
        supplemental: supplemental.into(),
    }
}
pub(super) fn write(options: GroupOptions) -> GroupFlags {
    let mut flags = GroupFlags(0);
    flags.set_exterior(options.exterior);
    flags.set_interior(options.interior);
    flags.set_exterior_lit(options.exterior_lit);
    flags.set_unreachable(options.unreachable);
    flags.set_show_exterior_sky(options.sky);
    flags.set_has_lightmap(options.baked_lighting);
    flags
}

pub(super) fn catalog(model: &Model<'_, Wmo>, group: &Group<'_>) -> GroupFlags {
    let mut flags = write(group.options);
    if let Some(instance) = group
        .instances
        .iter()
        .filter_map(|id| model.instance(*id))
        .min_by_key(|instance| instance.lod.unwrap_or(0))
    {
        let mesh = model.mesh(instance.mesh).expect("validated group geometry");
        flags.set_has_bsp(
            mesh.geometry
                .triangle_controls()
                .iter()
                .any(|face| face.spatial_queries),
        );
        flags.set_has_vertex_colors(!mesh.geometry.color_sets().is_empty());
        flags.set_has_lights(!instance.controls.lights.is_empty());
        flags.set_has_doodads(!instance.controls.doodads.is_empty());
        flags.set_has_liquid(
            !instance.controls.liquids.is_empty() || instance.controls.implicit_liquid.is_some(),
        );
        flags.set_lod(instance.lod.is_some_and(|lod| lod != 0));
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_presence_bits_are_not_duplicate_authored_controls() {
        let mut flags = GroupFlags(1 << 31);
        flags.set_has_bsp(true);
        flags.set_has_vertex_colors(true);
        flags.set_has_lights(true);
        flags.set_lod(true);
        flags.set_has_doodads(true);
        flags.set_has_liquid(true);
        flags.set_has_lightmap(true);
        flags.set_exterior(true);
        let options = read(flags, 9);
        assert_eq!(write(options).bits() & (1 << 31), 0);
        assert!(options.baked_lighting);
        assert!(options.exterior);
        assert_eq!(options.supplemental.bits(), 9);
        let rebuilt = write(options);
        assert!(rebuilt.has_lightmap());
        assert!(rebuilt.exterior());
        assert!(!rebuilt.has_bsp());
        assert!(!rebuilt.lod());
    }
}
