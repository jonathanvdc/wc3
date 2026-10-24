//! Collision membership is the set of faces reachable through the BSP root.
use crate::model::conversion::{Context, Error, IssueKind};
use crate::model::error::Location;
use crate::model::formats::wmo::group::GroupChunk;
use crate::model::formats::wmo::Dialect;
use std::collections::BTreeSet;

pub(super) fn faces<'a, 'data: 'a, D: Dialect>(
    chunks: impl Iterator<Item = &'a GroupChunk<'data, D>>,
    context: &mut Context,
) -> Result<BTreeSet<usize>, Error> {
    let chunks: Vec<_> = chunks.collect();
    let nodes = chunks.iter().find_map(|chunk| match chunk {
        GroupChunk::BspNodes(values) => Some(values),
        _ => None,
    });
    let references = chunks.iter().find_map(|chunk| match chunk {
        GroupChunk::BspFaces(values) => Some(values),
        _ => None,
    });
    let mut faces = BTreeSet::new();
    let Some(references) = references else {
        return Ok(faces);
    };
    let Some(nodes) = nodes else {
        return Err(Error::requirement(
            Location::new().field("world").field("query"),
            "BSP face references have no tree",
        ));
    };
    let mut visited = BTreeSet::new();
    let mut used = BTreeSet::new();
    let mut pending = if nodes.is_empty() {
        Vec::new()
    } else {
        vec![0usize]
    };
    while let Some(index) = pending.pop() {
        if !visited.insert(index) {
            continue;
        }
        let node = nodes
            .get(index)
            .ok_or_else(|| {
                Error::requirement(
                    Location::new().field("world").field("query"),
                    "BSP child is outside the tree",
                )
            })?
            .map_err(|error| {
                Error::from_source(Location::new().field("world").field("query"), error)
            })?;
        if node.flags.leaf() {
            let start = usize::try_from(node.first_face).map_err(|_| {
                Error::requirement(
                    Location::new().field("world").field("query"),
                    "BSP window exceeds addressable storage",
                )
            })?;
            let end = start
                .checked_add(usize::from(node.face_count))
                .ok_or_else(|| {
                    Error::requirement(
                        Location::new().field("world").field("query"),
                        "BSP window overflows",
                    )
                })?;
            if end > references.len() {
                return Err(Error::requirement(
                    Location::new().field("world").field("query"),
                    "BSP window exceeds face references",
                ));
            }
            for slot in start..end {
                used.insert(slot);
                let face = references
                    .get(slot)
                    .expect("checked BSP window")
                    .map_err(|error| {
                        Error::from_source(Location::new().field("world").field("query"), error)
                    })?;
                faces.insert(usize::from(face));
            }
        } else {
            for child in [node.negative_child, node.positive_child] {
                if child >= 0 {
                    pending.push(child as usize);
                }
            }
        }
    }
    if visited.len() != nodes.len() || used.len() != references.len() {
        context.issue("world.query", IssueKind::Normalized, "unreachable BSP nodes and unused face-reference entries omitted from the logical collision membership");
    }
    Ok(faces)
}

#[cfg(test)]
mod tests {
    use super::super::encode::array;
    use super::*;
    use crate::model::conversion::ImportOptions;
    use crate::model::formats::wmo::records::{BspFlags, BspNode};
    use crate::model::formats::wmo::Classic;

    #[test]
    fn inactive_tree_nodes_and_reference_slots_do_not_expand_membership() {
        let mut flags = BspFlags::default();
        flags.set_leaf(true);
        let nodes = [
            BspNode {
                flags,
                negative_child: -1,
                positive_child: -1,
                face_count: 1,
                first_face: 0,
                plane_distance: 0.0,
            },
            BspNode {
                flags,
                negative_child: -1,
                positive_child: -1,
                face_count: 1,
                first_face: 1,
                plane_distance: 0.0,
            },
        ];
        let chunks: Vec<GroupChunk<'_, Classic>> = vec![
            GroupChunk::BspNodes(array(&nodes).unwrap()),
            GroupChunk::BspFaces(array(&[3u16, 9, 12]).unwrap()),
        ];
        let mut context = Context::import(&ImportOptions::default());
        assert_eq!(
            faces(chunks.iter(), &mut context).unwrap(),
            BTreeSet::from([3])
        );
        assert!(context
            .report
            .issues
            .iter()
            .any(|issue| issue.kind == IssueKind::Normalized));
    }
}
