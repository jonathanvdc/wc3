use crate::model::animation::{GeosetAnimation, Sequence, TextureAnimation};
use crate::model::emitters::{ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter};
use crate::model::geometry::{BindPoseMatrix, CollisionShape};
use crate::model::materials::Texture;
use crate::model::scene::{Attachment, Bone, EventObject, FaceFx, Glider, ModelInfo, Node};
use crate::model::visit_model;
use crate::model::{DynamicModel, Model, ValueError, Vec3};
use crate::model::{ModelDialect, ModelExtension};

/// Read and replace collections shared by every model version.
///
/// Import this trait to use these operations on [`DynamicModel`]. Getters return
/// owned copies; edit them and pass them to a setter to update the model.
/// Collection setters replace all chunks of that kind with one collection.
pub trait CommonModelAccess {
    /// Returns owned copies of glider records in file order.
    fn gliders(&self) -> Vec<Glider>;
    /// Replaces all glider chunks with one collection of the supplied records.
    fn set_gliders(&mut self, gliders: &[Glider]);
    /// Returns owned copies of sequence records in file order.
    fn sequences(&self) -> Vec<Sequence>;
    /// Replaces all sequence chunks with one collection of the supplied records.
    fn set_sequences(&mut self, sequences: &[Sequence]);
    /// Returns global-sequence durations in milliseconds, in file order.
    fn global_sequences(&self) -> Vec<u32>;
    /// Replaces all global-sequence chunks with the supplied millisecond durations.
    fn set_global_sequences(&mut self, durations: &[u32]);
    /// Returns owned copies of texture-animation records in file order.
    fn texture_animations(&self) -> Vec<TextureAnimation>;
    /// Replaces all texture-animation chunks with one collection of the supplied records.
    fn set_texture_animations(&mut self, animations: &[TextureAnimation]);
    /// Returns owned copies of geoset-animation records in file order.
    fn geoset_animations(&self) -> Vec<GeosetAnimation>;
    /// Replaces all geoset-animation chunks with one collection of the supplied records.
    fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]);
    /// Returns owned copies of texture records in file order.
    fn textures(&self) -> Vec<Texture>;
    /// Replaces all texture chunks with one collection of the supplied records.
    fn set_textures(&mut self, textures: &[Texture]);
    /// Returns owned copies of bone records in file order.
    fn bones(&self) -> Vec<Bone>;
    /// Replaces all bone chunks with one collection of the supplied records.
    fn set_bones(&mut self, bones: &[Bone]);
    /// Returns owned copies of helper records in file order.
    fn helpers(&self) -> Vec<Node>;
    /// Replaces all helper chunks with one collection of the supplied records.
    fn set_helpers(&mut self, helpers: &[Node]);
    /// Returns owned copies of attachment records in file order.
    fn attachments(&self) -> Vec<Attachment>;
    /// Replaces all attachment chunks with one collection of the supplied records.
    fn set_attachments(&mut self, attachments: &[Attachment]);
    /// Returns owned copies of event-object records in file order.
    fn event_objects(&self) -> Vec<EventObject>;
    /// Replaces all event-object chunks with one collection of the supplied records.
    fn set_event_objects(&mut self, events: &[EventObject]);
    /// Returns owned copies of collision-shape records in file order.
    fn collision_shapes(&self) -> Vec<CollisionShape>;
    /// Replaces all collision-shape chunks with one collection of the supplied records.
    fn set_collision_shapes(&mut self, shapes: &[CollisionShape]);
    /// Returns owned copies of pivot-point records in file order.
    fn pivot_points(&self) -> Vec<Vec3>;
    /// Replaces all pivot-point chunks with one collection of the supplied records.
    fn set_pivot_points(&mut self, points: &[Vec3]);
    /// Returns owned copies of particle-emitter records in file order.
    fn particle_emitters(&self) -> Vec<ParticleEmitter>;
    /// Replaces all particle-emitter chunks with one collection of the supplied records.
    fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]);
    /// Returns owned copies of particle-emitter-2 records in file order.
    fn particle_emitters2(&self) -> Vec<ParticleEmitter2>;
    /// Replaces all particle-emitter-2 chunks with one collection of the supplied records.
    fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]);
    /// Returns owned copies of ribbon-emitter records in file order.
    fn ribbon_emitters(&self) -> Vec<RibbonEmitter>;
    /// Replaces all ribbon-emitter chunks with one collection of the supplied records.
    fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]);
    /// Returns an owned copy of the first model-info record, or `None` if absent.
    fn model_info(&self) -> Option<ModelInfo>;
    /// Replaces model metadata with an owned copy, creating a chunk if absent.
    /// Removes duplicate metadata chunks and discards MDX extension bytes.
    fn set_model_info(&mut self, info: &ModelInfo);
}

impl<D: ModelDialect> CommonModelAccess for Model<D> {
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

impl<E: ModelExtension> CommonModelAccess for DynamicModel<E> {
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
    /// Returns owned copies of bind-pose records in file order.
    /// Returns an error if the model version does not support this collection.
    fn try_bind_poses(&self) -> Result<Vec<BindPoseMatrix>, ValueError>;
    /// Replaces all bind-pose chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError>;
    /// Returns owned copies of FaceFX records in file order.
    /// Returns an error if the model version does not support this collection.
    fn try_face_fx(&self) -> Result<Vec<FaceFx>, ValueError>;
    /// Replaces all FaceFX chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError>;
    /// Returns owned copies of Popcorn-emitter records in file order.
    /// Returns an error if the model version does not support this collection.
    fn try_popcorn_emitters(&self) -> Result<Vec<PopcornEmitter>, ValueError>;
    /// Replaces all Popcorn-emitter chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) -> Result<(), ValueError>;
}

impl<D: ModelDialect> TryModelAccess for Model<D> {
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

impl<E: ModelExtension> TryModelAccess for DynamicModel<E> {
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
