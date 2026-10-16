//! Geometry-owned appearance controls, independent of surface material ownership.
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use wc3::model::{
    Animated, MeshId, Model, NodeFeature, Property, PropertyTarget, SampledAnimation,
};

/// Material sharing includes geometry appearance, without separating equal constants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(super) enum MeshMaterialState {
    #[default]
    Plain,
    Constant([u32; 4]),
    Animated(MeshId),
}

#[derive(Clone)]
pub(super) struct MeshAnimation {
    color: Animated<[f32; 3]>,
    opacity: Animated<f32>,
    common_color: Option<Property<[f32; 3]>>,
    common_opacity: Option<Property<f32>>,
    animated: bool,
}
impl MeshAnimation {
    pub(super) fn compile(model: &Model<'_>) -> HashMap<MeshId, Self> {
        let targets: HashSet<PropertyTarget> = model
            .timeline_channels()
            .iter()
            .chain(model.shared_channels())
            .chain(model.clips().flat_map(|(_, clip)| clip.channels()))
            .chain(
                model
                    .controllers()
                    .flat_map(|(_, controller)| &controller.channels),
            )
            .map(|channel| channel.target().clone())
            .collect();
        let mut values = HashMap::new();
        for (_, feature) in model.features() {
            if let NodeFeature::GeometryAnimation(source) = &feature.payload {
                if let Some(mesh) = source.mesh {
                    let color = if source.use_color {
                        source.color.clone()
                    } else {
                        Animated::constant([1.; 3])
                    };
                    let opacity = source.opacity.clone();
                    let animated = color
                        .property
                        .as_ref()
                        .is_some_and(|property| targets.contains(property.target()))
                        || opacity
                            .property
                            .as_ref()
                            .is_some_and(|property| targets.contains(property.target()));
                    values.insert(
                        mesh,
                        Self {
                            color,
                            opacity,
                            common_color: None,
                            common_opacity: None,
                            animated,
                        },
                    );
                }
            }
        }
        for (mesh, _) in model.meshes() {
            let color = mesh.color();
            let opacity = mesh.opacity();
            let common_color = targets.contains(color.target()).then_some(color);
            let common_opacity = targets.contains(opacity.target()).then_some(opacity);
            if common_color.is_some() || common_opacity.is_some() {
                let value = values.entry(mesh).or_insert_with(|| Self {
                    color: Animated::constant([1.; 3]),
                    opacity: Animated::constant(1.),
                    common_color: None,
                    common_opacity: None,
                    animated: false,
                });
                value.common_color = common_color;
                value.common_opacity = common_opacity;
                value.animated = true;
            }
        }
        values
    }
    pub(super) fn default_opacity(&self) -> f32 {
        self.opacity.default
    }
    pub(super) fn is_animated(&self) -> bool {
        self.animated
    }
    pub(super) fn state(&self, mesh: MeshId) -> MeshMaterialState {
        if self.animated {
            return MeshMaterialState::Animated(mesh);
        }
        let rgba = self.defaults();
        if rgba == [1.; 4] {
            MeshMaterialState::Plain
        } else {
            MeshMaterialState::Constant(rgba.map(f32::to_bits))
        }
    }
    fn defaults(&self) -> [f32; 4] {
        [
            self.color.default[0],
            self.color.default[1],
            self.color.default[2],
            self.opacity.default,
        ]
    }
    pub(super) fn sample(&self, sample: &SampledAnimation<'_, '_>) -> [f32; 4] {
        let color = self
            .common_color
            .as_ref()
            .and_then(|property| sample.property(property))
            .unwrap_or_else(|| sample.animated(&self.color));
        let opacity = self
            .common_opacity
            .as_ref()
            .and_then(|property| sample.property(property))
            .unwrap_or_else(|| sample.animated(&self.opacity));
        [color[0], color[1], color[2], opacity]
    }
    pub(super) fn apply_defaults(&self, color: Color) -> Color {
        multiply(color, self.defaults())
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
