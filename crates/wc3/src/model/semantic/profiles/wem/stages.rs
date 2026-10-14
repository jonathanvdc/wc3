//! Ordered pose operations own their weighted inputs and refer to actual simulation objects.
use super::physics::{ClothId, RigId};
use super::poses::Components;
use crate::model::semantic::{Error, Id, Link, NodeId, Scope, SystemId, ValidateLinks};
use std::borrow::Cow;
use std::collections::BTreeSet;
/// Identity marker for one operation in a model pose pipeline.
pub enum StageObject {}
/// Stable operation identity, independent of pipeline order.
pub type StageId = Id<StageObject>;
/// Identity marker for a weighted operation input.
pub enum StageSourceObject {}
/// Stable weighted input identity within its owning operation.
pub type StageSourceId = Id<StageSourceObject>;
/// Pose operation interpretation, including producer-defined operations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StageKind {
    #[default]
    /// Position.
    Position,
    /// Orientation.
    Orientation,
    /// LookAt.
    LookAt,
    /// Link.
    Link,
    /// LimbIk.
    LimbIk,
    /// ChainIk.
    ChainIk,
    /// Spring.
    Spring,
    /// Ragdoll.
    Ragdoll,
    /// Cloth.
    Cloth,
    /// Count.
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}
/// Weighted source whose absence of a node denotes the world frame.
#[derive(Clone, Debug)]
pub struct StageSource {
    id: StageSourceId,
    /// Hierarchy frame contributing to the operation, or the world frame.
    pub node: Option<NodeId>,
    /// Baseline contribution before animation overrides it.
    pub weight: f32,
}
impl StageSource {
    /// Creates an independently identified weighted input.
    pub fn new(node: Option<NodeId>, weight: f32) -> Self {
        Self {
            id: Scope::default().allocate(),
            node,
            weight,
        }
    }
    /// Identity preserved when inputs move within their operation.
    pub fn id(&self) -> StageSourceId {
        self.id
    }
}
/// Simulation group selected by a ragdoll operation.
#[derive(Clone, Copy, Debug)]
pub struct RagdollBinding {
    /// Model-owned simulation domain.
    pub system: SystemId,
    /// Activation group in that domain.
    pub rig: RigId,
}
/// Cloth selected by a pose operation.
#[derive(Clone, Copy, Debug)]
pub struct ClothBinding {
    /// Model-owned simulation domain.
    pub system: SystemId,
    /// Cloth object in that domain.
    pub cloth: ClothId,
}
/// One authored constraint or simulation operation in evaluation order.
#[derive(Clone, Debug)]
pub struct PoseStage<'a> {
    /// Display name.
    pub name: Cow<'a, str>,
    /// Operation interpretation.
    pub kind: StageKind,
    /// Baseline activation before animation overrides it.
    pub enabled: bool,
    /// Hierarchy nodes whose transforms this operation changes.
    pub driven: Vec<NodeId>,
    /// Hierarchy nodes supplying goals for this operation.
    pub targets: Vec<NodeId>,
    sources: Vec<StageSource>,
    /// Preserves the authored offset when solving the operation.
    pub keep_offset: bool,
    /// Local orientation or aiming axis.
    pub axis: [f32; 3],
    /// Up vector used by aiming constraints.
    pub up: [f32; 3],
    /// Optional hierarchy frame supplying the up direction.
    pub up_node: Option<NodeId>,
    /// Authored translation, rotation and scale offset.
    pub offset: Components,
    /// Spring stiffness.
    pub stiffness: f32,
    /// Spring damping.
    pub damping: f32,
    /// Gravity applied by this operation.
    pub gravity: f32,
    /// Simulation rig selected by this operation.
    pub ragdoll: Option<RagdollBinding>,
    /// Cloth selected by this operation.
    pub cloth: Option<ClothBinding>,
}
impl<'a> PoseStage<'a> {
    /// Creates an enabled operation with an identity offset and no inputs.
    pub fn new(kind: StageKind) -> Self {
        Self {
            name: Cow::Borrowed(""),
            kind,
            enabled: true,
            driven: Vec::new(),
            targets: Vec::new(),
            sources: Vec::new(),
            keep_offset: false,
            axis: [1., 0., 0.],
            up: [0., 0., 1.],
            up_node: None,
            offset: Components::default(),
            stiffness: 0.,
            damping: 0.,
            gravity: 0.,
            ragdoll: None,
            cloth: None,
        }
    }
    /// Adds a weighted input, rejecting repeated identities in this operation.
    pub fn add_source(&mut self, source: StageSource) -> Result<StageSourceId, Error> {
        if self.source(source.id()).is_some() {
            return Err(Error::new(
                "wem.pose.source",
                "input identity is already present",
            ));
        }
        let id = source.id();
        self.sources.push(source);
        Ok(id)
    }
    /// Borrows a weighted input by its actual identity.
    pub fn source(&self, id: StageSourceId) -> Option<&StageSource> {
        self.sources.iter().find(|value| value.id() == id)
    }
    /// Iterates weighted inputs in authored order.
    pub fn sources(&self) -> impl ExactSizeIterator<Item = &StageSource> {
        self.sources.iter()
    }
    /// Edits an existing input without changing its animation relationships.
    pub fn set_source(
        &mut self,
        id: StageSourceId,
        node: Option<NodeId>,
        weight: f32,
    ) -> Result<(), Error> {
        let source = self
            .sources
            .iter_mut()
            .find(|value| value.id() == id)
            .ok_or_else(|| Error::new("wem.pose.source", "input belongs to another operation"))?;
        source.node = node;
        source.weight = weight;
        Ok(())
    }
    /// Moves an input while preserving its identity and animation relationships.
    pub fn move_source(&mut self, id: StageSourceId, index: usize) -> Result<(), Error> {
        if index >= self.sources.len() {
            return Err(Error::new(
                "wem.pose.source",
                "destination exceeds operation inputs",
            ));
        }
        let old = self
            .sources
            .iter()
            .position(|value| value.id() == id)
            .ok_or_else(|| Error::new("wem.pose.source", "input belongs to another operation"))?;
        let source = self.sources.remove(old);
        self.sources.insert(index, source);
        Ok(())
    }
}
impl ValidateLinks for PoseStage<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.driven.visit_links(visitor);
        self.targets.visit_links(visitor);
        self.up_node.visit_links(visitor);
        for source in &self.sources {
            source.node.visit_links(visitor);
        }
        if let Some(value) = self.ragdoll {
            value.system.visit_links(visitor);
        }
        if let Some(value) = self.cloth {
            value.system.visit_links(visitor);
        }
        if self.offset.validate().is_err()
            || self
                .axis
                .iter()
                .chain(&self.up)
                .chain([&self.stiffness, &self.damping, &self.gravity])
                .chain(self.sources.iter().map(|value| &value.weight))
                .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
/// Ordered operations with identities that remain stable when evaluation order changes.
#[derive(Clone, Debug, Default)]
pub struct PosePipeline<'a> {
    scope: Scope,
    stages: Vec<(StageId, PoseStage<'a>)>,
}
impl<'a> PosePipeline<'a> {
    /// Appends a pose operation.
    pub fn add_stage(&mut self, value: PoseStage<'a>) -> StageId {
        let id = self.scope.allocate();
        self.stages.push((id, value));
        id
    }
    /// Borrows one pipeline-owned operation.
    pub fn stage(&self, id: StageId) -> Option<&PoseStage<'a>> {
        self.stages
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates operations in evaluation order.
    pub fn stages(&self) -> impl ExactSizeIterator<Item = (StageId, &PoseStage<'a>)> {
        self.stages.iter().map(|(id, value)| (*id, value))
    }
    /// Replaces an operation while retaining its pipeline identity.
    pub fn set_stage(&mut self, id: StageId, value: PoseStage<'a>) -> Result<(), Error> {
        let row = self
            .stages
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| Error::new("wem.pose.stage", "operation belongs to another pipeline"))?;
        row.1 = value;
        Ok(())
    }
    /// Changes evaluation order without changing operation or input identities.
    pub fn move_stage(&mut self, id: StageId, index: usize) -> Result<(), Error> {
        if index >= self.stages.len() {
            return Err(Error::new("wem.pose.stage", "destination exceeds pipeline"));
        }
        let old = self
            .stages
            .iter()
            .position(|(key, _)| *key == id)
            .ok_or_else(|| Error::new("wem.pose.stage", "operation belongs to another pipeline"))?;
        let stage = self.stages.remove(old);
        self.stages.insert(index, stage);
        Ok(())
    }
    pub(crate) fn validate_simulations(
        &self,
        rig: impl Fn(SystemId, RigId) -> bool,
        cloth: impl Fn(SystemId, ClothId) -> bool,
    ) -> Result<(), Error> {
        for (_, stage) in &self.stages {
            if stage
                .ragdoll
                .is_some_and(|value| !rig(value.system, value.rig))
                || stage
                    .cloth
                    .is_some_and(|value| !cloth(value.system, value.cloth))
            {
                return Err(Error::new(
                    "wem.pose.physics",
                    "selected simulation object belongs to another domain",
                ));
            }
        }
        Ok(())
    }
    /// Checks that weighted inputs each have one operation owner.
    pub fn validate(&self) -> Result<(), Error> {
        if self.stages.len() > 65536
            || self
                .stages
                .iter()
                .any(|(_, stage)| stage.sources.len() > 65535)
        {
            return Err(Error::new(
                "wem.pose.pipeline",
                "operation or input vocabulary exceeds packed animation selector capacity",
            ));
        }
        let mut seen = BTreeSet::new();
        for (_, stage) in &self.stages {
            for source in stage.sources() {
                if !seen.insert(source.id()) {
                    return Err(Error::new(
                        "wem.pose.source",
                        "input is owned by multiple operations",
                    ));
                }
            }
        }
        Ok(())
    }
}
impl ValidateLinks for PosePipeline<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (_, value) in &self.stages {
            value.visit_links(visitor);
        }
        if self.validate().is_err() {
            visitor(Link::InvalidValue);
        }
    }
}
