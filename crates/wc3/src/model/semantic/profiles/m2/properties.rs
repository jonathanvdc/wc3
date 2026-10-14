//! Typed World of Warcraft destinations, independent of native track descriptors.
use crate::model::semantic::{FeatureId, MaterialId, PropertyKey, Quaternion};
/// Camera destinations on an attached camera feature.
pub mod camera {
    use super::{FeatureId, PropertyKey};
    /// Animated camera position.
    pub const POSITION: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("m2.camera.position");
    /// Animated camera target.
    pub const TARGET: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("m2.camera.target");
    /// Animated camera roll.
    pub const ROLL: PropertyKey<FeatureId, f32> = PropertyKey::new("m2.camera.roll");
    /// Animated field of view.
    pub const FIELD_OF_VIEW: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.camera.field_of_view");
}
/// Animated material color.
pub const COLOR: PropertyKey<MaterialId, [f32; 3]> = PropertyKey::new("m2.color");
/// Animated material opacity.
pub const OPACITY: PropertyKey<MaterialId, f32> = PropertyKey::new("m2.opacity");

/// Animated attachment properties on an attached feature.
pub mod attachment {
    use super::{FeatureId, PropertyKey};
    /// Animated enabled.
    pub const ENABLED: PropertyKey<FeatureId, bool> = PropertyKey::new("m2.attachment.enabled");
}

/// Animated light properties on an attached feature.
pub mod light {
    use super::{FeatureId, PropertyKey};
    /// Animated ambient color.
    pub const AMBIENT_COLOR: PropertyKey<FeatureId, [f32; 3]> =
        PropertyKey::new("m2.light.ambient_color");
    /// Animated ambient intensity.
    pub const AMBIENT_INTENSITY: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.light.ambient_intensity");
    /// Animated diffuse color.
    pub const DIFFUSE_COLOR: PropertyKey<FeatureId, [f32; 3]> =
        PropertyKey::new("m2.light.diffuse_color");
    /// Animated diffuse intensity.
    pub const DIFFUSE_INTENSITY: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.light.diffuse_intensity");
    /// Animated attenuation start.
    pub const ATTENUATION_START: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.light.attenuation_start");
    /// Animated attenuation end.
    pub const ATTENUATION_END: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.light.attenuation_end");
    /// Animated enabled.
    pub const ENABLED: PropertyKey<FeatureId, bool> = PropertyKey::new("m2.light.enabled");
}

/// Animated ribbon properties on an attached feature.
pub mod ribbon {
    use super::{FeatureId, PropertyKey};
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("m2.ribbon.color");
    /// Animated alpha.
    pub const ALPHA: PropertyKey<FeatureId, f32> = PropertyKey::new("m2.ribbon.alpha");
    /// Animated height above.
    pub const HEIGHT_ABOVE: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.ribbon.height_above");
    /// Animated height below.
    pub const HEIGHT_BELOW: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.ribbon.height_below");
    /// Animated texture slot.
    pub const TEXTURE_SLOT: PropertyKey<FeatureId, u16> =
        PropertyKey::new("m2.ribbon.texture_slot");
    /// Animated enabled.
    pub const ENABLED: PropertyKey<FeatureId, bool> = PropertyKey::new("m2.ribbon.enabled");
}

/// Animated particle properties on an attached feature.
pub mod particle {
    use super::{FeatureId, PropertyKey};
    /// Animated speed.
    pub const SPEED: PropertyKey<FeatureId, f32> = PropertyKey::new("m2.particle.speed");
    /// Animated speed variation.
    pub const SPEED_VARIATION: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.particle.speed_variation");
    /// Animated vertical range.
    pub const VERTICAL_RANGE: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.particle.vertical_range");
    /// Animated horizontal range.
    pub const HORIZONTAL_RANGE: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.particle.horizontal_range");
    /// Animated lifespan.
    pub const LIFESPAN: PropertyKey<FeatureId, f32> = PropertyKey::new("m2.particle.lifespan");
    /// Animated rate.
    pub const RATE: PropertyKey<FeatureId, f32> = PropertyKey::new("m2.particle.rate");
    /// Animated area width.
    pub const AREA_WIDTH: PropertyKey<FeatureId, f32> = PropertyKey::new("m2.particle.area_width");
    /// Animated area length.
    pub const AREA_LENGTH: PropertyKey<FeatureId, f32> =
        PropertyKey::new("m2.particle.area_length");
    /// Animated z source.
    pub const Z_SOURCE: PropertyKey<FeatureId, f32> = PropertyKey::new("m2.particle.z_source");
    /// Animated enabled.
    pub const ENABLED: PropertyKey<FeatureId, bool> = PropertyKey::new("m2.particle.enabled");
    /// Animated gravity.
    pub const GRAVITY: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("m2.particle.gravity");
}

/// Material texture-unit destinations selected by logical slot.
#[derive(Clone, Copy, Debug)]
pub struct TextureUnit(pub usize);
impl TextureUnit {
    /// Animated sampling weight.
    pub fn weight(self) -> PropertyKey<MaterialId, f32> {
        PropertyKey::indexed(format!("m2.texture.{}.weight", self.0))
    }
    /// Animated coordinate translation.
    pub fn translation(self) -> PropertyKey<MaterialId, [f32; 3]> {
        PropertyKey::indexed(format!("m2.texture.{}.translation", self.0))
    }
    /// Animated coordinate rotation.
    pub fn rotation(self) -> PropertyKey<MaterialId, Quaternion> {
        PropertyKey::indexed(format!("m2.texture.{}.rotation", self.0))
    }
    /// Animated coordinate scale.
    pub fn scale(self) -> PropertyKey<MaterialId, [f32; 3]> {
        PropertyKey::indexed(format!("m2.texture.{}.scale", self.0))
    }
}
