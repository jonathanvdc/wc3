//! Authored geometry LOD and camera-dependent instance selection.
use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use std::collections::HashMap;
use std::iter::once;
use wc3::model::{Model, V1800};

use crate::animation::pose::{
    default_node_camera, inherited_node_camera, Wc3DefaultNodeCamera, Wc3NodeCamera,
};
use crate::materials::layers::AnimatedLayer;
use crate::preparation::PreparedModel;

/// Geometry quality policy for one model root. Without this component, the
/// effective [`Wc3LodSettings::default_lod`] is used.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wc3Lod {
    /// Select an authored level. Missing levels resolve to the nearest coarser
    /// available level, or the coarsest level if none is coarse enough.
    Fixed(u32),
    /// Select from projected model size and quality settings.
    Automatic,
}

impl Default for Wc3Lod {
    fn default() -> Self {
        Self::Fixed(0)
    }
}

/// Global LOD configuration. Use [`Wc3LodOverride`] for a complete local override.
/// Larger authored level numbers mean less detail. Invalid configurations fall
/// back to the built-in defaults; call [`Self::validate`] when accepting user input.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct Wc3LodSettings {
    /// Policy for roots without an explicit [`Wc3Lod`] component.
    pub default_lod: Wc3Lod,
    /// Multiplies projected diameter. Larger values retain more detail.
    pub quality_bias: f32,
    /// Pixel diameters for successive transitions in the sorted available level
    /// list. Must be positive and strictly decreasing. Further transitions halve
    /// the last threshold; an empty list keeps the finest allowed level.
    pub thresholds: Vec<f32>,
    /// Fractional dead band around each threshold, in [0, 1).
    pub hysteresis: f32,
    /// Finest permitted authored level (a detail cap). Applies to fixed and
    /// automatic selection. If unavailable, selects the next coarser level.
    pub minimum_level: u32,
}

/// Complete quality configuration override on one model root.
/// An explicit [`Wc3Lod`] component takes precedence over its default policy.
#[derive(Component, Clone, Debug)]
pub struct Wc3LodOverride(
    /// Settings applied to this instance in place of the global resource.
    pub Wc3LodSettings,
);

impl Default for Wc3LodSettings {
    fn default() -> Self {
        Self {
            default_lod: Wc3Lod::default(),
            quality_bias: 1.0,
            thresholds: vec![240.0, 120.0, 60.0],
            hysteresis: 0.15,
            minimum_level: 0,
        }
    }
}

impl Wc3LodSettings {
    /// Automatic low-quality preset; switches to coarser geometry sooner.
    pub fn low() -> Self {
        Self {
            default_lod: Wc3Lod::Automatic,
            quality_bias: 0.5,
            ..Self::default()
        }
    }
    /// Automatic medium-quality preset.
    pub fn medium() -> Self {
        Self {
            default_lod: Wc3Lod::Automatic,
            ..Self::default()
        }
    }
    /// Automatic high-quality preset; retains detailed geometry longer.
    pub fn high() -> Self {
        Self {
            default_lod: Wc3Lod::Automatic,
            quality_bias: 2.0,
            ..Self::default()
        }
    }

    /// Checks that quality bias, hysteresis, and transition thresholds are valid.
    ///
    /// Returns the first invalid setting's explanation without changing the
    /// configuration. Authored level availability is resolved per model later.
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.quality_bias.is_finite() || self.quality_bias <= 0.0 {
            return Err("quality_bias must be finite and positive");
        }
        if !self.hysteresis.is_finite() || !(0.0..1.0).contains(&self.hysteresis) {
            return Err("hysteresis must be finite and in [0, 1)");
        }
        if self
            .thresholds
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
            || self.thresholds.windows(2).any(|pair| pair[0] <= pair[1])
        {
            return Err("thresholds must be finite, positive, and strictly decreasing");
        }
        Ok(())
    }

    fn threshold(&self, index: usize) -> Option<f32> {
        self.thresholds.get(index).copied().or_else(|| {
            self.thresholds
                .last()
                .map(|last| last * 0.5f32.powf((index + 1 - self.thresholds.len()) as f32))
        })
    }
}

/// Runtime geometry selection, inserted once a model is prepared and spawned.
#[derive(Component, Debug)]
pub struct Wc3LodState {
    levels: Vec<u32>,
    selected: u32,
    automatic: bool,
    bounds: LodBounds,
}

impl Wc3LodState {
    pub(crate) fn new(prepared: &PreparedModel) -> Self {
        Self {
            levels: prepared.lod_levels.clone(),
            selected: prepared.lod_levels[0],
            automatic: false,
            bounds: prepared.lod_bounds,
        }
    }
    /// Currently selected authored level. Common geosets remain visible.
    pub fn selected_level(&self) -> u32 {
        self.selected
    }
    /// Sorted drawable levels. Common-only models expose level zero.
    pub fn available_levels(&self) -> &[u32] {
        &self.levels
    }
}

#[derive(Component)]
pub(crate) struct LodGroup {
    pub(crate) root: Entity,
    /// None is common geometry, visible at every level.
    pub(crate) level: Option<u32>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LodBounds {
    center: Vec3,
    radius: f32,
}

impl LodBounds {
    pub(crate) fn from_model(model: &Model<V1800>) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        let mut include = |point: Vec3| {
            if point.is_finite() {
                min = min.min(point);
                max = max.max(point);
            }
        };
        let mut radius = 0.0f32;
        for geoset in model.geosets() {
            for &vertex in geoset.vertices() {
                include(Vec3::from_array(vertex));
            }
            for extent in once(&geoset.extent).chain(&geoset.sequence_extents) {
                include(Vec3::from_array(extent.minimum));
                include(Vec3::from_array(extent.maximum));
                if extent.bounds_radius.is_finite() {
                    radius = radius.max(extent.bounds_radius);
                }
            }
        }
        if let Some(info) = model.model_info() {
            include(Vec3::from_array(info.minimum_extent));
            include(Vec3::from_array(info.maximum_extent));
            if info.bounds_radius.is_finite() {
                radius = radius.max(info.bounds_radius);
            }
        }
        for sequence in model.sequences() {
            include(Vec3::from_array(sequence.extent.minimum));
            include(Vec3::from_array(sequence.extent.maximum));
            if sequence.extent.bounds_radius.is_finite() {
                radius = radius.max(sequence.extent.bounds_radius);
            }
        }
        if !min.is_finite() || !max.is_finite() {
            return Self {
                center: Vec3::ZERO,
                radius: radius.max(1.0),
            };
        }
        Self {
            center: (min + max) * 0.5,
            radius: (radius + ((min + max) * 0.5).length()).max((max - min).length() * 0.5),
        }
    }

    fn world(self, transform: &GlobalTransform) -> Self {
        let affine = transform.affine();
        // The maximum absolute row sum of the Gram matrix bounds its largest
        // eigenvalue: exact for orthogonal scales and conservative under shear.
        let axes = [
            Vec3::from(affine.matrix3.x_axis),
            Vec3::from(affine.matrix3.y_axis),
            Vec3::from(affine.matrix3.z_axis),
        ];
        let scale = axes
            .iter()
            .map(|axis| axes.iter().map(|other| axis.dot(*other).abs()).sum::<f32>())
            .fold(0.0f32, f32::max)
            .sqrt();
        Self {
            center: transform.transform_point(self.center),
            radius: self.radius * scale,
        }
    }
}

fn projected_diameter(
    bounds: LodBounds,
    camera: &GlobalTransform,
    projection: &Projection,
    height: f32,
) -> Option<f32> {
    if !bounds.center.is_finite() || !bounds.radius.is_finite() || height <= 0.0 {
        return None;
    }
    let bounds = bounds.world(&GlobalTransform::from(camera.affine().inverse()));
    if !bounds.center.is_finite() || !bounds.radius.is_finite() {
        return None;
    }
    let diameter = match projection {
        Projection::Perspective(perspective) => {
            let depth = -bounds.center.z;
            if depth + bounds.radius <= 0.0 {
                return None;
            }
            if depth - bounds.radius <= perspective.near {
                return Some(f32::INFINITY);
            }
            2.0 * bounds.radius * height
                / (2.0 * (perspective.fov * 0.5).tan() * (depth - bounds.radius))
        }
        Projection::Orthographic(_) => {
            bounds.radius * projection.get_clip_from_view().y_axis.y.abs() * height
        }
        // Unknown projection: keep finest geometry rather than underestimate.
        Projection::Custom(_) => return Some(f32::INFINITY),
    };
    (!diameter.is_nan()).then_some(diameter)
}

fn resolve_level(levels: &[u32], requested: u32) -> usize {
    levels
        .iter()
        .position(|&level| level >= requested)
        .unwrap_or(levels.len() - 1)
}

fn select_level(
    levels: &[u32],
    policy: Wc3Lod,
    settings: &Wc3LodSettings,
    pixels: Option<f32>,
    previous: Option<u32>,
) -> u32 {
    let finest = resolve_level(levels, settings.minimum_level);
    if let Wc3Lod::Fixed(level) = policy {
        return levels[resolve_level(levels, level.max(settings.minimum_level))];
    }
    let Some(pixels) = pixels else {
        return levels[finest];
    };
    let size = pixels * settings.quality_bias;
    let mut index = previous
        .and_then(|level| levels.iter().position(|&value| value == level))
        .unwrap_or(finest)
        .max(finest);
    let hysteresis = if previous.is_some() {
        settings.hysteresis
    } else {
        0.0
    };
    while index > finest
        && settings
            .threshold(index - 1)
            .is_some_and(|threshold| size >= threshold * (1.0 + hysteresis))
    {
        index -= 1;
    }
    while index + 1 < levels.len()
        && settings
            .threshold(index)
            .is_some_and(|threshold| size < threshold * (1.0 - hysteresis))
    {
        index += 1;
    }
    if settings.thresholds.is_empty() {
        index = finest;
    }
    levels[index]
}

type LodInstances<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        Option<&'static Wc3Lod>,
        Option<&'static Wc3LodOverride>,
        &'static mut Wc3LodState,
    ),
>;
type LodCameras<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Camera,
        &'static GlobalTransform,
        &'static Projection,
        Option<&'static RenderTarget>,
        Has<Wc3DefaultNodeCamera>,
        Option<&'static RenderLayers>,
    ),
    With<Camera3d>,
>;

pub(crate) fn update_lod(
    settings: Res<Wc3LodSettings>,
    mut instances: LodInstances,
    cameras: LodCameras,
    ancestors: Query<(Option<&ChildOf>, Option<&Wc3NodeCamera>)>,
    mut groups: Query<(&LodGroup, &mut Visibility)>,
    passes: Query<(&AnimatedLayer, Option<&RenderLayers>)>,
) {
    let fallback = Wc3LodSettings::default();
    let default_layers = RenderLayers::default();
    let mut model_layers = HashMap::<Entity, RenderLayers>::new();
    for (pass, layers) in &passes {
        let entry = model_layers
            .entry(pass.root)
            .or_insert_with(RenderLayers::none);
        for layer in layers.unwrap_or(&default_layers).iter() {
            *entry = entry.clone().with(layer);
        }
    }
    let active: Vec<_> = cameras
        .iter()
        .filter(|(_, camera, _, _, _, _, _)| camera.is_active)
        .collect();
    let default_camera =
        default_node_camera(active.iter().map(|(entity, _, _, _, target, marked, _)| {
            (
                *entity,
                target.is_none_or(|target| matches!(target, RenderTarget::Window(_))),
                *marked,
            )
        }));
    for (root, transform, policy, local_settings, mut state) in &mut instances {
        let settings = local_settings
            .map(|settings| &settings.0)
            .unwrap_or(&settings);
        let settings = if settings.validate().is_ok() {
            settings
        } else {
            &fallback
        };
        let policy = policy.copied().unwrap_or(settings.default_lod);
        let camera = inherited_node_camera(root, |entity| {
            let (parent, selection) = ancestors.get(entity).ok()?;
            Some((
                parent.map(ChildOf::parent),
                selection.map(|selection| selection.0),
            ))
        })
        .or(default_camera);
        let pixels = if policy == Wc3Lod::Automatic {
            let bounds = state.bounds.world(transform);
            active
                .iter()
                .filter(|(entity, _, _, _, _, _, layers)| {
                    camera.is_none_or(|selected| selected == *entity)
                        && model_layers.get(&root).is_some_and(|model_layers| {
                            layers.unwrap_or(&default_layers).intersects(model_layers)
                        })
                })
                .filter_map(|(_, camera, transform, projection, _, _, _)| {
                    let height = camera.physical_viewport_size()?.y as f32;
                    projected_diameter(bounds, transform, projection, height)
                })
                .reduce(f32::max)
        } else {
            None
        };
        let selected = select_level(
            &state.levels,
            policy,
            settings,
            pixels,
            state.automatic.then_some(state.selected),
        );
        if state.selected != selected {
            state.selected = selected;
        }
        let automatic = policy == Wc3Lod::Automatic;
        if state.automatic != automatic {
            state.automatic = automatic;
        }
    }
    for (group, mut visibility) in &mut groups {
        let Ok((_, _, _, _, state)) = instances.get(group.root) else {
            continue;
        };
        let value = if group.level.is_none_or(|level| level == state.selected) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != value {
            *visibility = value;
        }
    }
}

#[cfg(test)]
#[path = "lod_tests.rs"]
mod tests;
