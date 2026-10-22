//! Bounded support-map distance and expanding-polytope contact queries.
//!
//! Numerical convergence is explicit. Neither a failed search nor its provisional
//! polytope is a contact, and this module never substitutes a sphere for a hull.
use super::contact::{Contact, Primitive};
use super::contact_work::{ContactWork, PhysicsContactIssue as Issue};
use bevy::math::{DMat3, DVec3, Mat4, Vec3};

const MAX_HULL_VERTICES: usize = 4096;
const MAX_VERTICES: usize = 96;
const MAX_FACES: usize = 192;
const MAX_EDGES: usize = 384;
const MAX_SEED: usize = 16;
const GJK_STEPS: usize = 48;
const EPA_STEPS: usize = 80;
const RELATIVE_TOLERANCE: f64 = 1e-8;

pub(super) struct Hull {
    vertices: Box<[Vec3]>,
    issue: Option<Issue>,
}
impl Hull {
    pub fn prepare(vertices: impl ExactSizeIterator<Item = Vec3>) -> Self {
        // The semantic model already retains its source array. Reject before
        // materializing any additional private copy of an oversized hull.
        if vertices.len() > MAX_HULL_VERTICES {
            return Self {
                vertices: Box::default(),
                issue: Some(Issue::CapacityLimit),
            };
        }
        Self::new(vertices.collect())
    }
    pub fn new(vertices: Vec<Vec3>) -> Self {
        let issue = if vertices.len() > MAX_HULL_VERTICES {
            Some(Issue::CapacityLimit)
        } else if vertices.iter().any(|point| !point.is_finite()) {
            Some(Issue::InvalidInput)
        } else if !full_dimension(&vertices) {
            Some(Issue::Degenerate)
        } else {
            None
        };
        Self {
            vertices: if issue.is_none() {
                vertices.into_boxed_slice()
            } else {
                Box::default()
            },
            issue,
        }
    }
    pub fn issue(&self) -> Option<Issue> {
        self.issue
    }
}
fn full_dimension(vertices: &[Vec3]) -> bool {
    let Some(first) = vertices.first().map(|p| p.as_dvec3()) else {
        return false;
    };
    let Some(edge) = vertices
        .iter()
        .map(|p| p.as_dvec3() - first)
        .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))
    else {
        return false;
    };
    let scale = edge.length();
    if scale == 0. || !scale.is_finite() {
        return false;
    }
    let edge = edge / scale;
    let Some(normal) = vertices
        .iter()
        .map(|p| edge.cross((p.as_dvec3() - first) / scale))
        .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))
    else {
        return false;
    };
    vertices
        .iter()
        .any(|p| normal.dot((p.as_dvec3() - first) / scale).abs() > 1e-14)
}
#[derive(Clone, Copy)]
pub(super) enum Shape<'a> {
    Hull(&'a Hull),
    Primitive(Primitive),
}
#[derive(Clone, Copy)]
pub(super) struct ShapeRef<'a> {
    shape: Shape<'a>,
    linear: DMat3,
    translation: DVec3,
}
impl<'a> ShapeRef<'a> {
    pub fn new(shape: Shape<'a>, matrix: Mat4) -> Result<Self, Issue> {
        if !matrix.is_finite()
            || matrix.x_axis.w != 0.
            || matrix.y_axis.w != 0.
            || matrix.z_axis.w != 0.
            || matrix.w_axis.w != 1.
        {
            return Err(Issue::InvalidInput);
        }
        let linear = DMat3::from_cols(
            matrix.x_axis.truncate().as_dvec3(),
            matrix.y_axis.truncate().as_dvec3(),
            matrix.z_axis.truncate().as_dvec3(),
        );
        if linear.determinant() == 0. {
            return Err(Issue::InvalidInput);
        }
        match shape {
            Shape::Hull(hull) => {
                if let Some(issue) = hull.issue {
                    return Err(issue);
                }
            }
            Shape::Primitive(Primitive::Sphere { center, radius }) => {
                if !center.is_finite() || !radius.is_finite() || radius < 0. {
                    return Err(Issue::InvalidInput);
                }
            }
            Shape::Primitive(Primitive::Capsule { endpoints, radius }) => {
                if endpoints.iter().any(|p| !p.is_finite()) || !radius.is_finite() || radius < 0. {
                    return Err(Issue::InvalidInput);
                }
            }
            Shape::Primitive(Primitive::Box {
                center,
                axes,
                half_extents,
            }) => {
                if !center.is_finite()
                    || !axes.is_finite()
                    || !half_extents.is_finite()
                    || half_extents.min_element() < 0.
                {
                    return Err(Issue::InvalidInput);
                }
            }
        }
        Ok(Self {
            shape,
            linear,
            translation: matrix.w_axis.truncate().as_dvec3(),
        })
    }
    fn support(self, direction: DVec3, work: &mut ContactWork) -> Result<DVec3, Issue> {
        let direction = self.linear.transpose() * direction;
        if !direction.is_finite() || direction.length_squared() == 0. {
            return Err(Issue::InvalidInput);
        }
        let local = match self.shape {
            Shape::Hull(hull) => {
                let mut best = DVec3::ZERO;
                let mut score = f64::NEG_INFINITY;
                for point in &hull.vertices {
                    work.vertex()?;
                    let point = point.as_dvec3();
                    let candidate = point.dot(direction);
                    // Strict improvement keeps the lowest original index on ties.
                    if candidate > score {
                        best = point;
                        score = candidate;
                    }
                }
                best
            }
            Shape::Primitive(Primitive::Sphere { center, radius }) => {
                work.vertex()?;
                center.as_dvec3() + direction.normalize() * f64::from(radius)
            }
            Shape::Primitive(Primitive::Capsule { endpoints, radius }) => {
                work.vertex()?;
                let a = endpoints[0].as_dvec3();
                let sa = a.dot(direction);
                work.vertex()?;
                let b = endpoints[1].as_dvec3();
                (if b.dot(direction) > sa { b } else { a })
                    + direction.normalize() * f64::from(radius)
            }
            Shape::Primitive(Primitive::Box {
                center,
                axes,
                half_extents,
            }) => {
                work.vertex()?;
                let axes = DMat3::from_cols(
                    axes.x_axis.as_dvec3(),
                    axes.y_axis.as_dvec3(),
                    axes.z_axis.as_dvec3(),
                );
                let projected = axes.transpose() * direction;
                let half = half_extents.as_dvec3();
                let corner = DVec3::new(
                    if projected.x < 0. { -half.x } else { half.x },
                    if projected.y < 0. { -half.y } else { half.y },
                    if projected.z < 0. { -half.z } else { half.z },
                );
                center.as_dvec3() + axes * corner
            }
        };
        let point = self.linear * local + self.translation;
        if !point.is_finite() {
            return Err(Issue::InvalidInput);
        }
        Ok(point)
    }
}
#[derive(Clone, Copy, Default)]
struct Vertex {
    point: DVec3,
    a: DVec3,
    b: DVec3,
}
fn support(
    a: ShapeRef<'_>,
    b: ShapeRef<'_>,
    direction: DVec3,
    work: &mut ContactWork,
) -> Result<Vertex, Issue> {
    let left = a.support(direction, work)?;
    let right = b.support(-direction, work)?;
    let point = left - right;
    if !point.is_finite() {
        return Err(Issue::InvalidInput);
    }
    Ok(Vertex {
        point,
        a: left,
        b: right,
    })
}
#[derive(Clone, Copy)]
struct Closest {
    point: DVec3,
    weights: [f64; 4],
    mask: usize,
}
// Checked affine projection with normalized edges. Enumerating all faces
// avoids a second collection of region-specific simplex formulae.
fn affine(vertices: &[Vertex], mask: usize) -> Option<Closest> {
    let mut ids = [0; 4];
    let mut count = 0;
    for i in 0..vertices.len() {
        if mask & (1 << i) != 0 {
            ids[count] = i;
            count += 1;
        }
    }
    if count == 0 {
        return None;
    }
    let p = vertices[ids[0]].point;
    let mut weights = [0.; 4];
    weights[ids[0]] = 1.;
    if count == 1 {
        return Some(Closest {
            point: p,
            weights,
            mask,
        });
    }
    let mut edges = [DVec3::ZERO; 3];
    let scale = vertices
        .iter()
        .map(|v| v.point.length())
        .fold(0., f64::max)
        .max(1e-120);
    for i in 0..count - 1 {
        edges[i] = (vertices[ids[i + 1]].point - p) / scale;
    }
    if count == 4 {
        // Normal equations square the condition number and discard resolvable
        // thin tetrahedra. Solve their affine coordinates directly instead.
        let [x, y, z] = edges;
        let determinant = x.dot(y.cross(z));
        if !determinant.is_finite()
            || determinant.abs() <= x.length() * y.length() * z.length() * 1e-14
        {
            return None;
        }
        let target = -p / scale;
        weights[ids[1]] = target.dot(y.cross(z)) / determinant;
        weights[ids[2]] = x.dot(target.cross(z)) / determinant;
        weights[ids[3]] = x.dot(y.cross(target)) / determinant;
        weights[ids[0]] = 1. - weights[ids[1]] - weights[ids[2]] - weights[ids[3]];
        return weighted(vertices, weights);
    }
    let mut matrix = [[0.; 4]; 3];
    for row in 0..count - 1 {
        for column in 0..count - 1 {
            matrix[row][column] = edges[row].dot(edges[column]);
        }
        matrix[row][count - 1] = -edges[row].dot(p / scale);
    }
    let magnitude = matrix
        .iter()
        .flat_map(|row| row[..count - 1].iter())
        .map(|v| v.abs())
        .fold(0., f64::max);
    for column in 0..count - 1 {
        let pivot = (column..count - 1)
            .max_by(|&a, &b| matrix[a][column].abs().total_cmp(&matrix[b][column].abs()))?;
        if matrix[pivot][column].abs() <= magnitude * 1e-14 {
            return None;
        }
        matrix.swap(column, pivot);
        let factor = matrix[column][column];
        for value in &mut matrix[column][column..count] {
            *value /= factor;
        }
        let pivot = matrix[column];
        for (row, values) in matrix.iter_mut().enumerate().take(count - 1) {
            if row != column {
                let factor = values[column];
                for (value, pivot) in values[column..count].iter_mut().zip(&pivot[column..count]) {
                    *value -= factor * pivot;
                }
            }
        }
    }
    for i in 0..count - 1 {
        weights[ids[i + 1]] = matrix[i][count - 1];
        weights[ids[0]] -= weights[ids[i + 1]];
    }
    let mut result = weighted(vertices, weights)?;
    if count == 3 {
        // Keep geometric plane projection separate from witness reconstruction:
        // its direction must not contain tangential barycentric cancellation.
        let normal = edges[0].cross(edges[1]).try_normalize()?;
        result.point = normal * normal.dot(p);
    }
    Some(result)
}
fn weighted(vertices: &[Vertex], weights: [f64; 4]) -> Option<Closest> {
    if weights.iter().any(|w| !w.is_finite() || *w < 0.) {
        return None;
    }
    let mut point = DVec3::ZERO;
    for i in 0..vertices.len() {
        point += vertices[i].point * weights[i];
    }
    let mask = weights.iter().enumerate().fold(0, |mask, (index, weight)| {
        if *weight > 0. {
            mask | (1 << index)
        } else {
            mask
        }
    });
    Some(Closest {
        point,
        weights,
        mask,
    })
}
fn closest(vertices: &[Vertex]) -> Closest {
    let mut best = Closest {
        point: vertices[0].point,
        weights: [1., 0., 0., 0.],
        mask: 1,
    };
    for mask in 1..1 << vertices.len() {
        if let Some(candidate) = affine(vertices, mask) {
            if candidate.point.length_squared() < best.point.length_squared() {
                best = candidate;
            }
        }
    }
    best
}
fn witnesses(vertices: &[Vertex], weights: [f64; 4]) -> [DVec3; 2] {
    let mut result = [DVec3::ZERO; 2];
    for (vertex, weight) in vertices.iter().zip(weights) {
        result[0] += vertex.a * weight;
        result[1] += vertex.b * weight;
    }
    result
}
fn contact(normal: DVec3, points: [DVec3; 2], depth: f64) -> Result<Option<Contact>, Issue> {
    let normal = normal.as_vec3().try_normalize().ok_or(Issue::Degenerate)?;
    let points = points.map(DVec3::as_vec3);
    let depth = depth as f32;
    if points.iter().any(|p| !p.is_finite()) || !depth.is_finite() || depth < 0. {
        return Err(Issue::InvalidInput);
    }
    Ok(Some(Contact {
        normal,
        points,
        depth,
    }))
}
#[derive(Clone, Copy, Default)]
struct Face {
    vertices: [usize; 3],
    normal: DVec3,
    closest: DVec3,
    weights: [f64; 4],
    active: bool,
}
impl Face {
    fn new(
        indices: [usize; 3],
        vertices: &[Vertex],
        opposite: Option<usize>,
    ) -> Result<Self, Issue> {
        let [a, b, c] = indices.map(|i| vertices[i].point);
        let raw = (b - a).cross(c - a);
        let Some(mut normal) = raw.try_normalize() else {
            return Err(Issue::Degenerate);
        };
        let mut indices = indices;
        if opposite.map_or(normal.dot(a) < 0., |opposite| {
            normal.dot(vertices[opposite].point - a) > 0.
        }) {
            normal = -normal;
            indices.swap(1, 2);
        }
        let triangle = indices.map(|i| vertices[i]);
        let nearest = closest(&triangle);
        Ok(Self {
            vertices: indices,
            normal,
            closest: nearest.point,
            weights: nearest.weights,
            active: true,
        })
    }
}
/// Fixed topology storage is owned by a physics instance, never grown in a query.
pub(super) struct Scratch {
    vertices: [Vertex; MAX_VERTICES],
    faces: [Face; MAX_FACES],
    edges: [[usize; 2]; MAX_EDGES],
    #[cfg(test)]
    max_faces: usize,
    #[cfg(test)]
    max_vertices: usize,
    #[cfg(test)]
    epa_steps: usize,
}
impl Default for Scratch {
    fn default() -> Self {
        Self {
            vertices: [Vertex::default(); MAX_VERTICES],
            faces: [Face::default(); MAX_FACES],
            edges: [[0; 2]; MAX_EDGES],
            #[cfg(test)]
            max_faces: MAX_FACES,
            #[cfg(test)]
            max_vertices: MAX_VERTICES,
            #[cfg(test)]
            epa_steps: EPA_STEPS,
        }
    }
}
fn seed_tetra(
    scratch: &mut Scratch,
    count: usize,
    last: bool,
    tolerance: f64,
) -> Result<bool, Issue> {
    for i in 0..count {
        for j in i + 1..count {
            for k in j + 1..count {
                for l in k + 1..count {
                    if last && l != count - 1 {
                        continue;
                    }
                    let tetra = [
                        scratch.vertices[i],
                        scratch.vertices[j],
                        scratch.vertices[k],
                        scratch.vertices[l],
                    ];
                    if let Some(candidate) = affine(&tetra, 15) {
                        if candidate.point.length() <= tolerance {
                            scratch.vertices[..4].copy_from_slice(&tetra);
                            scratch.faces.fill(Face::default());
                            for (index, (vertices, opposite)) in [
                                ([0, 1, 2], 3),
                                ([0, 3, 1], 2),
                                ([0, 2, 3], 1),
                                ([1, 3, 2], 0),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                scratch.faces[index] =
                                    Face::new(vertices, &scratch.vertices[..4], Some(opposite))?;
                            }
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}
fn seed(
    a: ShapeRef<'_>,
    b: ShapeRef<'_>,
    simplex: &[Vertex],
    scratch: &mut Scratch,
    work: &mut ContactWork,
    tolerance: f64,
) -> Result<(), Issue> {
    let mut count = simplex.len();
    scratch.vertices[..count].copy_from_slice(simplex);
    if count == 4 && seed_tetra(scratch, count, false, tolerance)? {
        return Ok(());
    }
    let directions = [
        DVec3::X,
        -DVec3::X,
        DVec3::Y,
        -DVec3::Y,
        DVec3::Z,
        -DVec3::Z,
        DVec3::new(1., 1., 1.),
        DVec3::new(1., -1., -1.),
        DVec3::new(-1., 1., -1.),
        DVec3::new(-1., -1., 1.),
    ];
    for direction in directions {
        let vertex = support(a, b, direction.normalize(), work)?;
        if scratch.vertices[..count]
            .iter()
            .all(|old| (old.point - vertex.point).length() > tolerance * 0.01)
        {
            if count >= MAX_SEED {
                return Err(Issue::CapacityLimit);
            }
            scratch.vertices[count] = vertex;
            count += 1;
            if count >= 4 && seed_tetra(scratch, count, true, tolerance)? {
                return Ok(());
            }
        }
    }
    Err(Issue::Degenerate)
}
fn epa(
    a: ShapeRef<'_>,
    b: ShapeRef<'_>,
    simplex: &[Vertex],
    scratch: &mut Scratch,
    work: &mut ContactWork,
    tolerance: f64,
) -> Result<Option<Contact>, Issue> {
    seed(a, b, simplex, scratch, work, tolerance)?;
    #[cfg(not(test))]
    let (max_faces, max_vertices, epa_steps) = (MAX_FACES, MAX_VERTICES, EPA_STEPS);
    #[cfg(test)]
    let (max_faces, max_vertices, epa_steps) =
        (scratch.max_faces, scratch.max_vertices, scratch.epa_steps);
    for vertices in (4..).take(epa_steps) {
        let face = scratch
            .faces
            .iter()
            .filter(|face| face.active)
            .min_by(|a, b| {
                a.closest
                    .length_squared()
                    .total_cmp(&b.closest.length_squared())
            })
            .copied()
            .ok_or(Issue::Degenerate)?;
        let distance = face.closest.length();
        let normal = if distance > tolerance * 0.01 {
            face.closest / distance
        } else {
            face.normal
        };
        let vertex = support(a, b, normal, work)?;
        let gap = normal.dot(vertex.point) - distance;
        if gap < -tolerance {
            return Err(Issue::Degenerate);
        }
        if gap <= tolerance {
            let triangle = face.vertices.map(|i| scratch.vertices[i]);
            return contact(normal, witnesses(&triangle, face.weights), distance);
        }
        if vertices >= max_vertices {
            return Err(Issue::CapacityLimit);
        }
        if scratch.vertices[..vertices]
            .iter()
            .any(|old| (old.point - vertex.point).length() <= tolerance * 0.01)
        {
            return Err(Issue::Degenerate);
        }
        let mut edges = 0;
        let mut removed = 0;
        for face in &scratch.faces {
            if face.active
                && face
                    .normal
                    .dot(vertex.point - scratch.vertices[face.vertices[0]].point)
                    > tolerance * 0.01
            {
                removed += 1;
                for edge in [
                    [face.vertices[0], face.vertices[1]],
                    [face.vertices[1], face.vertices[2]],
                    [face.vertices[2], face.vertices[0]],
                ] {
                    if let Some(index) = scratch.edges[..edges]
                        .iter()
                        .position(|old| *old == [edge[1], edge[0]])
                    {
                        scratch.edges.copy_within(index + 1..edges, index);
                        edges -= 1;
                    } else {
                        if edges == MAX_EDGES {
                            return Err(Issue::CapacityLimit);
                        }
                        scratch.edges[edges] = edge;
                        edges += 1;
                    }
                }
            }
        }
        if removed == 0 || edges == 0 {
            return Err(Issue::Degenerate);
        }
        let active = scratch.faces.iter().filter(|face| face.active).count();
        if active - removed + edges > max_faces {
            return Err(Issue::CapacityLimit);
        }
        for face in &mut scratch.faces {
            if face.active
                && face
                    .normal
                    .dot(vertex.point - scratch.vertices[face.vertices[0]].point)
                    > tolerance * 0.01
            {
                face.active = false;
            }
        }
        scratch.vertices[vertices] = vertex;
        for edge in scratch.edges[..edges].iter().copied() {
            let index = scratch
                .faces
                .iter()
                .position(|face| !face.active)
                .ok_or(Issue::CapacityLimit)?;
            scratch.faces[index] = Face::new(
                [edge[0], edge[1], vertices],
                &scratch.vertices[..vertices + 1],
                None,
            )?;
        }
    }
    Err(Issue::IterationLimit)
}
/// The caller owns query admission and records any returned typed issue.
pub(super) fn query(
    a: ShapeRef<'_>,
    b: ShapeRef<'_>,
    margin: f64,
    scale: f64,
    scratch: &mut Scratch,
    work: &mut ContactWork,
) -> Result<Option<Contact>, Issue> {
    if !margin.is_finite() || margin < 0. || !scale.is_finite() || scale <= 0. {
        return Err(Issue::InvalidInput);
    }
    let tolerance = (scale * RELATIVE_TOLERANCE).max(1e-120);
    let mut simplex = [Vertex::default(); 4];
    let mut direction = (b.translation - a.translation)
        .try_normalize()
        .unwrap_or(DVec3::X);
    simplex[0] = support(a, b, direction, work)?;
    let mut count = 1;
    for _ in 0..GJK_STEPS {
        if count == 4
            && affine(&simplex[..count], 15)
                .is_some_and(|candidate| candidate.point.length() <= tolerance)
        {
            // Do not discard containment because a face's floating residual
            // happens to be smaller than the full tetrahedron's residual.
            return epa(a, b, &simplex[..count], scratch, work, tolerance);
        }
        let nearest = closest(&simplex[..count]);
        let distance = nearest.point.length();
        if distance == 0. {
            // A supporting plane through the origin certifies a boundary,
            // rather than requiring a strictly enclosing EPA seed at a touch.
            let boundary = support(a, b, direction, work)?;
            if direction.dot(boundary.point) == 0. {
                return contact(direction, witnesses(&simplex[..count], nearest.weights), 0.);
            }
            return epa(a, b, &simplex[..count], scratch, work, tolerance);
        }
        direction = -nearest.point / distance;
        let vertex = support(a, b, direction, work)?;
        let lower = -direction.dot(vertex.point);
        if distance <= tolerance {
            // A tiny closest witness is not proof of intersection. A positive
            // support-plane lower bound certifies separation even below the
            // scale-relative tolerance, especially for mixed-size fixtures.
            if lower > 0. {
                if distance <= margin {
                    return contact(direction, witnesses(&simplex[..count], nearest.weights), 0.);
                }
                if lower > margin {
                    return Ok(None);
                }
            }
            // A negative lower bound and a small simplex distance do not
            // establish an enclosing simplex. Continue GJK refinement instead
            // of asking a fixed set of seed directions to find a tiny cap.
        }
        if lower > margin + tolerance {
            return Ok(None);
        }
        if distance - lower <= tolerance && lower > 0. {
            // Distance convergence alone does not prove touching or overlap.
            // Only a certified separated pair whose upper distance fits the
            // actual margin can produce a zero-depth margin contact here.
            if distance <= margin {
                return contact(direction, witnesses(&simplex[..count], nearest.weights), 0.);
            }
            if lower > margin {
                return Ok(None);
            }
        }
        let mut next = [Vertex::default(); 4];
        let mut kept = 0;
        for (index, point) in simplex[..count].iter().copied().enumerate() {
            if nearest.mask & (1 << index) != 0 {
                next[kept] = point;
                kept += 1;
            }
        }
        if kept == 4 {
            // All four positive barycentric coordinates establish containment;
            // a small floating residual need not be exactly zero.
            return epa(a, b, &simplex[..count], scratch, work, tolerance);
        }
        if next[..kept]
            .iter()
            .any(|old| (old.point - vertex.point).length() <= tolerance * 0.01)
        {
            return Err(Issue::Degenerate);
        }
        next[kept] = vertex;
        simplex = next;
        count = kept + 1;
    }
    Err(Issue::IterationLimit)
}

#[cfg(test)]
#[path = "convex_tests.rs"]
mod tests;
