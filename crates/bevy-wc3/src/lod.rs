//! Authored geometry detail selection using projected rest bounds.
use super::{ModelRig, Playback};
use crate::compiled::RuntimeDraw;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// Geometry detail policy for an independently animated model root.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lod {
    /// Select detail from projected model size and quality settings.
    Automatic,
    /// Resolve to the nearest coarser authored level, or the coarsest available.
    Fixed(u32),
}
impl Default for Lod {
    fn default() -> Self {
        Self::Fixed(0)
    }
}
/// Global quality settings; invalid overrides use the built-in defaults.
#[derive(Resource, Clone, Debug)]
pub struct LodSettings {
    /// Policy used when the root has no explicit component.
    pub default_lod: Lod,
    /// Multiplier on projected diameter; larger values retain finer geometry.
    pub quality_bias: f32,
    /// Strictly decreasing pixel thresholds; later thresholds halve the last.
    pub thresholds: Vec<f32>,
    /// Fractional dead band around each threshold, in [0,1).
    pub hysteresis: f32,
    /// Finest permitted authored level.
    pub minimum_level: u32,
}
impl Default for LodSettings {
    fn default() -> Self {
        Self {
            default_lod: Lod::Fixed(0),
            quality_bias: 1.,
            thresholds: vec![240., 120., 60.],
            hysteresis: 0.15,
            minimum_level: 0,
        }
    }
}
impl LodSettings {
    /// Lower detail preset, selecting coarser geometry sooner.
    pub fn low() -> Self {
        Self {
            default_lod: Lod::Automatic,
            quality_bias: 0.5,
            ..Default::default()
        }
    }
    /// Automatic selection at the default quality scale.
    pub fn medium() -> Self {
        Self {
            default_lod: Lod::Automatic,
            ..Default::default()
        }
    }
    /// Higher detail preset, retaining finer geometry longer.
    pub fn high() -> Self {
        Self {
            default_lod: Lod::Automatic,
            quality_bias: 2.,
            ..Default::default()
        }
    }
    /// Validate quality, dead band, and threshold ordering.
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.quality_bias.is_finite() || self.quality_bias <= 0. {
            return Err("quality bias must be finite and positive");
        }
        if !self.hysteresis.is_finite() || !(0. ..1.).contains(&self.hysteresis) {
            return Err("hysteresis must be in [0,1)");
        }
        if self.thresholds.iter().any(|x| !x.is_finite() || *x <= 0.)
            || self.thresholds.windows(2).any(|x| x[0] <= x[1])
        {
            return Err("thresholds must be finite positive and strictly decreasing");
        }
        Ok(())
    }
    fn threshold(&self, index: usize) -> Option<f32> {
        self.thresholds.get(index).copied().or_else(|| {
            self.thresholds.last().map(|last| {
                last * 2f32.powi(-((index + 1 - self.thresholds.len()).min(126) as i32))
            })
        })
    }
}
/// Complete per-root quality settings override.
#[derive(Component, Clone, Debug)]
pub struct LodOverride(pub LodSettings);
/// Most recent effective detail selection.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct LodState {
    selected: u32,
    automatic: bool,
}
impl LodState {
    /// Authored level currently selected for rendering.
    pub fn selected_level(&self) -> u32 {
        self.selected
    }
}
#[derive(Default)]
pub(super) struct LodDefinition {
    levels: Vec<u32>,
}
impl LodDefinition {
    pub fn compile<M>(draws: &[RuntimeDraw<M>], world_levels: impl Iterator<Item = u32>) -> Self {
        let mut levels: Vec<_> = draws
            .iter()
            .filter_map(|draw| draw.lod)
            .chain(world_levels)
            .collect();
        if levels.is_empty() {
            levels.push(0);
        }
        levels.sort_unstable();
        levels.dedup();
        Self { levels }
    }
}
fn resolve(levels: &[u32], requested: u32) -> usize {
    levels
        .iter()
        .position(|level| *level >= requested)
        .unwrap_or(levels.len() - 1)
}
fn select(
    levels: &[u32],
    policy: Lod,
    settings: &LodSettings,
    pixels: Option<f32>,
    previous: Option<u32>,
) -> u32 {
    let finest = resolve(levels, settings.minimum_level);
    if let Lod::Fixed(level) = policy {
        return levels[resolve(levels, level.max(settings.minimum_level))];
    }
    let Some(pixels) = pixels else {
        return levels[finest];
    };
    if settings.thresholds.is_empty() {
        return levels[finest];
    }
    let pixels = pixels * settings.quality_bias;
    let mut index = previous
        .map(|level| resolve(levels, level))
        .unwrap_or(finest)
        .max(finest);
    let deadband = if previous.is_some() {
        settings.hysteresis
    } else {
        0.
    };
    while index > finest
        && settings
            .threshold(index - 1)
            .is_some_and(|threshold| pixels >= threshold * (1. + deadband))
    {
        index -= 1;
    }
    while index + 1 < levels.len()
        && settings
            .threshold(index)
            .is_some_and(|threshold| pixels < threshold * (1. - deadband))
    {
        index += 1;
    }
    levels[index]
}
fn projected(
    center: Vec3,
    radius: f32,
    camera: &GlobalTransform,
    projection: &Projection,
    height: f32,
) -> Option<f32> {
    let center = camera.affine().inverse().transform_point3(center);
    if !center.is_finite() || !radius.is_finite() || height <= 0. {
        return None;
    }
    let pixels = match projection {
        Projection::Perspective(view) => {
            let depth = -center.z;
            if depth + radius <= 0. {
                return None;
            }
            if depth - radius <= view.near {
                return Some(f32::INFINITY);
            }
            radius * height / ((view.fov * 0.5).tan() * (depth - radius))
        }
        Projection::Orthographic(_) => {
            radius * projection.get_clip_from_view().y_axis.y.abs() * height
        }
        Projection::Custom(_) => f32::INFINITY,
    };
    (!pixels.is_nan()).then_some(pixels)
}
type Instances<'w, 's> = Query<
    'w,
    's,
    (
        &'static ModelRig,
        &'static GlobalTransform,
        &'static mut Playback,
        Option<&'static Lod>,
        Option<&'static LodOverride>,
        Option<&'static RenderLayers>,
        &'static mut LodState,
    ),
>;
type Cameras<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Camera,
        &'static GlobalTransform,
        &'static Projection,
        Option<&'static RenderLayers>,
    ),
>;
/// Select detail against active matching cameras; common geometry remains visible.
pub(crate) fn update_lod(
    settings: Res<LodSettings>,
    mut instances: Instances<'_, '_>,
    cameras: Cameras<'_, '_>,
) {
    let fallback = LodSettings::default();
    let default_layers = RenderLayers::default();
    for (instance, transform, mut playback, policy, local, layers, mut state) in &mut instances {
        let settings = local.map(|local| &local.0).unwrap_or(&settings);
        let settings = if settings.validate().is_ok() {
            settings
        } else {
            &fallback
        };
        let policy = policy.copied().unwrap_or(settings.default_lod);
        let pixels = if policy == Lod::Automatic {
            instance.lod_bounds().and_then(|bounds| {
                let affine = transform.affine();
                let axes = [
                    Vec3::from(affine.matrix3.x_axis),
                    Vec3::from(affine.matrix3.y_axis),
                    Vec3::from(affine.matrix3.z_axis),
                ];
                let scale = axes
                    .iter()
                    .map(|axis| axes.iter().map(|other| axis.dot(*other).abs()).sum::<f32>())
                    .fold(0., f32::max)
                    .sqrt();
                let center = affine.transform_point3(bounds.center.into());
                let radius = Vec3::from(bounds.half_extents).length() * scale;
                cameras
                    .iter()
                    .filter(|(id, camera, _, _, camera_layers)| {
                        camera.is_active
                            && playback
                                .driving_camera
                                .is_none_or(|selected| selected == *id)
                            && camera_layers
                                .unwrap_or(&default_layers)
                                .intersects(layers.unwrap_or(&default_layers))
                    })
                    .filter_map(|(_, camera, transform, projection, _)| {
                        projected(
                            center,
                            radius,
                            transform,
                            projection,
                            camera.physical_viewport_size()?.y as f32,
                        )
                    })
                    .reduce(f32::max)
            })
        } else {
            None
        };
        let selected = select(
            &instance.lod_definition().levels,
            policy,
            settings,
            pixels,
            state.automatic.then_some(state.selected),
        );
        if playback.lod != selected {
            playback.lod = selected;
        }
        if state.selected != selected {
            state.selected = selected;
        }
        let automatic = policy == Lod::Automatic;
        if state.automatic != automatic {
            state.automatic = automatic;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sparse_levels_quality_caps_and_hysteresis() {
        let levels = [0, 2, 5];
        let settings = LodSettings::default();
        assert_eq!(select(&levels, Lod::Fixed(1), &settings, None, None), 2);
        assert_eq!(select(&levels, Lod::Fixed(99), &settings, None, None), 5);
        assert_eq!(
            select(&levels, Lod::Automatic, &settings, Some(230.), Some(0)),
            0
        );
        assert_eq!(
            select(&levels, Lod::Automatic, &settings, Some(190.), Some(0)),
            2
        );
        assert_eq!(select(&levels, Lod::Automatic, &settings, None, Some(5)), 0);
    }
}
