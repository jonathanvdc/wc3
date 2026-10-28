//! Semantic body-particle candidates, independent of native record layouts.
use bevy::math::Vec3;
use std::collections::{HashMap, HashSet};
use std::f32::consts::TAU;
use std::sync::Arc;
use wc3::model::{MeshProperties, Model, NodeId, NodeProperties, VertexInfluences};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BoneCandidate {
    pub node: usize,
    pub pivot: Vec3,
    pub parent: Option<(usize, Vec3)>,
}
#[derive(Clone, Default)]
pub(super) struct BoneSource {
    candidates: Arc<[BoneCandidate]>,
    pub inferred: bool,
    pub first_lod: Option<u32>,
}
impl BoneSource {
    pub fn compile(model: &Model<'_>, indices: &HashMap<NodeId, usize>) -> Self {
        let first_lod = model
            .instances()
            .iter()
            .filter_map(|(_, instance)| instance.lod)
            .min();
        let mut candidates = HashSet::new();
        let mut inferred = false;
        for (_, instance) in model.instances() {
            if instance.lod.is_some() && instance.lod != first_lod {
                continue;
            }
            let Some(mesh) = model.mesh(instance.mesh) else {
                continue;
            };
            let declared = match &mesh.properties {
                Some(MeshProperties::M2(properties)) => properties.bone_palette.as_deref(),
                _ => None,
            };
            if let Some(declared) = declared {
                candidates.extend(declared.iter().copied());
                continue;
            }
            if instance.draws.is_empty() {
                continue;
            }
            inferred = true;
            let skin = instance.skin.and_then(|id| model.skin(id));
            for row in mesh.influences().iter() {
                match row {
                    VertexInfluences::Joints(row) => {
                        for influence in row.into_iter().filter(|influence| influence.weight > 0.) {
                            if let Some(node) =
                                skin.and_then(|skin| skin.joints.get(influence.joint as usize))
                            {
                                candidates.insert(*node);
                            }
                        }
                    }
                    VertexInfluences::Nodes(row) => {
                        candidates.extend(
                            row.iter()
                                .filter(|influence| influence.weight > 0.)
                                .map(|influence| influence.node),
                        );
                    }
                }
            }
        }
        let mut candidates: Vec<_> = candidates.into_iter().filter_map(|id| {
            let node = model.node(id)?;
            if matches!(&node.properties, Some(NodeProperties::M2(properties)) if properties.flags.no_body_spawn()) { return None; }
            Some(BoneCandidate { node: *indices.get(&id)?, pivot: node.transform.pivot, parent: node.parent.and_then(|parent| Some((*indices.get(&parent)?, model.node(parent)?.transform.pivot))) })
        }).collect();
        candidates.sort_unstable_by_key(|candidate| candidate.node);
        Self {
            candidates: candidates.into(),
            inferred,
            first_lod,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
    /// Composition remaps a shared candidate set once, even with many emitters.
    pub fn remap_cached(&self, indices: &[usize], cache: &mut HashMap<usize, Self>) -> Self {
        let identity = Arc::as_ptr(&self.candidates) as *const () as usize;
        cache
            .entry(identity)
            .or_insert_with(|| Self {
                candidates: self
                    .candidates
                    .iter()
                    .map(|candidate| BoneCandidate {
                        node: indices[candidate.node],
                        pivot: candidate.pivot,
                        parent: candidate
                            .parent
                            .map(|(parent, pivot)| (indices[parent], pivot)),
                    })
                    .collect(),
                inferred: self.inferred,
                first_lod: self.first_lod,
            })
            .clone()
    }
    /// Sample a posed model-space limb span and a plane containing its direction.
    pub fn sample(
        &self,
        draws: [f32; 4],
        width: f32,
        length: f32,
        mut pose: impl FnMut(BoneCandidate) -> Option<(Vec3, Option<Vec3>)>,
    ) -> Option<Vec3> {
        if self.candidates.is_empty() {
            return None;
        }
        let candidate = self.candidates
            [((draws[0] * self.candidates.len() as f32) as usize).min(self.candidates.len() - 1)];
        let (child, parent) = pose(candidate)?;
        let parent = parent.unwrap_or(child);
        let axis = (child - parent).try_normalize().unwrap_or(Vec3::X);
        let reference = if axis.z.abs() < 0.99 {
            Vec3::Z
        } else {
            Vec3::X
        };
        let side = axis.cross(reference).normalize_or_zero();
        let radius = width + draws[2] * (length - width);
        let (sin, cos) = (draws[3] * TAU).sin_cos();
        let position = child.lerp(parent, draws[1]) + (axis * cos + side * sin) * radius;
        position.is_finite().then_some(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::profiles::{
        m2::{BoneProperties, MeshProperties as SectionProperties},
        Universal,
    };
    use wc3::model::{
        GeometryData, IdentityBinding, Mesh, MeshInstance, ModelBuilder, ModelMetadata, Skin,
        UniversalBinding,
    };

    fn placed_palette(
        builder: &mut ModelBuilder<'static, Universal>,
        owner: NodeId,
        palette: Vec<NodeId>,
        lod: u32,
    ) {
        let mesh =
            builder
                .add_mesh(Mesh::new(GeometryData::empty()).with_properties(Some(
                    MeshProperties::M2(SectionProperties {
                        bone_palette: Some(palette),
                        center_bone: None,
                        center: Vec3::ZERO,
                        sorting: None,
                    }),
                )))
                .unwrap();
        let mut instance = MeshInstance::new(owner, mesh);
        instance.lod = Some(lod);
        builder.add_instance(instance).unwrap();
    }
    #[test]
    fn declared_first_lod_candidates_keep_unused_members_and_skip_exclusions_or_orphan_skins() {
        let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
        let mut parent = builder.new_node("parent");
        parent.transform.pivot = Vec3::new(7., 8., 9.);
        let parent = builder.add_node(parent).unwrap();
        let mut excluded = builder.new_node("excluded");
        let mut behavior = BoneProperties::default();
        behavior.flags.set_no_body_spawn(true);
        excluded.properties = Some(NodeProperties::M2(behavior));
        let excluded = builder.add_node(excluded).unwrap();
        let other_lod = builder.add_named_node("other lod").unwrap();
        let mut unused = builder.new_node("unused palette member");
        unused.transform.pivot = Vec3::new(10., 11., 12.);
        unused.parent = Some(parent);
        let unused = builder.add_node(unused).unwrap();
        let helper = builder.add_named_node("orphan skin helper").unwrap();
        builder
            .add_skin(Skin::new(
                vec![helper],
                UniversalBinding::Identity(IdentityBinding),
            ))
            .unwrap();
        placed_palette(
            &mut builder,
            parent,
            vec![unused, excluded, parent, unused],
            0,
        );
        placed_palette(&mut builder, parent, vec![parent, unused], 0);
        placed_palette(&mut builder, parent, vec![other_lod], 1);
        let model = builder.finish().unwrap();
        let indices = model
            .nodes()
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id, i))
            .collect();
        let source = BoneSource::compile(&model, &indices);
        assert!(!source.inferred);
        assert_eq!(source.first_lod, Some(0));
        assert_eq!(
            source.candidates.as_ref(),
            &[
                BoneCandidate {
                    node: 0,
                    pivot: Vec3::new(7., 8., 9.),
                    parent: None
                },
                BoneCandidate {
                    node: 3,
                    pivot: Vec3::new(10., 11., 12.),
                    parent: Some((0, Vec3::new(7., 8., 9.)))
                }
            ]
        );
        let mut remapped = HashMap::new();
        let a = source.remap_cached(&[8, 9, 10, 11, 12], &mut remapped);
        let b = source.remap_cached(&[8, 9, 10, 11, 12], &mut remapped);
        assert!(Arc::ptr_eq(&a.candidates, &b.candidates));
        assert_eq!(
            a.candidates[1],
            BoneCandidate {
                node: 11,
                pivot: Vec3::new(10., 11., 12.),
                parent: Some((8, Vec3::new(7., 8., 9.)))
            }
        );
    }
    #[test]
    fn span_scatter_contains_the_limb_direction_and_handles_degenerate_parents() {
        let source = BoneSource {
            candidates: Arc::from([BoneCandidate {
                node: 1,
                pivot: Vec3::ZERO,
                parent: Some((0, Vec3::ZERO)),
            }]),
            ..Default::default()
        };
        let posed = |_| Some((Vec3::new(4., 5., 6.), Some(Vec3::new(2., 5., 6.))));
        let center = source.sample([0., 0.25, 0., 0.], 0., 0., posed).unwrap();
        assert!(center.abs_diff_eq(Vec3::new(3.5, 5., 6.), 1e-6));
        let along_limb = source.sample([0., 0.25, 0., 0.], 2., 2., posed).unwrap();
        assert!(along_limb.abs_diff_eq(center + Vec3::X * 2., 1e-6));
        let across_limb = source.sample([0., 0.25, 0., 0.25], 2., 2., posed).unwrap();
        assert!((across_limb - center).dot(Vec3::Z).abs() < 1e-5);
        assert!(((across_limb - center).length() - 2.).abs() < 1e-5);
        for parent in [None, Some(Vec3::ZERO)] {
            assert!(source
                .sample([0., 0.5, 0.5, 0.7], -1., 2., |_| Some((Vec3::ZERO, parent)))
                .unwrap()
                .is_finite());
        }
        let empty = BoneSource::default();
        assert!(empty
            .sample([0.; 4], 0., 0., |_| panic!(
                "empty candidates must not evaluate a pose"
            ))
            .is_none());
    }
    #[test]
    fn explicit_empty_palette_does_not_fall_back_to_model_nodes() {
        let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
        let owner = builder.add_named_node("owner").unwrap();
        let helper = builder.add_named_node("helper").unwrap();
        placed_palette(&mut builder, owner, vec![], 0);
        placed_palette(&mut builder, owner, vec![helper], 1);
        let model = builder.finish().unwrap();
        let indices = model
            .nodes()
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id, i))
            .collect();
        let source = BoneSource::compile(&model, &indices);
        assert!(source.is_empty());
        assert!(!source.inferred);
        assert_eq!(source.first_lod, Some(0));
    }
    #[test]
    fn missing_or_nonfinite_poses_reject_births() {
        let source = BoneSource {
            candidates: Arc::from([BoneCandidate {
                node: 0,
                pivot: Vec3::ZERO,
                parent: None,
            }]),
            ..Default::default()
        };
        assert!(source.sample([0.; 4], 0., 0., |_| None).is_none());
        assert!(source
            .sample([0.; 4], 0., 0., |_| Some((Vec3::NAN, None)))
            .is_none());
        assert!(source
            .sample([0.; 4], f32::INFINITY, 1., |_| Some((Vec3::ZERO, None)))
            .is_none());
    }
    #[test]
    fn excluded_unlisted_parent_remains_the_selected_childs_span_endpoint() {
        let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
        let mut parent = builder.new_node("excluded parent");
        let mut properties = BoneProperties::default();
        properties.flags.set_no_body_spawn(true);
        parent.properties = Some(NodeProperties::M2(properties));
        let parent = builder.add_node(parent).unwrap();
        let mut child = builder.new_node("selected child");
        child.parent = Some(parent);
        let child = builder.add_node(child).unwrap();
        placed_palette(&mut builder, child, vec![child], 0);
        let model = builder.finish().unwrap();
        let source = BoneSource::compile(&model, &HashMap::from([(parent, 0), (child, 1)]));
        assert_eq!(source.candidates.len(), 1);
        assert_eq!(source.candidates[0].parent, Some((0, Vec3::ZERO)));
    }
}
