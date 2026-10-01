//! Model-authored lights integrated with Bevy's scene lighting.
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::scene::{Light, LightType};
use wc3::model::V1800;

use crate::animation::{sample, sample_value, Wc3Animation};
use crate::spawn::PreparedModel;

/// Conversion controls on a model's animation root. Absence uses the defaults.
/// These are integration scales, not calibrated Warcraft-to-physical conversions.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wc3LightSettings {
    /// Disable all imported lights without removing their entities.
    pub enabled: bool,
    /// Lumens per authored point-light intensity unit.
    pub point_intensity_scale: f32,
    /// Lux per authored directional-light intensity unit.
    pub directional_intensity_scale: f32,
    /// World units per authored attenuation-end unit. Node scale is not applied.
    pub range_scale: f32,
    /// World-space range when attenuation end is nonpositive or nonfinite.
    pub fallback_range: f32,
    /// Allow shadow maps for lights whose source has ShadowCasting enabled.
    pub shadows_enabled: bool,
}

impl Default for Wc3LightSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            point_intensity_scale: 1_000.0,
            directional_intensity_scale: 10_000.0,
            range_scale: 1.0,
            fallback_range: 1_000.0,
            shadows_enabled: true,
        }
    }
}

/// An imported light, parented to its animated WC3 node.
///
/// Point/directional entities also carry ordinary Bevy light components. The
/// plugin owns their color, power, range, and shadow-map enable flag; applications
/// may customize other fields (radius, shadow bias, render layers, etc.). Ambient
/// and unknown records retain this component but create no Bevy light.
#[derive(Component)]
pub struct Wc3Light {
    /// Animation root that owns this light.
    pub root: Entity,
    /// Source record, including ambient/falloff/shadow properties not mapped to Bevy.
    definition: Light<V1800>,
    /// Per-light suppression, combined with root settings and animated visibility.
    pub enabled: bool,
}

impl Wc3Light {
    /// Source record, including properties with no native Bevy mapping.
    pub fn definition(&self) -> &Light<V1800> {
        &self.definition
    }
}

pub(crate) fn spawn_lights(
    commands: &mut Commands,
    prepared: &PreparedModel,
    root: Entity,
    nodes: &HashMap<u32, Entity>,
) {
    for definition in prepared.model.lights() {
        let Some(&node) = nodes.get(&definition.node.object_id) else {
            continue;
        };
        let mut entity = commands.spawn((
            Name::new(format!("WC3 light: {}", definition.node.name.text())),
            Transform::default(),
            Visibility::default(),
            Wc3Light {
                root,
                definition: definition.clone(),
                enabled: true,
            },
        ));
        match definition.light_type {
            LightType::Omnidirectional => {
                entity.insert(PointLight {
                    intensity: 0.0,
                    ..default()
                });
            }
            LightType::Directional => {
                // WC3 +Z is the surface-to-light direction. Bevy emits along -Z.
                entity.insert(DirectionalLight {
                    illuminance: 0.0,
                    ..default()
                });
            }
            LightType::Ambient => {
                warn!("WC3 ambient light '{}' has no spatial Bevy equivalent; leaving scene ambient unchanged", definition.node.name.text());
            }
            LightType::Unknown(kind) => {
                warn!(
                    "Unsupported WC3 light type {kind} on '{}'",
                    definition.node.name.text()
                );
            }
        }
        let light = entity.id();
        commands.entity(node).add_child(light);
    }
}

fn nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

pub(crate) fn animate_lights(
    roots: Query<(&Wc3Animation, Option<&Wc3LightSettings>)>,
    mut lights: Query<(
        &Wc3Light,
        &mut Visibility,
        Option<&mut PointLight>,
        Option<&mut DirectionalLight>,
    )>,
) {
    for (light, mut visibility, point, directional) in &mut lights {
        let Ok((animation, settings)) = roots.get(light.root) else {
            continue;
        };
        let settings = settings.copied().unwrap_or_default();
        let definition = &light.definition;
        let visible = light.enabled
            && settings.enabled
            && definition
                .visibility
                .as_ref()
                .and_then(|track| sample(track, animation))
                .unwrap_or(1.0)
                > 0.0;
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        let intensity = if visible {
            nonnegative(sample_value(&definition.intensity, animation))
        } else {
            0.0
        };
        let [r, g, b] = sample_value(&definition.color, animation).map(nonnegative);
        let color = Color::linear_rgb(r, g, b);
        let shadows = visible && settings.shadows_enabled && definition.shadow_casting();
        if let Some(mut point) = point {
            let end = sample_value(&definition.attenuation_end, animation);
            point.color = color;
            point.intensity = nonnegative(intensity * nonnegative(settings.point_intensity_scale));
            point.range = if end.is_finite() && end > 0.0 {
                nonnegative(end * nonnegative(settings.range_scale))
            } else {
                nonnegative(settings.fallback_range)
            }
            .max(0.001);
            point.shadow_maps_enabled = shadows;
        }
        if let Some(mut directional) = directional {
            directional.color = color;
            directional.illuminance =
                nonnegative(intensity * nonnegative(settings.directional_intensity_scale));
            directional.shadow_maps_enabled = shadows;
        }
    }
}

#[cfg(test)]
#[path = "light_tests.rs"]
mod tests;
