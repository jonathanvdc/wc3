use crate::animation::{GeosetAnimation, Sequence, TextureAnimation};
use crate::emitters::{ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter};
use crate::geometry::{BindPoseMatrix, CollisionShape};
use crate::materials::Texture;
use crate::model::dispatch_model;
use crate::scene::{Attachment, Bone, EventObject, FaceFx, ModelInfo, Node};
use crate::{AnyVersionModel, Model, ModelVersion, Vec3};

/// Accessors whose record types do not depend on the model version.
pub trait ModelAccess {
    fn sequences(&self) -> Vec<Sequence>;
    fn set_sequences(&mut self, sequences: &[Sequence]);
    fn global_sequences(&self) -> Vec<u32>;
    fn set_global_sequences(&mut self, durations: &[u32]);
    fn texture_animations(&self) -> Vec<TextureAnimation>;
    fn set_texture_animations(&mut self, animations: &[TextureAnimation]);
    fn geoset_animations(&self) -> Vec<GeosetAnimation>;
    fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]);
    fn textures(&self) -> Vec<Texture>;
    fn set_textures(&mut self, textures: &[Texture]);
    fn bones(&self) -> Vec<Bone>;
    fn set_bones(&mut self, bones: &[Bone]);
    fn helpers(&self) -> Vec<Node>;
    fn set_helpers(&mut self, helpers: &[Node]);
    fn attachments(&self) -> Vec<Attachment>;
    fn set_attachments(&mut self, attachments: &[Attachment]);
    fn event_objects(&self) -> Vec<EventObject>;
    fn set_event_objects(&mut self, events: &[EventObject]);
    fn collision_shapes(&self) -> Vec<CollisionShape>;
    fn set_collision_shapes(&mut self, shapes: &[CollisionShape]);
    fn pivot_points(&self) -> Vec<Vec3>;
    fn set_pivot_points(&mut self, points: &[Vec3]);
    fn particle_emitters(&self) -> Vec<ParticleEmitter>;
    fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]);
    fn particle_emitters2(&self) -> Vec<ParticleEmitter2>;
    fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]);
    fn ribbon_emitters(&self) -> Vec<RibbonEmitter>;
    fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]);
    fn popcorn_emitters(&self) -> Vec<PopcornEmitter>;
    fn set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]);
    fn face_fx(&self) -> Vec<FaceFx>;
    fn set_face_fx(&mut self, entries: &[FaceFx]);
    fn bind_poses(&self) -> Vec<BindPoseMatrix>;
    fn set_bind_poses(&mut self, poses: &[BindPoseMatrix]);
    fn model_info(&self) -> Option<ModelInfo>;
    fn set_model_info(&mut self, info: &ModelInfo);
}

impl<V: ModelVersion> ModelAccess for Model<V> {
    fn sequences(&self) -> Vec<Sequence> {
        Model::sequences(self)
    }
    fn set_sequences(&mut self, sequences: &[Sequence]) {
        Model::set_sequences(self, sequences)
    }
    fn global_sequences(&self) -> Vec<u32> {
        Model::global_sequences(self)
    }
    fn set_global_sequences(&mut self, durations: &[u32]) {
        Model::set_global_sequences(self, durations)
    }
    fn texture_animations(&self) -> Vec<TextureAnimation> {
        Model::texture_animations(self)
    }
    fn set_texture_animations(&mut self, animations: &[TextureAnimation]) {
        Model::set_texture_animations(self, animations)
    }
    fn geoset_animations(&self) -> Vec<GeosetAnimation> {
        Model::geoset_animations(self)
    }
    fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) {
        Model::set_geoset_animations(self, animations)
    }
    fn textures(&self) -> Vec<Texture> {
        Model::textures(self)
    }
    fn set_textures(&mut self, textures: &[Texture]) {
        Model::set_textures(self, textures)
    }
    fn bones(&self) -> Vec<Bone> {
        Model::bones(self)
    }
    fn set_bones(&mut self, bones: &[Bone]) {
        Model::set_bones(self, bones)
    }
    fn helpers(&self) -> Vec<Node> {
        Model::helpers(self)
    }
    fn set_helpers(&mut self, helpers: &[Node]) {
        Model::set_helpers(self, helpers)
    }
    fn attachments(&self) -> Vec<Attachment> {
        Model::attachments(self)
    }
    fn set_attachments(&mut self, attachments: &[Attachment]) {
        Model::set_attachments(self, attachments)
    }
    fn event_objects(&self) -> Vec<EventObject> {
        Model::event_objects(self)
    }
    fn set_event_objects(&mut self, events: &[EventObject]) {
        Model::set_event_objects(self, events)
    }
    fn collision_shapes(&self) -> Vec<CollisionShape> {
        Model::collision_shapes(self)
    }
    fn set_collision_shapes(&mut self, shapes: &[CollisionShape]) {
        Model::set_collision_shapes(self, shapes)
    }
    fn pivot_points(&self) -> Vec<Vec3> {
        Model::pivot_points(self)
    }
    fn set_pivot_points(&mut self, points: &[Vec3]) {
        Model::set_pivot_points(self, points)
    }
    fn particle_emitters(&self) -> Vec<ParticleEmitter> {
        Model::particle_emitters(self)
    }
    fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]) {
        Model::set_particle_emitters(self, emitters)
    }
    fn particle_emitters2(&self) -> Vec<ParticleEmitter2> {
        Model::particle_emitters2(self)
    }
    fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]) {
        Model::set_particle_emitters2(self, emitters)
    }
    fn ribbon_emitters(&self) -> Vec<RibbonEmitter> {
        Model::ribbon_emitters(self)
    }
    fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]) {
        Model::set_ribbon_emitters(self, emitters)
    }
    fn popcorn_emitters(&self) -> Vec<PopcornEmitter> {
        Model::popcorn_emitters(self)
    }
    fn set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) {
        Model::set_popcorn_emitters(self, emitters)
    }
    fn face_fx(&self) -> Vec<FaceFx> {
        Model::face_fx(self)
    }
    fn set_face_fx(&mut self, entries: &[FaceFx]) {
        Model::set_face_fx(self, entries)
    }
    fn bind_poses(&self) -> Vec<BindPoseMatrix> {
        Model::bind_poses(self)
    }
    fn set_bind_poses(&mut self, poses: &[BindPoseMatrix]) {
        Model::set_bind_poses(self, poses)
    }
    fn model_info(&self) -> Option<ModelInfo> {
        Model::model_info(self)
    }
    fn set_model_info(&mut self, info: &ModelInfo) {
        Model::set_model_info(self, info)
    }
}

impl ModelAccess for AnyVersionModel {
    fn sequences(&self) -> Vec<Sequence> {
        dispatch_model!(self, model => model.sequences())
    }
    fn set_sequences(&mut self, sequences: &[Sequence]) {
        dispatch_model!(self, model => model.set_sequences(sequences))
    }
    fn global_sequences(&self) -> Vec<u32> {
        dispatch_model!(self, model => model.global_sequences())
    }
    fn set_global_sequences(&mut self, durations: &[u32]) {
        dispatch_model!(self, model => model.set_global_sequences(durations))
    }
    fn texture_animations(&self) -> Vec<TextureAnimation> {
        dispatch_model!(self, model => model.texture_animations())
    }
    fn set_texture_animations(&mut self, animations: &[TextureAnimation]) {
        dispatch_model!(self, model => model.set_texture_animations(animations))
    }
    fn geoset_animations(&self) -> Vec<GeosetAnimation> {
        dispatch_model!(self, model => model.geoset_animations())
    }
    fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) {
        dispatch_model!(self, model => model.set_geoset_animations(animations))
    }
    fn textures(&self) -> Vec<Texture> {
        dispatch_model!(self, model => model.textures())
    }
    fn set_textures(&mut self, textures: &[Texture]) {
        dispatch_model!(self, model => model.set_textures(textures))
    }
    fn bones(&self) -> Vec<Bone> {
        dispatch_model!(self, model => model.bones())
    }
    fn set_bones(&mut self, bones: &[Bone]) {
        dispatch_model!(self, model => model.set_bones(bones))
    }
    fn helpers(&self) -> Vec<Node> {
        dispatch_model!(self, model => model.helpers())
    }
    fn set_helpers(&mut self, helpers: &[Node]) {
        dispatch_model!(self, model => model.set_helpers(helpers))
    }
    fn attachments(&self) -> Vec<Attachment> {
        dispatch_model!(self, model => model.attachments())
    }
    fn set_attachments(&mut self, attachments: &[Attachment]) {
        dispatch_model!(self, model => model.set_attachments(attachments))
    }
    fn event_objects(&self) -> Vec<EventObject> {
        dispatch_model!(self, model => model.event_objects())
    }
    fn set_event_objects(&mut self, events: &[EventObject]) {
        dispatch_model!(self, model => model.set_event_objects(events))
    }
    fn collision_shapes(&self) -> Vec<CollisionShape> {
        dispatch_model!(self, model => model.collision_shapes())
    }
    fn set_collision_shapes(&mut self, shapes: &[CollisionShape]) {
        dispatch_model!(self, model => model.set_collision_shapes(shapes))
    }
    fn pivot_points(&self) -> Vec<Vec3> {
        dispatch_model!(self, model => model.pivot_points())
    }
    fn set_pivot_points(&mut self, points: &[Vec3]) {
        dispatch_model!(self, model => model.set_pivot_points(points))
    }
    fn particle_emitters(&self) -> Vec<ParticleEmitter> {
        dispatch_model!(self, model => model.particle_emitters())
    }
    fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]) {
        dispatch_model!(self, model => model.set_particle_emitters(emitters))
    }
    fn particle_emitters2(&self) -> Vec<ParticleEmitter2> {
        dispatch_model!(self, model => model.particle_emitters2())
    }
    fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]) {
        dispatch_model!(self, model => model.set_particle_emitters2(emitters))
    }
    fn ribbon_emitters(&self) -> Vec<RibbonEmitter> {
        dispatch_model!(self, model => model.ribbon_emitters())
    }
    fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]) {
        dispatch_model!(self, model => model.set_ribbon_emitters(emitters))
    }
    fn popcorn_emitters(&self) -> Vec<PopcornEmitter> {
        dispatch_model!(self, model => model.popcorn_emitters())
    }
    fn set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) {
        dispatch_model!(self, model => model.set_popcorn_emitters(emitters))
    }
    fn face_fx(&self) -> Vec<FaceFx> {
        dispatch_model!(self, model => model.face_fx())
    }
    fn set_face_fx(&mut self, entries: &[FaceFx]) {
        dispatch_model!(self, model => model.set_face_fx(entries))
    }
    fn bind_poses(&self) -> Vec<BindPoseMatrix> {
        dispatch_model!(self, model => model.bind_poses())
    }
    fn set_bind_poses(&mut self, poses: &[BindPoseMatrix]) {
        dispatch_model!(self, model => model.set_bind_poses(poses))
    }
    fn model_info(&self) -> Option<ModelInfo> {
        dispatch_model!(self, model => model.model_info())
    }
    fn set_model_info(&mut self, info: &ModelInfo) {
        dispatch_model!(self, model => model.set_model_info(info))
    }
}

impl AnyVersionModel {
    /// Returns sequences for this model.
    #[inline]
    pub fn sequences(&self) -> Vec<Sequence> {
        ModelAccess::sequences(self)
    }
    /// Replaces sequences for this model.
    #[inline]
    pub fn set_sequences(&mut self, sequences: &[Sequence]) {
        ModelAccess::set_sequences(self, sequences)
    }
    /// Returns global sequences for this model.
    #[inline]
    pub fn global_sequences(&self) -> Vec<u32> {
        ModelAccess::global_sequences(self)
    }
    /// Replaces global sequences for this model.
    #[inline]
    pub fn set_global_sequences(&mut self, durations: &[u32]) {
        ModelAccess::set_global_sequences(self, durations)
    }
    /// Returns texture animations for this model.
    #[inline]
    pub fn texture_animations(&self) -> Vec<TextureAnimation> {
        ModelAccess::texture_animations(self)
    }
    /// Replaces texture animations for this model.
    #[inline]
    pub fn set_texture_animations(&mut self, animations: &[TextureAnimation]) {
        ModelAccess::set_texture_animations(self, animations)
    }
    /// Returns geoset animations for this model.
    #[inline]
    pub fn geoset_animations(&self) -> Vec<GeosetAnimation> {
        ModelAccess::geoset_animations(self)
    }
    /// Replaces geoset animations for this model.
    #[inline]
    pub fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) {
        ModelAccess::set_geoset_animations(self, animations)
    }
    /// Returns textures for this model.
    #[inline]
    pub fn textures(&self) -> Vec<Texture> {
        ModelAccess::textures(self)
    }
    /// Replaces textures for this model.
    #[inline]
    pub fn set_textures(&mut self, textures: &[Texture]) {
        ModelAccess::set_textures(self, textures)
    }
    /// Returns bones for this model.
    #[inline]
    pub fn bones(&self) -> Vec<Bone> {
        ModelAccess::bones(self)
    }
    /// Replaces bones for this model.
    #[inline]
    pub fn set_bones(&mut self, bones: &[Bone]) {
        ModelAccess::set_bones(self, bones)
    }
    /// Returns helpers for this model.
    #[inline]
    pub fn helpers(&self) -> Vec<Node> {
        ModelAccess::helpers(self)
    }
    /// Replaces helpers for this model.
    #[inline]
    pub fn set_helpers(&mut self, helpers: &[Node]) {
        ModelAccess::set_helpers(self, helpers)
    }
    /// Returns attachments for this model.
    #[inline]
    pub fn attachments(&self) -> Vec<Attachment> {
        ModelAccess::attachments(self)
    }
    /// Replaces attachments for this model.
    #[inline]
    pub fn set_attachments(&mut self, attachments: &[Attachment]) {
        ModelAccess::set_attachments(self, attachments)
    }
    /// Returns event objects for this model.
    #[inline]
    pub fn event_objects(&self) -> Vec<EventObject> {
        ModelAccess::event_objects(self)
    }
    /// Replaces event objects for this model.
    #[inline]
    pub fn set_event_objects(&mut self, events: &[EventObject]) {
        ModelAccess::set_event_objects(self, events)
    }
    /// Returns collision shapes for this model.
    #[inline]
    pub fn collision_shapes(&self) -> Vec<CollisionShape> {
        ModelAccess::collision_shapes(self)
    }
    /// Replaces collision shapes for this model.
    #[inline]
    pub fn set_collision_shapes(&mut self, shapes: &[CollisionShape]) {
        ModelAccess::set_collision_shapes(self, shapes)
    }
    /// Returns pivot points for this model.
    #[inline]
    pub fn pivot_points(&self) -> Vec<Vec3> {
        ModelAccess::pivot_points(self)
    }
    /// Replaces pivot points for this model.
    #[inline]
    pub fn set_pivot_points(&mut self, points: &[Vec3]) {
        ModelAccess::set_pivot_points(self, points)
    }
    /// Returns particle emitters for this model.
    #[inline]
    pub fn particle_emitters(&self) -> Vec<ParticleEmitter> {
        ModelAccess::particle_emitters(self)
    }
    /// Replaces particle emitters for this model.
    #[inline]
    pub fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]) {
        ModelAccess::set_particle_emitters(self, emitters)
    }
    /// Returns particle emitters2 for this model.
    #[inline]
    pub fn particle_emitters2(&self) -> Vec<ParticleEmitter2> {
        ModelAccess::particle_emitters2(self)
    }
    /// Replaces particle emitters2 for this model.
    #[inline]
    pub fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]) {
        ModelAccess::set_particle_emitters2(self, emitters)
    }
    /// Returns ribbon emitters for this model.
    #[inline]
    pub fn ribbon_emitters(&self) -> Vec<RibbonEmitter> {
        ModelAccess::ribbon_emitters(self)
    }
    /// Replaces ribbon emitters for this model.
    #[inline]
    pub fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]) {
        ModelAccess::set_ribbon_emitters(self, emitters)
    }
    /// Returns popcorn emitters for this model.
    #[inline]
    pub fn popcorn_emitters(&self) -> Vec<PopcornEmitter> {
        ModelAccess::popcorn_emitters(self)
    }
    /// Replaces popcorn emitters for this model.
    #[inline]
    pub fn set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) {
        ModelAccess::set_popcorn_emitters(self, emitters)
    }
    /// Returns face fx for this model.
    #[inline]
    pub fn face_fx(&self) -> Vec<FaceFx> {
        ModelAccess::face_fx(self)
    }
    /// Replaces face fx for this model.
    #[inline]
    pub fn set_face_fx(&mut self, entries: &[FaceFx]) {
        ModelAccess::set_face_fx(self, entries)
    }
    /// Returns bind poses for this model.
    #[inline]
    pub fn bind_poses(&self) -> Vec<BindPoseMatrix> {
        ModelAccess::bind_poses(self)
    }
    /// Replaces bind poses for this model.
    #[inline]
    pub fn set_bind_poses(&mut self, poses: &[BindPoseMatrix]) {
        ModelAccess::set_bind_poses(self, poses)
    }
    /// Returns the first model information record.
    #[inline]
    pub fn model_info(&self) -> Option<ModelInfo> {
        ModelAccess::model_info(self)
    }
    /// Replaces the model information record.
    #[inline]
    pub fn set_model_info(&mut self, info: &ModelInfo) {
        ModelAccess::set_model_info(self, info)
    }
}
