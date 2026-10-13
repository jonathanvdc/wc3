//! Translation between common render state and native material controls.
use super::{nr, wr, Error};
use crate::model::conversion::Context;

pub(super) fn blend(value: nr::BlendMode) -> wr::BlendMode {
    match value {
        nr::BlendMode::Opaque => wr::BlendMode::Opaque,
        nr::BlendMode::AlphaBlend => wr::BlendMode::AlphaBlend,
        nr::BlendMode::Add => wr::BlendMode::Additive,
        nr::BlendMode::AlphaAdd => wr::BlendMode::AdditiveAlpha,
        nr::BlendMode::Mod => wr::BlendMode::Modulate,
        nr::BlendMode::Mod2x => wr::BlendMode::Modulate2x,
        nr::BlendMode::Unknown(_) => wr::BlendMode::Opaque,
    }
}
pub(super) fn flags(value: nr::MaterialFlags) -> wr::MaterialFlags {
    let mut flags = wr::MaterialFlags::default();
    flags.set_unlit(value.unshaded());
    flags.set_unfogged(value.unfogged());
    flags.set_no_shadow_cast(value.no_shadows_cast());
    flags.set_no_shadow_receive(value.no_shadows_receive());
    flags
}
pub(super) fn apply(
    material: &mut nr::StandardMaterial<nr::V20>,
    common_blend: wr::BlendMode,
    common_flags: wr::MaterialFlags,
    context: &mut Context,
    path: &str,
) -> Result<(), Error> {
    if common_blend != blend(material.blend_mode) {
        material.blend_mode = match common_blend {
            wr::BlendMode::Opaque => nr::BlendMode::Opaque,
            wr::BlendMode::AlphaBlend => nr::BlendMode::AlphaBlend,
            wr::BlendMode::Additive => nr::BlendMode::Add,
            wr::BlendMode::AdditiveAlpha => nr::BlendMode::AlphaAdd,
            wr::BlendMode::Modulate => nr::BlendMode::Mod,
            wr::BlendMode::Modulate2x => nr::BlendMode::Mod2x,
            other => {
                context.unsupported(
                    format!("{path}.blend"),
                    format!("Common blend {other:?} has no exact M3 material mapping"),
                )?;
                material.blend_mode
            }
        };
    }
    let known = wr::MaterialFlags::UNLIT.bits()
        | wr::MaterialFlags::UNFOGGED.bits()
        | wr::MaterialFlags::NO_SHADOW_CAST.bits()
        | wr::MaterialFlags::NO_SHADOW_RECEIVE.bits();
    if common_flags.bits() & !known != 0 {
        context.unsupported(
            format!("{path}.flags"),
            "Common material controls have no exact M3 shader flag mapping",
        )?;
    }
    material.flags.set_unshaded(common_flags.unlit());
    material.flags.set_unfogged(common_flags.unfogged());
    material
        .flags
        .set_no_shadows_cast(common_flags.no_shadow_cast());
    material
        .flags
        .set_no_shadows_receive(common_flags.no_shadow_receive());
    Ok(())
}
