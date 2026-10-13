//! Native geometry tables rebuilt from editable WEM physics shapes.
use super::*;
#[derive(Default)]
pub(super) struct Geometry {
    pub boxes: Vec<native::companions::PhysicsBox>,
    pub spheres: Vec<native::companions::PhysicsSphere>,
    pub capsules: Vec<native::companions::PhysicsCapsule>,
    pub polytopes: Vec<native::companions::Polytope<'static>>,
    pub shapes: Vec<native::companions::PhysicsShape2>,
}
impl Geometry {
    pub(super) fn add(
        &mut self,
        shape: &wr::PhysicsShape<wr::V2>,
        doc: &wem::RawDocument<'_>,
    ) -> Result<(), Error> {
        let (kind, index) = match shape.kind {
            wr::PhysicsShapeKind::BOX => {
                let index = self.boxes.len();
                self.boxes.push(native::companions::PhysicsBox {
                    frame: native_frame(shape.transform),
                    half_extents: shape.half_extents,
                });
                (0, index)
            }
            wr::PhysicsShapeKind::SPHERE => {
                let index = self.spheres.len();
                self.spheres.push(native::companions::PhysicsSphere {
                    position: shape.transform[3][..3].try_into().unwrap(),
                    radius: shape.radius,
                });
                (2, index)
            }
            wr::PhysicsShapeKind::CAPSULE => {
                let index = self.capsules.len();
                let points = doc.resolve::<wr::Vector3>(shape.points.cast())?;
                let (a, b) = if points.len() == 2 {
                    (points.get(0).unwrap()?.value, points.get(1).unwrap()?.value)
                } else {
                    let translation: [f32; 3] = shape.transform[3][..3].try_into().unwrap();
                    let axis: [f32; 3] = shape.transform[2][..3].try_into().unwrap();
                    (
                        from_fn(|i| translation[i] - axis[i] * shape.length * 0.5),
                        from_fn(|i| translation[i] + axis[i] * shape.length * 0.5),
                    )
                };
                self.capsules.push(native::companions::PhysicsCapsule {
                    position1: a,
                    position2: b,
                    radius: shape.radius,
                });
                (1, index)
            }
            wr::PhysicsShapeKind::CONVEX_HULL => {
                let index = self.polytopes.len();
                let points = doc
                    .resolve::<wr::Vector3>(shape.points.cast())?
                    .iter()
                    .map(|point| {
                        let point = point?.value;
                        Ok(from_fn(|row| {
                            shape.transform[3][row]
                                + (0..3)
                                    .map(|axis| shape.transform[axis][row] * point[axis])
                                    .sum::<f32>()
                        }))
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                self.polytopes.push(hull(points)?);
                (3, index)
            }
            _ => {
                return Err(Error::new(
                    "physics.shape.kind",
                    "native PHYS supports boxes, capsules, spheres and convex hulls",
                ));
            }
        };
        let shape_index = i16::try_from(index)
            .map_err(|_| Error::new("physics.shape.index", "geometry index exceeds i16"))?;
        self.shapes.push(native::companions::PhysicsShape2 {
            shape: native::companions::PhysicsShape {
                shape_type: kind,
                shape_index,
                game_flags: shape.game_flags,
                friction: shape.material.friction,
                restitution: shape.material.restitution,
                density: shape.material.density,
                ..Default::default()
            },
            ..Default::default()
        });
        Ok(())
    }
    pub(super) fn chunks(self) -> Vec<Pc<'static>> {
        let mut chunks = Vec::new();
        if !self.boxes.is_empty() {
            chunks.push(Pc::Boxes(self.boxes));
        }
        if !self.spheres.is_empty() {
            chunks.push(Pc::Spheres(self.spheres));
        }
        if !self.capsules.is_empty() {
            chunks.push(Pc::Capsules(self.capsules));
        }
        if !self.polytopes.is_empty() {
            chunks.push(Pc::Polytopes(self.polytopes));
        }
        chunks.push(Pc::Shapes2(self.shapes));
        chunks
    }
}
