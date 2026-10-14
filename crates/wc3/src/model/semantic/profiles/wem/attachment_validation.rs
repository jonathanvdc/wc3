//! Surface addresses are checked against the primitive ordering actually owned by geometry.
use crate::model::semantic::Topology;
pub(super) fn usable_face(topology: &Topology<'_>, mut face: usize) -> bool {
    match topology {
        Topology::Triangles(rows) => face < rows.len(),
        Topology::Polygons { offsets, .. } => offsets
            .get(face)
            .zip(offsets.get(face + 1))
            .is_some_and(|(first, last)| last.saturating_sub(first) >= 3),
        Topology::Lines(_) | Topology::Points(_) => false,
        Topology::Mixed(groups) => {
            for group in groups {
                let count = group.primitive_count();
                if face < count {
                    return usable_face(group, face);
                }
                face -= count;
            }
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::semantic::Values;
    #[test]
    fn surface_faces_follow_mixed_primitive_order_and_require_three_corners() {
        let topology = Topology::Mixed(vec![
            Topology::Lines(Values::owned(vec![[0, 1]])),
            Topology::Polygons {
                corners: Values::owned(vec![0, 1, 2, 0, 1]),
                offsets: Values::owned(vec![0, 3, 5]),
            },
        ]);
        assert!(!usable_face(&topology, 0));
        assert!(usable_face(&topology, 1));
        assert!(!usable_face(&topology, 2));
        assert!(!usable_face(&topology, 3));
    }
}
