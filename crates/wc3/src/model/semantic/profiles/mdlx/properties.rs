//! Typed Warcraft III property definitions used by semantic animation bindings.
use crate::model::semantic::{FeatureId, MaterialId, PropertyKey, TextureId};

/// Animated geometry properties on an attached feature.
pub mod geometry {
    use super::{FeatureId, PropertyKey};
    /// Animated opacity.
    pub const OPACITY: PropertyKey<FeatureId, f32> = PropertyKey::new("geometry.opacity");
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("geometry.color");
}

/// Animated particle properties on an attached feature.
pub mod particle {
    use super::{FeatureId, PropertyKey};
    /// Animated emission rate.
    pub const EMISSION_RATE: PropertyKey<FeatureId, f32> =
        PropertyKey::new("particle.emission_rate");
    /// Animated gravity.
    pub const GRAVITY: PropertyKey<FeatureId, f32> = PropertyKey::new("particle.gravity");
    /// Animated longitude.
    pub const LONGITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("particle.longitude");
    /// Animated latitude.
    pub const LATITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("particle.latitude");
    /// Animated life span.
    pub const LIFE_SPAN: PropertyKey<FeatureId, f32> = PropertyKey::new("particle.life_span");
    /// Animated initial velocity.
    pub const INITIAL_VELOCITY: PropertyKey<FeatureId, f32> =
        PropertyKey::new("particle.initial_velocity");
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32> = PropertyKey::new("particle.visibility");
}

/// Animated particle2 properties on an attached feature.
pub mod particle2 {
    use super::{FeatureId, PropertyKey};
    /// Animated speed.
    pub const SPEED: PropertyKey<FeatureId, f32> = PropertyKey::new("particle2.speed");
    /// Animated variation.
    pub const VARIATION: PropertyKey<FeatureId, f32> = PropertyKey::new("particle2.variation");
    /// Animated latitude.
    pub const LATITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("particle2.latitude");
    /// Animated gravity.
    pub const GRAVITY: PropertyKey<FeatureId, f32> = PropertyKey::new("particle2.gravity");
    /// Animated emission rate.
    pub const EMISSION_RATE: PropertyKey<FeatureId, f32> =
        PropertyKey::new("particle2.emission_rate");
    /// Animated length.
    pub const LENGTH: PropertyKey<FeatureId, f32> = PropertyKey::new("particle2.length");
    /// Animated width.
    pub const WIDTH: PropertyKey<FeatureId, f32> = PropertyKey::new("particle2.width");
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32> = PropertyKey::new("particle2.visibility");
}

/// Animated ribbon properties on an attached feature.
pub mod ribbon {
    use super::{FeatureId, PropertyKey};
    /// Animated height above.
    pub const HEIGHT_ABOVE: PropertyKey<FeatureId, f32> = PropertyKey::new("ribbon.height_above");
    /// Animated height below.
    pub const HEIGHT_BELOW: PropertyKey<FeatureId, f32> = PropertyKey::new("ribbon.height_below");
    /// Animated opacity.
    pub const OPACITY: PropertyKey<FeatureId, f32> = PropertyKey::new("ribbon.alpha");
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("ribbon.color");
    /// Animated texture slot.
    pub const TEXTURE_SLOT: PropertyKey<FeatureId, u32> = PropertyKey::new("ribbon.texture_slot");
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32> = PropertyKey::new("ribbon.visibility");
}

/// Animated popcorn properties on an attached feature.
pub mod popcorn {
    use super::{FeatureId, PropertyKey};
    /// Animated life span.
    pub const LIFE_SPAN: PropertyKey<FeatureId, f32> = PropertyKey::new("popcorn.life_span");
    /// Animated emission rate.
    pub const EMISSION_RATE: PropertyKey<FeatureId, f32> =
        PropertyKey::new("popcorn.emission_rate");
    /// Animated speed.
    pub const SPEED: PropertyKey<FeatureId, f32> = PropertyKey::new("popcorn.speed");
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("popcorn.color");
    /// Animated opacity.
    pub const OPACITY: PropertyKey<FeatureId, f32> = PropertyKey::new("popcorn.alpha");
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32> = PropertyKey::new("popcorn.visibility");
}

/// Animated light properties on an attached feature.
pub mod light {
    use super::{FeatureId, PropertyKey};
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("light.color");
    /// Animated intensity.
    pub const INTENSITY: PropertyKey<FeatureId, f32> = PropertyKey::new("light.intensity");
    /// Animated ambient color.
    pub const AMBIENT_COLOR: PropertyKey<FeatureId, [f32; 3]> =
        PropertyKey::new("light.ambient_color");
    /// Animated ambient intensity.
    pub const AMBIENT_INTENSITY: PropertyKey<FeatureId, f32> =
        PropertyKey::new("light.ambient_intensity");
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32> = PropertyKey::new("light.visibility");
    /// Animated attenuation start.
    pub const ATTENUATION_START: PropertyKey<FeatureId, f32> =
        PropertyKey::new("light.attenuation_start");
    /// Animated attenuation end.
    pub const ATTENUATION_END: PropertyKey<FeatureId, f32> =
        PropertyKey::new("light.attenuation_end");
    /// Animated shadow start.
    pub const SHADOW_START: PropertyKey<FeatureId, f32> = PropertyKey::new("light.shadow_start");
    /// Animated shadow end.
    pub const SHADOW_END: PropertyKey<FeatureId, f32> = PropertyKey::new("light.shadow_end");
    /// Animated quadratic.
    pub const QUADRATIC: PropertyKey<FeatureId, f32> = PropertyKey::new("light.quadratic");
    /// Animated linear.
    pub const LINEAR: PropertyKey<FeatureId, f32> = PropertyKey::new("light.linear");
    /// Animated damping.
    pub const DAMPING: PropertyKey<FeatureId, f32> = PropertyKey::new("light.damping");
}

/// Animated camera properties on an attached feature.
pub mod camera {
    use super::{FeatureId, PropertyKey};
    /// Animated eye offset.
    pub const EYE_OFFSET: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("camera.eye_offset");
    /// Animated target offset.
    pub const TARGET_OFFSET: PropertyKey<FeatureId, [f32; 3]> =
        PropertyKey::new("camera.target_offset");
    /// Animated roll.
    pub const ROLL: PropertyKey<FeatureId, f32> = PropertyKey::new("camera.roll");
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32> = PropertyKey::new("camera.visibility");
    /// Animated focus distance.
    pub const FOCUS_DISTANCE: PropertyKey<FeatureId, f32> =
        PropertyKey::new("camera.focus_distance");
    /// Animated focal length.
    pub const FOCAL_LENGTH: PropertyKey<FeatureId, f32> = PropertyKey::new("camera.focal_length");
    /// Animated f stop.
    pub const F_STOP: PropertyKey<FeatureId, f32> = PropertyKey::new("camera.f_stop");
}

/// Animated attachment properties on an attached feature.
pub mod attachment {
    use super::{FeatureId, PropertyKey};
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32> = PropertyKey::new("attachment.visibility");
}

/// Material-layer destinations selected by their logical layer index.
#[derive(Clone, Copy, Debug)]
pub struct Layer(pub usize);
impl Layer {
    /// Animated layer opacity.
    pub fn opacity(self) -> PropertyKey<MaterialId, f32> {
        PropertyKey::indexed(format!("layers[{}].opacity", self.0))
    }
    /// Animated layer texture selection.
    pub fn texture(self) -> PropertyKey<MaterialId, Option<TextureId>> {
        PropertyKey::indexed(format!("layers[{}].texture", self.0))
    }
    /// Animated emissive gain.
    pub fn emissive_gain(self) -> PropertyKey<MaterialId, f32> {
        PropertyKey::indexed(format!("layers[{}].emissive_gain", self.0))
    }
    /// Animated Fresnel color.
    pub fn fresnel_color(self) -> PropertyKey<MaterialId, [f32; 3]> {
        PropertyKey::indexed(format!("layers[{}].fresnel.color", self.0))
    }
    /// Animated Fresnel opacity.
    pub fn fresnel_opacity(self) -> PropertyKey<MaterialId, f32> {
        PropertyKey::indexed(format!("layers[{}].fresnel.opacity", self.0))
    }
    /// Animated Fresnel team-color contribution.
    pub fn fresnel_team_color(self) -> PropertyKey<MaterialId, f32> {
        PropertyKey::indexed(format!("layers[{}].fresnel.team_color", self.0))
    }
    /// Animated texture selection for an additional logical sampling slot.
    pub fn texture_slot(self, slot: usize) -> PropertyKey<MaterialId, Option<TextureId>> {
        PropertyKey::indexed(format!("layers[{}].textures[{slot}]", self.0))
    }
}
