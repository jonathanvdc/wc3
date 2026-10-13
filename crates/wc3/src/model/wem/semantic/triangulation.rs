//! Polygon triangulation in the dominant plane, preserving the source winding.
use super::ReadError;

pub(super) fn triangulate(
    face: &[u32],
    positions: &[[f32; 3]],
) -> Result<Vec<[u32; 3]>, ReadError> {
    if face.len() < 3 || face.iter().any(|&i| i as usize >= positions.len()) {
        return Err(ReadError::invalid(0, "polygon vertex loop"));
    }
    let mut normal = [0f64; 3];
    for i in 0..face.len() {
        let a = positions[face[i] as usize].map(f64::from);
        let b = positions[face[(i + 1) % face.len()] as usize].map(f64::from);
        if !a.iter().all(|v| v.is_finite()) {
            return Err(ReadError::invalid(0, "polygon position"));
        }
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let axis = (0..3)
        .max_by(|&a, &b| normal[a].abs().total_cmp(&normal[b].abs()))
        .unwrap();
    let points: Vec<_> = face
        .iter()
        .map(|&i| {
            let p = positions[i as usize].map(f64::from);
            [p[(axis + 1) % 3], p[(axis + 2) % 3]]
        })
        .collect();
    let area: f64 = (0..points.len())
        .map(|i| {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    if area == 0.0 {
        return Err(ReadError::invalid(0, "degenerate polygon"));
    }
    let sign = area.signum();
    let mut remaining: Vec<_> = (0..face.len()).collect();
    let mut triangles = Vec::with_capacity(face.len() - 2);
    while remaining.len() > 3 {
        let mut ear = None;
        for i in 0..remaining.len() {
            let a = remaining[(i + remaining.len() - 1) % remaining.len()];
            let b = remaining[i];
            let c = remaining[(i + 1) % remaining.len()];
            if cross(points[a], points[b], points[c]) * sign <= 0.0 {
                continue;
            }
            let contains = remaining
                .iter()
                .copied()
                .filter(|&p| p != a && p != b && p != c)
                .any(|p| {
                    cross(points[a], points[b], points[p]) * sign >= 0.0
                        && cross(points[b], points[c], points[p]) * sign >= 0.0
                        && cross(points[c], points[a], points[p]) * sign >= 0.0
                });
            if !contains {
                ear = Some((i, [face[a], face[b], face[c]]));
                break;
            }
        }
        let Some((i, triangle)) = ear else {
            return Err(ReadError::invalid(0, "polygon cannot be triangulated"));
        };
        triangles.push(triangle);
        remaining.remove(i);
    }
    triangles.push([face[remaining[0]], face[remaining[1]], face[remaining[2]]]);
    Ok(triangles)
}
fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concave_polygon_preserves_area_and_winding() {
        let p = [
            [0., 0., 0.],
            [2., 0., 0.],
            [2., 2., 0.],
            [1., 1., 0.],
            [0., 2., 0.],
        ];
        for (face, sign) in [(vec![0, 1, 2, 3, 4], 1.), (vec![4, 3, 2, 1, 0], -1.)] {
            let triangles = triangulate(&face, &p).unwrap();
            assert_eq!(triangles.len(), 3);
            let areas: Vec<_> = triangles
                .iter()
                .map(|t| {
                    let xy = |i: u32| [f64::from(p[i as usize][0]), f64::from(p[i as usize][1])];
                    cross(xy(t[0]), xy(t[1]), xy(t[2])) * sign
                })
                .collect();
            assert!(areas.iter().all(|a| *a > 0.));
            assert_eq!(areas.iter().sum::<f64>(), 6.);
        }
        assert!(triangulate(&[0, 1, 10], &p).is_err());
    }
}
