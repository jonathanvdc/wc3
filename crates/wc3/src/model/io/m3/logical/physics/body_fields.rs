//! Revision-specific body fields become authored controls rather than source layouts.
use super::super::{
    materials::{anim, read_error, revision},
    slots::Slot,
};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{records, Container, Envelope, Ref};
use crate::model::PropertyValues;
use crate::model::{
    profiles::m3::{values, Body, BodyParameters, DynamicState},
    NodeId,
};

impl Slot<u16> for u16 {
    fn slot(self) -> Option<u16> {
        Some(self)
    }
}
impl Slot<u32> for records::RigidBodyFlags {
    fn slot(self) -> Option<u32> {
        Some(self.bits())
    }
}
impl Slot<u16> for records::LocalForces {
    fn slot(self) -> Option<u16> {
        Some(self.bits())
    }
}
impl Slot<u16> for records::WorldForces {
    fn slot(self) -> Option<u16> {
        Some(self.bits())
    }
}
impl Slot<records::PhysicalMaterial> for records::PhysicalMaterial {
    fn slot(self) -> Option<Self> {
        Some(self)
    }
}

pub(super) struct Input<'a> {
    pub body: Body<'a>,
    pub shapes: Ref<records::targets::PhysicsShape>,
}
pub(super) fn read<'view, E: Envelope>(
    document: &'view Container<'_, E>,
    reference: Ref<records::targets::RigidBody>,
    nodes: &[NodeId],
    context: &mut Context,
    values: &mut PropertyValues,
) -> Result<Vec<Input<'view>>, Error> {
    if reference.is_empty() {
        return Ok(Vec::new());
    }
    let mut output = Vec::new();
    macro_rules! read {
        ($version:ty) => {
            for row in document
                .resolve::<records::RigidBody<$version>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                for (field, empty) in [
                    ("prefix", r.reserved_0.empty()),
                    ("attachment_controls", r.reserved_1.empty()),
                    ("control_2", r.reserved_2.empty()),
                    ("control_3", r.reserved_3.empty()),
                    ("control_4", r.reserved_4.empty()),
                    ("control_5", r.reserved_5.empty()),
                ] {
                    if !empty {
                        context.unsupported(
                            format!("m3.physics.body.{field}"),
                            "uninterpreted body controls have no logical meaning assigned",
                        )?;
                    }
                }
                let node = *nodes.get(r.parent_bone_index as usize).ok_or_else(|| {
                    Error::new(
                        "m3.physics.body.node",
                        "body attachment exceeds the declared skeleton",
                    )
                })?;
                let material = Slot::<records::PhysicalMaterial>::slot(r.physics_type);
                let parameters = material.map(|material| BodyParameters {
                    material: values::PhysicalMaterial::from_raw(material.raw()),
                    mass: Slot::<f32>::slot(r.mass).unwrap_or_default(),
                    friction: Slot::<f32>::slot(r.friction).unwrap_or_default(),
                    restitution: Slot::<f32>::slot(r.restitution).unwrap_or_default(),
                    linear_damping: Slot::<f32>::slot(r.linear_damping).unwrap_or_default(),
                    angular_damping: Slot::<f32>::slot(r.angular_damping).unwrap_or_default(),
                    gravity_factor: Slot::<f32>::slot(r.gravity_factor).unwrap_or_default(),
                });
                let dynamic =
                    Slot::<records::AnimationValue<u32>>::slot(r.dynamic_state).map(|state| {
                        DynamicState {
                            state: anim(state, values),
                            blend_out: Slot::<f32>::slot(r.dynamic_blend_out).unwrap_or_default(),
                        }
                    });
                let mut body = Body::new(node);
                body.simulation_type = Slot::<u16>::slot(r.simulation_type).unwrap_or_default();
                body.parameters = parameters;
                body.dynamic = dynamic;
                body.flags = values::RigidBodyFlags(Slot::<u32>::slot(r.flags).unwrap_or_default());
                body.local_forces =
                    values::LocalForces(Slot::<u16>::slot(r.local_forces).unwrap_or_default());
                body.world_forces =
                    values::WorldForces(Slot::<u16>::slot(r.world_forces).unwrap_or_default());
                body.priority = Slot::<u32>::slot(r.priority).unwrap_or_default();
                output.push(Input {
                    shapes: r.rigid_body_shape,
                    body,
                });
            }
        };
    }
    match revision(document, reference).map_err(read_error)? {
        0 => read!(records::V0),
        1 => read!(records::V1),
        2 => read!(records::V2),
        3 => read!(records::V3),
        4 => read!(records::V4),
        _ => {
            return Err(Error::new(
                "m3.physics.body",
                "unsupported rigid-body record revision",
            ));
        }
    }
    Ok(output)
}

trait Empty {
    fn empty(self) -> bool;
}
impl Empty for () {
    fn empty(self) -> bool {
        true
    }
}
impl Empty for u8 {
    fn empty(self) -> bool {
        self == 0
    }
}
impl<const N: usize> Empty for records::FixedBytes<N> {
    fn empty(self) -> bool {
        self.0.iter().all(|value| *value == 0)
    }
}
