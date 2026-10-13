//! Native body revisions normalized into the WEM rigid-body fields.
use super::*;
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
    pub reserved: u16,
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
        reserved: value.reserved,
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
                reserved: r.reserved,
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
                reserved: r.reserved,
            })),
            _ => {}
        }
    }
    bodies
}
pub(super) fn body_record(
    value: &Body,
    id: u32,
    node: u32,
    shapes: wem::Ref<wr::targets::PhysicsShape>,
) -> wr::PhysicsBody<wr::V3> {
    wr::PhysicsBody {
        id,
        node,
        motion: match value.body_type {
            PhysicsBodyKind::Kinematic => wr::BodyMotion::Kinematic,
            PhysicsBodyKind::Dynamic => wr::BodyMotion::Dynamic,
            _ => wr::BodyMotion::Static,
        },
        simulates: wem::Bool::from(value.body_type == PhysicsBodyKind::Dynamic),
        shapes,
        gravity_scale: value.gravity,
        inertia_scale: value.inertia,
        linear_damping: value.linear,
        angular_damping: value.angular,
        wow: wem::InlineOption::from(Some(wr::WowBodyExtension {
            follow_factor: value.follow,
            has_children: wem::Bool::from(value.attachment.attachment_host()),
            ragdoll_root: wem::Bool::from(value.attachment.ragdoll_root()),
            parent: if value.attachment.parent_body() != 0 {
                u32::from(value.attachment.parent_body()) + 1
            } else {
                u32::MAX
            },
        })),
        ..Default::default()
    }
}
