use super::*;
use std::cell::Cell;
#[derive(Default)]
struct Graph {
    local: HashMap<Entity, (Option<Entity>, Mat4)>,
    owners: HashMap<Entity, NodePoseSource>,
    rigs: HashMap<Entity, (Entity, Vec<Entity>, Vec<Mat4>)>,
    reads: Cell<usize>,
}
impl CurrentTransformSource for Graph {
    fn edge(&self, entity: Entity) -> Option<(Option<Entity>, Mat4)> {
        self.reads.set(self.reads.get() + 1);
        if let Some(source) = self.owners.get(&entity) {
            let (basis, nodes, poses) = self.rigs.get(&source.rig)?;
            (*nodes.get(source.index)? == entity).then(|| (Some(*basis), poses[source.index]))
        } else {
            self.local.get(&entity).copied()
        }
    }
}

fn entities<const N: usize>() -> [Entity; N] {
    let mut world = World::new();
    std::array::from_fn(|_| world.spawn_empty().id())
}
#[test]
fn current_root_moves_before_propagation_and_each_frame_discards_old_worlds() {
    let [root, child] = entities();
    let mut graph = Graph::default();
    let mut cache = CurrentAffines::default();
    graph
        .local
        .insert(root, (None, Mat4::from_translation(Vec3::X)));
    graph
        .local
        .insert(child, (Some(root), Mat4::from_translation(Vec3::Y)));
    assert_eq!(
        cache.resolve(child, &graph).unwrap().w_axis.truncate(),
        Vec3::new(1., 1., 0.)
    );
    let reads = graph.reads.get();
    assert!(cache.resolve(child, &graph).is_some());
    assert_eq!(graph.reads.get(), reads);
    graph
        .local
        .insert(root, (None, Mat4::from_translation(Vec3::X * 4.)));
    cache.begin_frame();
    assert_eq!(
        cache.resolve(child, &graph).unwrap().w_axis.truncate(),
        Vec3::new(4., 1., 0.)
    );
}
#[test]
fn corrected_mounted_rig_preserves_exact_shear_and_skips_informational_bone_edges() {
    let [root, basis, bone, rig, mount, child_basis, child_bone, child_rig, particle] = entities();
    let mut graph = Graph::default();
    let mut cache = CurrentAffines::default();
    let shear = Mat4::from_cols(Vec4::X, Vec4::new(0.7, 1., 0., 0.), Vec4::Z, Vec4::W);
    let root_world = Mat4::from_rotation_z(0.4) * Mat4::from_translation(Vec3::Y * 3.);
    let correction = Mat4::from_scale(Vec3::new(2., 3., 4.));
    let mount_local = Mat4::from_translation(Vec3::X * 2.);
    let child_pose = Mat4::from_rotation_x(0.8);
    graph.local.extend([
        (root, (None, root_world)),
        (basis, (Some(root), correction)),
        (bone, (Some(basis), Mat4::ZERO)),
        (mount, (Some(bone), mount_local)),
        (child_basis, (Some(mount), Mat4::IDENTITY)),
        (child_bone, (Some(child_basis), Mat4::ZERO)),
        (
            particle,
            (Some(child_bone), Mat4::from_translation(Vec3::Z)),
        ),
    ]);
    graph.owners.insert(bone, NodePoseSource { rig, index: 0 });
    graph.rigs.insert(rig, (basis, vec![bone], vec![shear]));
    graph.owners.insert(
        child_bone,
        NodePoseSource {
            rig: child_rig,
            index: 0,
        },
    );
    graph
        .rigs
        .insert(child_rig, (child_basis, vec![child_bone], vec![child_pose]));
    let expected = root_world
        * correction
        * shear
        * mount_local
        * child_pose
        * Mat4::from_translation(Vec3::Z);
    assert!(cache
        .resolve(particle, &graph)
        .unwrap()
        .abs_diff_eq(expected, 1e-6));
}
#[test]
fn stale_owner_is_rejected_after_reload_without_falling_back_to_lossy_transform() {
    let [basis, bone, rig, replacement] = entities();
    let mut graph = Graph::default();
    let mut cache = CurrentAffines::default();
    graph.local.insert(basis, (None, Mat4::IDENTITY));
    graph.local.insert(bone, (Some(basis), Mat4::IDENTITY));
    graph.owners.insert(bone, NodePoseSource { rig, index: 0 });
    graph
        .rigs
        .insert(rig, (basis, vec![replacement], vec![Mat4::IDENTITY]));
    assert!(cache.resolve(bone, &graph).is_none());
    cache.begin_frame();
    graph.rigs.get_mut(&rig).unwrap().1[0] = bone;
    assert!(cache.resolve(bone, &graph).is_some());
}
#[test]
fn malformed_ordinary_and_cross_rig_cycles_are_bounded() {
    let [a, b, rig] = entities();
    let mut graph = Graph::default();
    let mut cache = CurrentAffines::default();
    graph.local.extend([
        (a, (Some(b), Mat4::IDENTITY)),
        (b, (Some(a), Mat4::IDENTITY)),
    ]);
    assert!(cache.resolve(a, &graph).is_none());
    cache.begin_frame();
    graph.owners.insert(a, NodePoseSource { rig, index: 0 });
    graph.rigs.insert(rig, (b, vec![a], vec![Mat4::IDENTITY]));
    assert!(cache.resolve(a, &graph).is_none());
}
