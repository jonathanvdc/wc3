//! Skeleton and fracture preparation belongs to a simulation rig, independent of file revisions.
mod values;
use super::{BodyId, Physics, ShapeKind};
use crate::model::semantic::{
    Error, Id, Link, MaterialSlotId, MeshId, NodeId, ValidateLinks, Values,
};
use std::borrow::Cow;
pub use values::*;
/// Identity marker for a simulation activation and preparation group.
pub enum RigObject {}
/// Simulation rig belonging to one physics domain.
pub type RigId = Id<RigObject>;
/// Authored WowVegetation settings, independent of encoded layout.
#[derive(Clone, Debug, Default)]
pub struct WowVegetation {
    /// Maximum displacement contribution from position pushing.
    pub pos_max_push: f32,
    /// Strength of the position-push response.
    pub pos_push_amt: f32,
    /// Rate at which displaced vegetation returns toward its rest position.
    pub pos_relax_speed: f32,
    /// Maximum velocity-push contribution.
    pub vel_max_push: f32,
    /// Rate control for the velocity-driven response.
    pub vel_speed: f32,
    /// Squared-distance threshold for the push interaction.
    pub min_push_dist: f32,
}
/// Authored RagdollRecipe settings, independent of encoded layout.
#[derive(Clone, Debug, Default)]
pub struct RagdollRecipe {
    /// Shape-generation mode for the torso.
    pub torso: ShapeKind,
    /// Shape-generation mode for the head.
    pub head: ShapeKind,
    /// Shape-generation mode for limb bodies.
    pub limbs: ShapeKind,
    /// Shape-generation mode for attached props.
    pub props: ShapeKind,
    /// Point-count control for generated convex hulls.
    pub hull_points: u8,
    /// Fit control determining how closely generated bodies follow the geometry.
    pub tightness: f32,
    /// Thickness control for generated collision shapes.
    pub thickness: f32,
    /// Joint-range selection mode.
    pub range: JointRange,
    /// Prefers hinge constraints where the preparation supports them.
    pub hinges: bool,
    /// Permits collision between connected generated bodies.
    pub collide_connected: bool,
    /// Combines short skeleton links during ragdoll preparation.
    pub fold_short_links: bool,
    /// Welds attached props to their carrying body.
    pub weld_props: bool,
    /// Physics-material preset selected for generated bodies.
    pub material: u32,
    /// Translational damping applied to generated bodies.
    pub linear_damping: f32,
    /// Rotational damping applied to generated bodies.
    pub angular_damping: f32,
}
/// Authored FractureRecipe settings, independent of encoded layout.
#[derive(Clone, Debug, Default)]
pub struct FractureRecipe<'a> {
    /// Keeps copies of the complete source meshes alongside generated pieces.
    pub keep_whole: bool,
    /// Original geometry for each fracture target; absence leaves that target without a source.
    pub sources: Vec<Option<MeshId>>,
    /// Generated exterior and optional interior geometry in target order.
    pub made: Vec<Option<MeshId>>,
    /// Requested piece count per target; one retains the target whole.
    pub pieces: Values<'a, u16>,
    /// Skeleton nodes used by fracture skin bindings; absence leaves a slot unbound.
    pub skin_nodes: Vec<Option<NodeId>>,
    /// Seed for repeatable fracture choices.
    pub seed: u32,
    /// Control for making pieces smaller near the blast center.
    pub near_blast: f32,
    /// Control for how uniform generated piece sizes should be.
    pub even: f32,
    /// Preferred grain axis: none, X, Y, or Z.
    pub grain: GrainAxis,
    /// Elongation control along the grain direction.
    pub stretch: f32,
    /// Minimum piece-size fraction relative to the average piece.
    pub smallest: f32,
    /// Treatment of open geometry during fracture.
    pub open_parts: OpenParts,
    /// Shell thickness in model units; zero requests a size-derived value.
    pub thickness: f32,
    /// Material slot for fracture interiors; absence reuses the exterior material.
    pub inside: Option<MaterialSlotId>,
    /// Scale applied to generated interior texture coordinates.
    pub uv_scale: f32,
    /// Point-count control for piece collision hulls.
    pub hull_points: u8,
    /// Allows hollow geometry to split into separate pieces.
    pub split_hollow: bool,
    /// Physics-material preset assigned to generated pieces.
    pub material: u32,
    /// Optional hierarchy node inserted between pieces and their parent.
    pub helper: Option<NodeId>,
    /// Optional hierarchy node defining the blast force field.
    pub field: Option<NodeId>,
    /// Local force-channel bit shared by the field and generated pieces.
    pub channel: u8,
    /// Model-space seed positions retained for each generated piece.
    pub seeds: Values<'a, [f32; 3]>,
    /// Seed positions associated with pieces that remain static.
    pub statics: Values<'a, [f32; 3]>,
    /// Generation method selecting general fracture or planar slices.
    pub method: FractureMethod,
    /// Shared requested piece total; zero leaves each target with its own count.
    pub total: u16,
    /// Per-target byte flags selecting independent piece counts.
    pub pinned: Values<'a, u8>,
    /// Axis mode for each slice set: X, Y, Z, or an explicitly supplied direction.
    pub slice_along: Values<'a, SliceAlong>,
    /// Direction vectors for custom-axis slice sets.
    pub slice_directions: Values<'a, [f32; 3]>,
    /// Plane counts for the slice sets.
    pub slice_counts: Values<'a, u16>,
    /// Plane jitter as a fraction of half the slice spacing.
    pub slice_jitters: Values<'a, f32>,
    /// Plane tilts in radians.
    pub slice_tilts: Values<'a, f32>,
    /// Plane shifts in model units along their slicing directions.
    pub slice_shifts: Values<'a, f32>,
    /// Normals of the actual cutting planes.
    pub plane_normals: Values<'a, [f32; 3]>,
    /// Plane equation distances satisfying normal dot position = offset.
    pub plane_offsets: Values<'a, f32>,
    /// Piece motion mode selecting blast, implosion, or no push.
    pub push: PushMode,
    /// Release ordering selecting simultaneous, bottom-up, top-down, or outward activation.
    pub release: ReleaseOrder,
    /// Duration of staggered piece activation, in seconds.
    pub over: f32,
    /// Reduction in piece collision-hull size expressed as a fraction of piece size.
    pub loosen: f32,
    /// Model-space center of the fracture activation.
    pub centre: [f32; 3],
    /// Activation radius in model units.
    pub radius: f32,
    /// Push acceleration expressed as a multiple of gravitational acceleration.
    pub strength: f32,
    /// Duration of the push, in seconds.
    pub length: f32,
    /// Activation time measured in seconds from the clip start.
    pub at: f32,
    /// Texture-coordinate layout for generated interior geometry.
    pub filling_uvs: InteriorUvs,
    /// Resolution selected for the generated interior texture map.
    pub map_size: u32,
    /// Records whether the recipe created its interior material.
    pub filling_made: bool,
    /// Geometry shown before activation: automatic choice, whole mesh, or pieces.
    pub before: BeforeActivation,
    /// Optional hierarchy node whose visibility enables drawing of whole meshes.
    pub whole_gate: Option<NodeId>,
    /// Optional hierarchy node whose visibility enables drawing of generated pieces.
    pub pieces_gate: Option<NodeId>,
    /// Centers of bounded cutting-plane rectangles.
    pub plane_centres: Values<'a, [f32; 3]>,
    /// In-plane direction defining the rectangle width axis.
    pub plane_alongs: Values<'a, [f32; 3]>,
    /// Half-widths of cutting rectangles; zero selects an unbounded cut.
    pub plane_widths: Values<'a, f32>,
    /// Half-heights of cutting rectangles; zero selects an unbounded cut.
    pub plane_heights: Values<'a, f32>,
    /// Copies the affected skeleton onto generated pieces.
    pub cut_skeleton: bool,
    /// Additional skeleton nodes created by fracture preparation.
    pub bones: Vec<NodeId>,
    /// Copies the affected ragdoll bodies and surviving joints onto generated pieces.
    pub cut_ragdoll: bool,
}
impl ValidateLinks for FractureRecipe<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.sources.visit_links(visitor);
        self.made.visit_links(visitor);
        self.skin_nodes.visit_links(visitor);
        self.bones.visit_links(visitor);
        self.inside.visit_links(visitor);
        self.helper.visit_links(visitor);
        self.field.visit_links(visitor);
        self.whole_gate.visit_links(visitor);
        self.pieces_gate.visit_links(visitor);
        if self
            .seeds
            .iter()
            .chain(self.statics.iter())
            .chain(self.slice_directions.iter())
            .chain(self.plane_normals.iter())
            .chain(self.plane_centres.iter())
            .chain(self.plane_alongs.iter())
            .flatten()
            .chain(self.slice_jitters.iter())
            .chain(self.slice_tilts.iter())
            .chain(self.slice_shifts.iter())
            .chain(self.plane_offsets.iter())
            .chain(self.plane_widths.iter())
            .chain(self.plane_heights.iter())
            .chain(self.centre)
            .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        if [
            self.near_blast,
            self.even,
            self.stretch,
            self.smallest,
            self.thickness,
            self.uv_scale,
            self.over,
            self.loosen,
            self.radius,
            self.strength,
            self.length,
            self.at,
        ]
        .iter()
        .any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
/// World of Warcraft response and host skeleton selection.
#[derive(Clone, Debug)]
pub struct WowRig<'a> {
    /// Response category.
    pub kind: WowPhysicsKind,
    /// Optional vegetation response.
    pub vegetation: Option<WowVegetation>,
    /// Host skeleton name CRCs permitted to retain following coefficients.
    pub allowed_skeleton_names: Values<'a, u32>,
}
/// One actual simulation group and its authored preparation recipes.
#[derive(Clone, Debug)]
pub struct Rig<'a> {
    /// Display name.
    pub name: Cow<'a, str>,
    /// Activation convention.
    pub start: RigStart,
    /// Bodies affected by this activation program.
    pub bodies: Vec<BodyId>,
    /// World of Warcraft response behavior.
    pub wow: Option<WowRig<'a>>,
    /// Skeleton-derived body preparation.
    pub ragdoll: Option<RagdollRecipe>,
    /// Fracture generation and activation.
    pub fracture: Option<FractureRecipe<'a>>,
}
impl ValidateLinks for Rig<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.fracture.visit_links(visitor);
        if self.ragdoll.as_ref().is_some_and(|value| {
            [
                value.tightness,
                value.thickness,
                value.linear_damping,
                value.angular_damping,
            ]
            .iter()
            .any(|value| !value.is_finite())
        }) || self
            .wow
            .as_ref()
            .and_then(|wow| wow.vegetation.as_ref())
            .is_some_and(|value| {
                [
                    value.pos_max_push,
                    value.pos_push_amt,
                    value.pos_relax_speed,
                    value.vel_max_push,
                    value.vel_speed,
                    value.min_push_dist,
                ]
                .iter()
                .any(|value| !value.is_finite())
            })
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl<'a> Physics<'a> {
    /// Adds an activation group while retaining stable body relationships.
    pub fn add_rig(&mut self, rig: Rig<'a>) -> RigId {
        let id = self.scope.allocate();
        self.rigs.push((id, rig));
        id
    }
    /// Borrows one domain-owned activation group.
    pub fn rig(&self, id: RigId) -> Option<&Rig<'a>> {
        self.rigs
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates authored activation groups.
    pub fn rigs(&self) -> impl ExactSizeIterator<Item = (RigId, &Rig<'a>)> {
        self.rigs.iter().map(|(id, value)| (*id, value))
    }
    /// Updates a rig while preserving playback relationships to its identity.
    pub fn set_rig(&mut self, id: RigId, value: Rig<'a>) -> Result<(), Error> {
        let row = self
            .rigs
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new("wem.physics.rig", "rig belongs to another physics domain")
            })?;
        row.1 = value;
        Ok(())
    }
}
