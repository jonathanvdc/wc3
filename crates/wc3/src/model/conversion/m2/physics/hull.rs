//! Deterministic convex-boundary reconstruction for native polytope topology.
use super::*;
use native::companions::{PhysicsEdge, Polytope, PolytopeHeader};
fn difference(a: [f32; 3], b: [f32; 3]) -> [f64; 3] {
    from_fn(|i| f64::from(a[i]) - f64::from(b[i]))
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn plane(points: &[[f32; 3]], triangle: [usize; 3]) -> [f64; 3] {
    cross(
        difference(points[triangle[1]], points[triangle[0]]),
        difference(points[triangle[2]], points[triangle[0]]),
    )
}
fn orient(points: &[[f32; 3]], mut face: [usize; 3], inside: [f32; 3]) -> [usize; 3] {
    if dot(plane(points, face), difference(inside, points[face[0]])) > 0. {
        face.swap(1, 2);
    }
    face
}
pub(super) fn hull(points: Vec<[f32; 3]>) -> Result<Polytope<'static>, Error> {
    if points.len() < 4 || points.len() > 256 || points.iter().flatten().any(|v| !v.is_finite()) {
        return Err(Error::new(
            "physics.hull.points",
            "native hull needs four to 256 finite points",
        ));
    }
    let a = 0;
    let b = (1..points.len())
        .max_by(|&a, &b| {
            dot(
                difference(points[a], points[0]),
                difference(points[a], points[0]),
            )
            .total_cmp(&dot(
                difference(points[b], points[0]),
                difference(points[b], points[0]),
            ))
        })
        .unwrap();
    // Select the third point by distance from the initial edge.
    let edge = difference(points[b], points[a]);
    let c = (0..points.len())
        .filter(|&i| i != a && i != b)
        .max_by(|&i, &j| {
            let x = cross(edge, difference(points[i], points[a]));
            let y = cross(edge, difference(points[j], points[a]));
            dot(x, x).total_cmp(&dot(y, y))
        })
        .unwrap();
    let normal = plane(&points, [a, b, c]);
    let d = (0..points.len())
        .filter(|&i| i != a && i != b && i != c)
        .max_by(|&i, &j| {
            dot(normal, difference(points[i], points[a]))
                .abs()
                .total_cmp(&dot(normal, difference(points[j], points[a])).abs())
        })
        .unwrap();
    if dot(normal, difference(points[d], points[a])) == 0. {
        return Err(Error::new(
            "physics.hull.points",
            "points do not enclose a three-dimensional hull",
        ));
    }
    let inside = from_fn(|i| (points[a][i] + points[b][i] + points[c][i] + points[d][i]) * 0.25);
    let mut faces = vec![
        orient(&points, [a, b, c], inside),
        orient(&points, [a, d, b], inside),
        orient(&points, [a, c, d], inside),
        orient(&points, [b, d, c], inside),
    ];
    for index in 0..points.len() {
        if [a, b, c, d].contains(&index) {
            continue;
        }
        let visible: Vec<_> = faces
            .iter()
            .enumerate()
            .filter_map(|(i, &face)| {
                (dot(
                    plane(&points, face),
                    difference(points[index], points[face[0]]),
                ) > 0.)
                    .then_some(i)
            })
            .collect();
        if visible.is_empty() {
            continue;
        }
        let mut boundary = BTreeMap::new();
        for &i in &visible {
            let [u, v, w] = faces[i];
            for edge in [(u, v), (v, w), (w, u)] {
                if boundary.remove(&(edge.1, edge.0)).is_none() {
                    boundary.insert(edge, ());
                }
            }
        }
        faces = faces
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !visible.contains(i))
            .map(|(_, f)| f)
            .collect();
        for ((u, v), _) in boundary {
            faces.push(orient(&points, [u, v, index], inside));
        }
    }
    if faces.len() * 3 > 256 {
        return Err(Error::new(
            "physics.hull.topology",
            "triangulated half-edge count exceeds native byte indices",
        ));
    }
    let mut planes = Vec::new();
    let mut edges = Vec::new();
    let mut first = Vec::new();
    let mut pairs = BTreeMap::new();
    let mut volume = 0f64;
    let mut surface = 0f64;
    let mut center = [0f64; 3];
    for (face, &[a, b, c]) in faces.iter().enumerate() {
        let normal = plane(&points, [a, b, c]);
        let length = dot(normal, normal).sqrt();
        if length == 0. {
            return Err(Error::new("physics.hull.topology", "degenerate hull face"));
        }
        let n = normal.map(|v| v / length);
        planes.push([
            n[0] as f32,
            n[1] as f32,
            n[2] as f32,
            -dot(n, points[a].map(f64::from)) as f32,
        ]);
        surface += length * 0.5;
        let v = dot(
            difference(points[a], inside),
            cross(difference(points[b], inside), difference(points[c], inside)),
        ) / 6.;
        volume += v;
        for i in 0..3 {
            center[i] += v
                * (f64::from(points[a][i])
                    + f64::from(points[b][i])
                    + f64::from(points[c][i])
                    + f64::from(inside[i]))
                * 0.25;
        }
        let start = edges.len();
        first.push(start as u8);
        for (index, (from, to)) in [(a, b), (b, c), (c, a)].into_iter().enumerate() {
            pairs.insert((from, to), start + index);
            edges.push(PhysicsEdge {
                origin_vertex: from as u8,
                face_index: face as u8,
                next_edge: (start + (index + 1) % 3) as u8,
                ..Default::default()
            });
        }
    }
    let mut order = Vec::new();
    let mut inverse = vec![usize::MAX; edges.len()];
    for (&(from, to), &index) in &pairs {
        if inverse[index] != usize::MAX {
            continue;
        }
        let twin = *pairs
            .get(&(to, from))
            .ok_or_else(|| Error::new("physics.hull.topology", "hull boundary is not closed"))?;
        inverse[index] = order.len();
        order.push(index);
        inverse[twin] = order.len();
        order.push(twin);
    }
    let old = edges;
    let mut edges = Vec::new();
    for (index, &original) in order.iter().enumerate() {
        let mut edge = old[original].clone();
        edge.next_edge = inverse[usize::from(edge.next_edge)] as u8;
        edge.twin_offset = if index % 2 == 0 { 1 } else { -1 };
        edges.push(edge);
    }
    let first: Vec<_> = first
        .into_iter()
        .map(|index| inverse[usize::from(index)] as u8)
        .collect();

    if volume <= 0. {
        return Err(Error::new(
            "physics.hull.volume",
            "convex boundary has no positive volume",
        ));
    }
    Ok(Polytope {
        header: PolytopeHeader {
            centroid: center.map(|v| (v / volume) as f32),
            volume: volume as f32,
            surface_area: surface as f32,
            ..Default::default()
        },
        vertices: ZeroVec::alloc_from_slice(&points),
        planes: ZeroVec::alloc_from_slice(&planes),
        face_first_edge: ZeroVec::alloc_from_slice(&first),
        edges,
    })
}
