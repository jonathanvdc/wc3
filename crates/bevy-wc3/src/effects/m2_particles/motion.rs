//! Shared analytical M2 motion for ordinary and adopted sprite pools.
use super::{Birth, Emitter};
use bevy::prelude::Vec3;

impl Emitter {
    pub(super) fn motion(&self, birth: &Birth, time: f64) -> (Vec3, Vec3) {
        let age = (time - birth.time).max(0.) as f32;
        if self.drag > 0.00001 {
            let decay = (-self.drag * age).exp();
            let vscale = (1. - decay) / self.drag;
            let ascale = (age - vscale) / self.drag;
            (
                birth.origin + birth.velocity * vscale + birth.acceleration * ascale,
                birth.velocity * decay + birth.acceleration * vscale,
            )
        } else {
            (
                birth.origin + birth.velocity * age + birth.acceleration * (0.5 * age * age),
                birth.velocity + birth.acceleration * age,
            )
        }
    }
}
