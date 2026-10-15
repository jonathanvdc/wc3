//! Static collision surfaces borrow their actual positions, faces, and face normals.
use super::{materials::read_error, write::append, Graph, GraphBuilder};
use crate::model::conversion::materialize;
use crate::model::conversion::Error;
use crate::model::m3::records::{self, Extent, Vector3, U16};
use crate::model::m3::{self, Document, Envelope, Model, Ref};
use crate::model::semantic::profiles::m3::{CollisionSystem, System};
use crate::model::semantic::{Bounds, M3CollisionGeometry, M3Geometry, Mesh, Topology};
use std::array::from_fn;

pub(super) fn read<'view, E: Envelope + Send + Sync>(
    source: &'view Model<'_, E>,
    builder: &mut GraphBuilder<'view>,
) -> Result<(), Error> {
    let root = source.root_record().map_err(read_error)?;
    let (positions, faces, normals, bounds) = m3::visit_m3_root!(root, |r| (
        r.collision_verts,
        r.collision_faces,
        r.collision_normals,
        r.collision_bounds
    ));
    if positions.is_empty() && faces.is_empty() && normals.is_empty() && bounds == Extent::default()
    {
        return Ok(());
    }
    let points = source
        .document()
        .resolve::<Vector3>(positions.cast())
        .map_err(read_error)?;
    for row in points.iter() {
        row.map_err(read_error)?;
    }
    let positions = materialize(points.len(), move |index| {
        points
            .get(index)
            .expect("checked collision position index")
            .expect("validated collision position")
            .value
    });
    let indices = source
        .document()
        .resolve::<U16>(faces.cast())
        .map_err(read_error)?;
    if indices.len() % 3 != 0 {
        return Err(Error::new(
            "m3.collision.faces",
            "collision indices do not form complete triangles",
        ));
    }
    for row in indices.iter() {
        row.map_err(read_error)?;
    }
    let triangles = materialize(indices.len() / 3, move |index| {
        from_fn(|lane| {
            u32::from(
                indices
                    .get(index * 3 + lane)
                    .expect("checked collision face index")
                    .expect("validated collision face")
                    .value,
            )
        })
    });
    let normals = source
        .document()
        .resolve::<Vector3>(normals.cast())
        .map_err(read_error)?;
    for row in normals.iter() {
        row.map_err(read_error)?;
    }
    let normals = materialize(normals.len(), move |index| {
        normals
            .get(index)
            .expect("checked collision normal index")
            .expect("validated collision normal")
            .value
    });
    let geometry = M3CollisionGeometry::new(positions, triangles)?.with_face_normals(normals)?;
    let mut mesh = Mesh::new(M3Geometry::Collision(geometry));
    mesh.name = "Collision surface".into();
    mesh.bounds = Bounds {
        minimum: bounds.minimum,
        maximum: bounds.maximum,
        sphere_radius: bounds.radius,
    };
    let mesh = builder.add_mesh(mesh)?;
    builder.add_system(System::Collision(CollisionSystem { mesh }))?;
    Ok(())
}
#[derive(Default)]
pub(super) struct Collision {
    pub positions: Ref<records::targets::Vector3>,
    pub faces: Ref<records::targets::U16>,
    pub normals: Ref<records::targets::Vector3>,
    pub bounds: Extent,
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Document<'_, E>,
) -> Result<Collision, Error> {
    let mut result = Collision::default();
    let mut found = false;
    for (_, system) in graph.systems() {
        let System::Collision(system) = system else {
            continue;
        };
        if found {
            return Err(Error::new(
                "m3.collision",
                "native root contains one static collision surface",
            ));
        }
        found = true;
        let mesh = graph
            .mesh(system.mesh)
            .ok_or_else(|| Error::new("m3.collision.mesh", "collision mesh is absent"))?;
        let M3Geometry::Collision(geometry) = &mesh.geometry else {
            return Err(Error::new(
                "m3.collision.geometry",
                "collision surface requires collision geometry",
            ));
        };
        let positions = mesh
            .positions()
            .iter()
            .map(|value| Vector3 { value })
            .collect::<Vec<_>>();
        let Topology::Triangles(triangles) = mesh.topology() else {
            return Err(Error::new(
                "m3.collision.topology",
                "collision surface requires triangles",
            ));
        };
        let faces = triangles
            .iter()
            .flat_map(|face| face.into_iter())
            .map(|value| {
                u16::try_from(value)
                    .map(|value| U16 { value })
                    .map_err(|_| {
                        Error::new(
                            "m3.collision.faces",
                            "collision index exceeds native u16 capacity",
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let normals = geometry
            .face_normals()
            .map(|values| {
                values
                    .iter()
                    .map(|value| Vector3 { value })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        result.positions = append(document, &positions)?.cast();
        result.faces = append(document, &faces)?.cast();
        result.normals = append(document, &normals)?.cast();
        let bounds = mesh.bounds;
        result.bounds = Extent {
            minimum: bounds.minimum,
            maximum: bounds.maximum,
            radius: bounds.sphere_radius,
        };
    }
    Ok(result)
}
