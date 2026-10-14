//! Native body revisions expose their authored rigid-body parameters.
use super::{native, Pc, PhysicsFile};
use native::records::{PhysicsAttachment, PhysicsBodyKind};
#[derive(Clone, Debug)]
pub(super) struct Body {
    pub body_type: PhysicsBodyKind,
    pub bone: u32,
    pub position: [f32; 3],
    pub first: i32,
    pub count: i32,
    pub gravity: f32,
    pub inertia: f32,
    pub linear: f32,
    pub angular: f32,
    pub follow: f32,
    pub attachment: PhysicsAttachment,
}
fn legacy(value: &native::companions::PhysicsBody) -> Body {
    Body {
        body_type: value.body_type,
        bone: value.bone_index,
        position: value.position,
        first: value.shape_index,
        count: value.shape_count,
        gravity: 1.,
        inertia: 1.,
        linear: 0.,
        angular: 0.,
        follow: 1.,
        attachment: PhysicsAttachment::default(),
    }
}
pub(super) fn bodies(source: &PhysicsFile<'_>) -> Vec<Body> {
    let mut bodies = Vec::new();
    for chunk in &source.chunks {
        match chunk {
            Pc::Bodies(rows) => bodies.extend(rows.iter().map(legacy)),
            Pc::Bodies2(rows) => bodies.extend(rows.iter().map(|row| {
                let mut value = legacy(&row.body);
                value.gravity = row.gravity_scale;
                value
            })),
            Pc::Bodies3(rows) => bodies.extend(rows.iter().map(|r| Body {
                body_type: r.body_type,
                bone: u32::from(r.bone_index),
                position: r.position,
                first: r.shape_index,
                count: r.shape_count,
                gravity: r.gravity_scale,
                inertia: r.inertia_scale,
                linear: r.linear_damping,
                angular: r.angular_damping,
                follow: 1.,
                attachment: r.attachment,
            })),
            Pc::Bodies4(rows) => bodies.extend(rows.iter().map(|r| Body {
                body_type: r.body_type,
                bone: u32::from(r.bone_index),
                position: r.position,
                first: r.shape_index,
                count: r.shape_count,
                gravity: r.gravity_scale,
                inertia: r.inertia_scale,
                linear: r.linear_damping,
                angular: r.angular_damping,
                follow: r.follow_factor,
                attachment: r.attachment,
            })),
            _ => {}
        }
    }
    bodies
}
