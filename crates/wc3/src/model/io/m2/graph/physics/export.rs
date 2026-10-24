//! Builds a native companion from independently editable simulation data.
use super::super::overrides::file_key;
use super::*;
use crate::model::error::Location;
use export_joints::Constraints;
use export_shapes::Geometry;
use glam::Vec3;
use native::companions::{PhysicsBody4, PhysicsVegetation};
use native::records::{PhysicsAttachment, PhysicsBodyKind, PhysicsKind};
use native::ZeroVec;
pub(in super::super) fn export<D: ExportDialect>(
    model: &Graph<'_>,
    nodes: &[NodeId],
    bundle: &mut Bundle<'static, D>,
    context: &mut Context,
) -> Result<(), Error> {
    for (_, system) in model.systems() {
        if let profile::System::Physics(physics) = system {
            export_one(model, physics, nodes, bundle, context)?;
        }
    }
    Ok(())
}
fn export_one<D: ExportDialect>(
    model: &Graph<'_>,
    physics: &profile::Physics<'_>,
    nodes: &[NodeId],
    bundle: &mut Bundle<'static, D>,
    context: &mut Context,
) -> Result<(), Error> {
    if !physics.collision_pairs.is_empty() {
        context.unsupported(
            Location::new().field("physics").field("collision_pairs"),
            "M2 has no explicit collision-pair table",
        )?;
    }
    if physics.wind_coefficients.is_some() {
        context.unsupported(
            Location::new().field("physics").field("wind"),
            "M2 has no model-wide wind-coefficient record",
        )?;
    }
    let placements: Vec<_> = physics.bodies.iter().map(|body| body.node).collect();
    let mut geometry = Geometry::default();
    let mut bodies = Vec::new();
    for body in &physics.bodies {
        let node = model.node(body.node).ok_or_else(|| {
            Error::requirement(
                Location::new().field("physics").field("body").field("node"),
                "placement is absent",
            )
        })?;
        if node.transform.defaults(model.values()).rotation != semantic::Quat::IDENTITY
            || node.transform.defaults(model.values()).scale != Vec3::from_array([1.; 3])
            || node.transform.defaults(model.values()).pivot != Vec3::from_array([0.; 3])
        {
            return Err(Error::requirement(
                Location::new()
                    .field("physics")
                    .field("body")
                    .field("transform"),
                "native body offsets support translation only",
            ));
        }
        let anchor = node.parent.unwrap_or(body.node);
        let bone = nodes.iter().position(|id| *id == anchor).ok_or_else(|| {
            Error::requirement(
                Location::new().field("physics").field("body").field("bone"),
                "bone anchor is absent",
            )
        })?;
        let first = geometry.shapes.len();
        for shape in &body.shapes {
            geometry.add(shape, context)?;
        }
        let mut attachment = PhysicsAttachment::default();
        attachment.set_attachment_host(body.attachment_host);
        attachment.set_ragdoll_root(body.ragdoll_root);
        if let Some(parent) = body.parent_body {
            let index = placements
                .iter()
                .position(|id| *id == parent)
                .ok_or_else(|| {
                    Error::requirement(
                        Location::new()
                            .field("physics")
                            .field("body")
                            .field("parent"),
                        "parent body is absent",
                    )
                })?;
            if index == 0 || index > 0x3fff {
                return Err(Error::requirement(
                    Location::new()
                        .field("physics")
                        .field("body")
                        .field("parent"),
                    "native parent index cannot express this body",
                ));
            }
            attachment.set_parent_body(index as u16);
        }
        bodies.push(PhysicsBody4 {
            body_type: PhysicsBodyKind::from_raw(body.kind.raw()),
            bone_index: u16::try_from(bone).map_err(|_| {
                Error::requirement(
                    Location::new().field("physics").field("body").field("bone"),
                    "bone index exceeds native range",
                )
            })?,
            position: (node.transform.defaults(model.values()).translation).to_array(),
            shape_index: i32::try_from(first).map_err(|_| {
                Error::requirement(
                    Location::new()
                        .field("physics")
                        .field("body")
                        .field("shapes"),
                    "shape start exceeds native range",
                )
            })?,
            shape_count: i32::try_from(geometry.shapes.len() - first).map_err(|_| {
                Error::requirement(
                    Location::new()
                        .field("physics")
                        .field("body")
                        .field("shapes"),
                    "shape count exceeds native range",
                )
            })?,
            gravity_scale: body.gravity_scale,
            inertia_scale: body.inertia_scale,
            linear_damping: body.linear_damping,
            angular_damping: body.angular_damping,
            follow_factor: body.follow_factor,
            attachment,
            reserved: 0,
        });
    }
    let mut constraints = Constraints::default();
    for joint in &physics.joints {
        constraints.add(joint, &placements, context)?;
    }
    let Geometry {
        boxes,
        capsules,
        spheres,
        polytopes,
        shapes,
    } = geometry;
    let Constraints {
        joints,
        spherical,
        shoulder,
        weld,
        revolute,
        prismatic,
        distance,
    } = constraints;
    let vegetation = physics
        .vegetation
        .iter()
        .map(|v| PhysicsVegetation {
            position_max_push: v.position_max_push,
            position_push_amount: v.position_push_amount,
            position_relax_speed: v.position_relax_speed,
            velocity_max_push: v.velocity_max_push,
            velocity_speed: v.velocity_speed,
            minimum_push_distance_squared: v.minimum_push_distance_squared,
        })
        .collect();
    let file = PhysicsFile {
        chunks: vec![
            Pc::Header(6),
            Pc::Kind(PhysicsKind::from_raw(physics.kind.raw())),
            Pc::Boxes(boxes),
            Pc::Capsules(capsules),
            Pc::Spheres(spheres),
            Pc::Polytopes(polytopes),
            Pc::Shapes2(shapes),
            Pc::Bodies4(bodies),
            Pc::Sphericals(spherical),
            Pc::Shoulders2(shoulder),
            Pc::Welds3(weld),
            Pc::Revolutes2(revolute),
            Pc::Prismatics2(prismatic),
            Pc::Distances(distance),
            Pc::Joints(joints),
            Pc::Vegetation(vegetation),
            Pc::AllowList {
                header: 0,
                keys: ZeroVec::alloc_from_slice(&physics.follow_allow_list.to_vec()),
            },
        ],
        padding: std::borrow::Cow::Borrowed(&[]),
    };
    file.validate_with_bone_count(nodes.len())
        .map_err(failure)?;
    if let Some(id) = &physics.resource {
        let resource = model
            .physics_resource(*id)
            .expect("validated physics resource");
        let key = file_key(resource);
        if let M2ResourceLocator::FileDataId(id) = resource {
            D::physics_id(&mut bundle.model, Some(*id), context)?;
        }
        if !matches!(resource, M2ResourceLocator::FileDataId(_)) {
            D::physics_id(&mut bundle.model, None, context)?;
        }
        bundle.insert(key, File::Physics(file));
    } else {
        if let Some(file) = D::inline_physics(&mut bundle.model, file)? {
            D::physics_id(&mut bundle.model, None, context)?;
            bundle.insert(bundle.key.sibling(".phys"), File::Physics(file));
        }
    }
    Ok(())
}
