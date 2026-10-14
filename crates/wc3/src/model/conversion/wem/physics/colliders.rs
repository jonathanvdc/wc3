//! Cloth collision primitives follow checked body-owned shapes.
use super::codec::{allocate, append, mapped, node, read_error, revision, Scalar};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::{
    BodyId, ClothCollider, ClothColliderId, ClothColliderKind, Physics,
};
use crate::model::semantic::NodeId;
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::BTreeMap;
pub(super) fn read(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::ClothCollider>,
    nodes: &[NodeId],
    target: &mut Physics<'_>,
    bodies: &BTreeMap<u32, BodyId>,
) -> Result<BTreeMap<u32, ClothColliderId>, Error> {
    let mut ids = BTreeMap::new();
    macro_rules! rows {
        ($revision:ty) => {{
            for row in document
                .resolve::<records::ClothCollider<$revision>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let collider = collider(&row, nodes, target, bodies)?;
                let id = target.add_cloth_collider(collider);
                if ids.insert(row.id, id).is_some() {
                    return Err(Error::new(
                        "wem.physics.collider",
                        "duplicate collider identity",
                    ));
                }
            }
        }};
    }
    if reference.entries != 0 {
        match revision(document, reference)? {
            Some(1) => rows!(records::V1),
            Some(2) => rows!(records::V2),
            _ => {
                return Err(Error::new(
                    "wem.physics.collider",
                    "unsupported cloth collider revision",
                ));
            }
        }
    }
    Ok(ids)
}
fn collider<D: records::ClothColliderDialect>(
    value: &records::ClothCollider<D>,
    nodes: &[NodeId],
    target: &Physics<'_>,
    bodies: &BTreeMap<u32, BodyId>,
) -> Result<ClothCollider, Error>
where
    D::Body: Scalar<u32>,
    D::Shape: Scalar<u32>,
{
    let follows = Scalar::<u32>::scalar(&value.body)
        .filter(|body| *body != 0)
        .map(|body| {
            let body = *bodies.get(&body).ok_or_else(|| {
                Error::new(
                    "wem.physics.collider",
                    "following body is outside the simulation domain",
                )
            })?;
            let index = Scalar::<u32>::scalar(&value.shape).ok_or_else(|| {
                Error::new("wem.physics.collider", "following shape is undeclared")
            })?;
            let shape = *target
                .body(body)
                .unwrap()
                .shapes
                .get(index as usize)
                .ok_or_else(|| {
                    Error::new(
                        "wem.physics.collider",
                        "following shape is not attached to this body",
                    )
                })?;
            Ok::<_, Error>((body, shape))
        })
        .transpose()?;
    Ok(ClothCollider {
        node: if value.node == u32::MAX {
            None
        } else {
            Some(node(nodes, value.node)?)
        },
        kind: match value.kind {
            records::ClothColliderKind::Capsule => ClothColliderKind::Capsule,
            records::ClothColliderKind::Plane => ClothColliderKind::Plane,
            records::ClothColliderKind::Count => ClothColliderKind::Count,
            records::ClothColliderKind::Unknown(code) => ClothColliderKind::Unknown(code),
        },
        transform: value.transform,
        radius: value.radius,
        length: value.length,
        follows,
    })
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    source: &Physics<'_>,
    bodies: &BTreeMap<BodyId, u32>,
    nodes: &BTreeMap<NodeId, u32>,
    next: &mut u32,
) -> Result<
    (
        Ref<records::targets::ClothCollider>,
        BTreeMap<ClothColliderId, u32>,
    ),
    Error,
> {
    let mut ids = BTreeMap::new();
    let mut rows = Vec::new();
    for (id, value) in source.cloth_colliders() {
        let number = allocate(next)?;
        ids.insert(id, number);
        let (body, shape) = value
            .follows
            .map(|(body, shape)| {
                let index = source
                    .body(body)
                    .ok_or_else(|| {
                        Error::new(
                            "wem.physics.collider",
                            "following body belongs to another domain",
                        )
                    })?
                    .shapes
                    .iter()
                    .position(|id| *id == shape)
                    .ok_or_else(|| {
                        Error::new(
                            "wem.physics.collider",
                            "following shape is not attached to its body",
                        )
                    })?;
                Ok::<_, Error>((
                    mapped(bodies, body, "following body")?,
                    u32::try_from(index).map_err(read_error)?,
                ))
            })
            .transpose()?
            .unwrap_or((0, 0));
        rows.push(records::ClothCollider::<records::V2> {
            id: number,
            node: value
                .node
                .map(|id| mapped(nodes, id, "cloth anchor"))
                .transpose()?
                .unwrap_or(u32::MAX),
            kind: match value.kind {
                ClothColliderKind::Capsule => records::ClothColliderKind::Capsule,
                ClothColliderKind::Plane => records::ClothColliderKind::Plane,
                ClothColliderKind::Count => records::ClothColliderKind::Count,
                ClothColliderKind::Unknown(code) => records::ClothColliderKind::Unknown(code),
            },
            transform: value.transform,
            radius: value.radius,
            length: value.length,
            body,
            shape,
        });
    }
    Ok((append(document, &rows)?.cast(), ids))
}
