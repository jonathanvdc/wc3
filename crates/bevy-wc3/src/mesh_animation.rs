//! Bevy material sharing and transport for shared geometry appearance evaluation.
use bevy::prelude::*;
pub(super) use wc3::model::MeshAppearance as MeshAnimation;
use wc3::model::MeshId;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(super) enum MeshMaterialState {
    #[default]
    Plain,
    Constant([u32; 4]),
    Animated(MeshId),
}
pub(super) fn state(value: &MeshAnimation, mesh: MeshId) -> MeshMaterialState {
    if value.is_animated() {
        MeshMaterialState::Animated(mesh)
    } else {
        let rgba = value.defaults();
        if rgba == [1.; 4] {
            MeshMaterialState::Plain
        } else {
            MeshMaterialState::Constant(rgba.map(f32::to_bits))
        }
    }
}
pub(super) fn multiply(color: Color, tint: [f32; 4]) -> Color {
    let color = color.to_linear();
    Color::linear_rgba(
        color.red * tint[0],
        color.green * tint[1],
        color.blue * tint[2],
        color.alpha * tint[3],
    )
}

#[cfg(test)]
#[path = "mesh_animation_tests.rs"]
mod tests;
