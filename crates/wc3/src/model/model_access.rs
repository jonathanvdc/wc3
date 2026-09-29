use crate::model::animation::{GeosetAnimation, Sequence, TextureAnimation};
use crate::model::emitters::{ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter};
use crate::model::geometry::{BindPoseMatrix, CollisionShape};
use crate::model::materials::Texture;
use crate::model::scene::{Attachment, Bone, EventObject, FaceFx, Glider, ModelInfo, Node};
use crate::model::visit_model;
use crate::model::{DynamicModel, Model, ModelVersion, ValueError, Vec3};

/// Read and replace collections shared by every model version.
///
/// Import this trait to use these operations on [`DynamicModel`]. Getters return
/// owned copies; edit them and pass them to a setter to update the model.
/// Collection setters replace all chunks of that kind with one collection.
pub trait CommonModelAccess {
    fn gliders(&self) -> Vec<Glider>;
    fn set_gliders(&mut self, gliders: &[Glider]);
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
    fn model_info(&self) -> Option<ModelInfo>;
    fn set_model_info(&mut self, info: &ModelInfo);
}

impl<V: ModelVersion> CommonModelAccess for Model<V> {
    fn gliders(&self) -> Vec<Glider> {
        Model::gliders(self)
    }
    fn set_gliders(&mut self, gliders: &[Glider]) {
        Model::set_gliders(self, gliders)
    }

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
    fn model_info(&self) -> Option<ModelInfo> {
        Model::model_info(self)
    }
    fn set_model_info(&mut self, info: &ModelInfo) {
        Model::set_model_info(self, info)
    }
}

impl CommonModelAccess for DynamicModel {
    fn gliders(&self) -> Vec<Glider> {
        visit_model!(self, |model| model.gliders())
    }
    fn set_gliders(&mut self, gliders: &[Glider]) {
        visit_model!(self, |model| model.set_gliders(gliders))
    }

    fn sequences(&self) -> Vec<Sequence> {
        visit_model!(self, |model| model.sequences())
    }
    fn set_sequences(&mut self, sequences: &[Sequence]) {
        visit_model!(self, |model| model.set_sequences(sequences))
    }
    fn global_sequences(&self) -> Vec<u32> {
        visit_model!(self, |model| model.global_sequences())
    }
    fn set_global_sequences(&mut self, durations: &[u32]) {
        visit_model!(self, |model| model.set_global_sequences(durations))
    }
    fn texture_animations(&self) -> Vec<TextureAnimation> {
        visit_model!(self, |model| model.texture_animations())
    }
    fn set_texture_animations(&mut self, animations: &[TextureAnimation]) {
        visit_model!(self, |model| model.set_texture_animations(animations))
    }
    fn geoset_animations(&self) -> Vec<GeosetAnimation> {
        visit_model!(self, |model| model.geoset_animations())
    }
    fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) {
        visit_model!(self, |model| model.set_geoset_animations(animations))
    }
    fn textures(&self) -> Vec<Texture> {
        visit_model!(self, |model| model.textures())
    }
    fn set_textures(&mut self, textures: &[Texture]) {
        visit_model!(self, |model| model.set_textures(textures))
    }
    fn bones(&self) -> Vec<Bone> {
        visit_model!(self, |model| model.bones())
    }
    fn set_bones(&mut self, bones: &[Bone]) {
        visit_model!(self, |model| model.set_bones(bones))
    }
    fn helpers(&self) -> Vec<Node> {
        visit_model!(self, |model| model.helpers())
    }
    fn set_helpers(&mut self, helpers: &[Node]) {
        visit_model!(self, |model| model.set_helpers(helpers))
    }
    fn attachments(&self) -> Vec<Attachment> {
        visit_model!(self, |model| model.attachments())
    }
    fn set_attachments(&mut self, attachments: &[Attachment]) {
        visit_model!(self, |model| model.set_attachments(attachments))
    }
    fn event_objects(&self) -> Vec<EventObject> {
        visit_model!(self, |model| model.event_objects())
    }
    fn set_event_objects(&mut self, events: &[EventObject]) {
        visit_model!(self, |model| model.set_event_objects(events))
    }
    fn collision_shapes(&self) -> Vec<CollisionShape> {
        visit_model!(self, |model| model.collision_shapes())
    }
    fn set_collision_shapes(&mut self, shapes: &[CollisionShape]) {
        visit_model!(self, |model| model.set_collision_shapes(shapes))
    }
    fn pivot_points(&self) -> Vec<Vec3> {
        visit_model!(self, |model| model.pivot_points())
    }
    fn set_pivot_points(&mut self, points: &[Vec3]) {
        visit_model!(self, |model| model.set_pivot_points(points))
    }
    fn particle_emitters(&self) -> Vec<ParticleEmitter> {
        visit_model!(self, |model| model.particle_emitters())
    }
    fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]) {
        visit_model!(self, |model| model.set_particle_emitters(emitters))
    }
    fn particle_emitters2(&self) -> Vec<ParticleEmitter2> {
        visit_model!(self, |model| model.particle_emitters2())
    }
    fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]) {
        visit_model!(self, |model| model.set_particle_emitters2(emitters))
    }
    fn ribbon_emitters(&self) -> Vec<RibbonEmitter> {
        visit_model!(self, |model| model.ribbon_emitters())
    }
    fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]) {
        visit_model!(self, |model| model.set_ribbon_emitters(emitters))
    }
    fn model_info(&self) -> Option<ModelInfo> {
        visit_model!(self, |model| model.model_info())
    }
    fn set_model_info(&mut self, info: &ModelInfo) {
        visit_model!(self, |model| model.set_model_info(info))
    }
}

/// Access Reforged features on models whose version is known at runtime.
///
/// Methods return `ValueError` for versions before 900. Getters return owned
/// copies; setters replace the corresponding model collection.
pub trait TryModelAccess {
    fn try_bind_poses(&self) -> Result<Vec<BindPoseMatrix>, ValueError>;
    fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError>;
    fn try_face_fx(&self) -> Result<Vec<FaceFx>, ValueError>;
    fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError>;
    fn try_popcorn_emitters(&self) -> Result<Vec<PopcornEmitter>, ValueError>;
    fn try_set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) -> Result<(), ValueError>;
}

impl<V: ModelVersion> TryModelAccess for Model<V> {
    fn try_bind_poses(&self) -> Result<Vec<BindPoseMatrix>, ValueError> {
        Model::try_bind_poses(self)
    }
    fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError> {
        Model::try_set_bind_poses(self, poses)
    }
    fn try_face_fx(&self) -> Result<Vec<FaceFx>, ValueError> {
        Model::try_face_fx(self)
    }
    fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError> {
        Model::try_set_face_fx(self, entries)
    }
    fn try_popcorn_emitters(&self) -> Result<Vec<PopcornEmitter>, ValueError> {
        Model::try_popcorn_emitters(self)
    }
    fn try_set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) -> Result<(), ValueError> {
        Model::try_set_popcorn_emitters(self, emitters)
    }
}

impl TryModelAccess for DynamicModel {
    fn try_bind_poses(&self) -> Result<Vec<BindPoseMatrix>, ValueError> {
        visit_model!(self, |model| model.try_bind_poses())
    }
    fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError> {
        visit_model!(self, |model| model.try_set_bind_poses(poses))
    }
    fn try_face_fx(&self) -> Result<Vec<FaceFx>, ValueError> {
        visit_model!(self, |model| model.try_face_fx())
    }
    fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError> {
        visit_model!(self, |model| model.try_set_face_fx(entries))
    }
    fn try_popcorn_emitters(&self) -> Result<Vec<PopcornEmitter>, ValueError> {
        visit_model!(self, |model| model.try_popcorn_emitters())
    }
    fn try_set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) -> Result<(), ValueError> {
        visit_model!(self, |model| model.try_set_popcorn_emitters(emitters))
    }
}
