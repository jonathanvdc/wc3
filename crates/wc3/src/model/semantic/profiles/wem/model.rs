//! WEM hierarchy-wide pose conventions and default animation selection.
use super::poses::{Poses, TestPose};
use super::{AnimationSetId, LodPreparation};
use crate::model::semantic::{Error, Id, Link, Scope, ValidateLinks};

/// Identity marker for an authored skinning test pose.
pub enum TestPoseObject {}
/// Stable test-pose identity owned by one model's authoring setup.
pub type TestPoseId = Id<TestPoseObject>;

/// Hierarchy-wide declarations independent of physical model record versions.
#[derive(Clone, Debug, Default)]
pub struct ModelControls<'a> {
    /// Ordered hierarchy constraints and simulation pose operations.
    pub pose_pipeline: super::PosePipeline<'a>,
    /// Model-owned animation declarations, including unused declarations and initialization aliases.
    pub channels: super::animation::Channels,
    /// Independently named selections of the model-owned channel declarations.
    pub channel_selections: super::animation::Selections<'a>,
    /// Declared hierarchical transform convention, when the authoring vocabulary includes it.
    pub hierarchy_convention: Option<HierarchyConvention>,
    /// Simplification request; absence means the authoring document declares no request.
    pub lod_preparation: Option<LodPreparation>,
    /// Default animation-tag selection program in the owning collection.
    pub animation_set: Option<AnimationSetId>,
    /// Saved pose conventions shared by the actual node-local saved values.
    pub poses: Poses<'a>,
    test_scope: Scope,
    tests: Vec<(TestPoseId, TestPose<'a>)>,
    selected_test: Option<TestPoseId>,
}
impl<'a> ModelControls<'a> {
    /// Adds an independently named skinning test frame.
    pub fn add_test_pose(&mut self, pose: TestPose<'a>) -> TestPoseId {
        let id = self.test_scope.allocate();
        self.tests.push((id, pose));
        id
    }
    /// Borrows a test frame owned by this collectionup.
    pub fn test_pose(&self, id: TestPoseId) -> Option<&TestPose<'a>> {
        self.tests
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates test frames in authored order.
    pub fn test_poses(&self) -> impl ExactSizeIterator<Item = (TestPoseId, &TestPose<'a>)> {
        self.tests.iter().map(|(id, value)| (*id, value))
    }
    /// Selects the test frame used for repose operations after checking ownership.
    pub fn set_selected_test(&mut self, id: Option<TestPoseId>) -> Result<(), Error> {
        if id.is_some_and(|id| self.test_pose(id).is_none()) {
            return Err(Error::new(
                "wem.model.test_pose",
                "test pose belongs to another model",
            ));
        }
        self.selected_test = id;
        Ok(())
    }
    /// Selected repose test frame.
    pub fn selected_test(&self) -> Option<TestPoseId> {
        self.selected_test
    }
}
impl ValidateLinks for ModelControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.pose_pipeline.visit_links(visitor);
        self.channels.visit_links(visitor);
        self.lod_preparation.visit_links(visitor);
        for (_, test) in &self.tests {
            test.visit_links(visitor);
        }
    }
}

/// Convention for interpreting authored hierarchy and bind transforms.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HierarchyConvention {
    /// Transforms are relative to native pivots.
    #[default]
    PivotRelative,
    /// Explicit bind transforms define the hierarchy frame.
    ExplicitBind,
    /// Producer-defined hierarchy convention.
    Unknown(u8),
}
