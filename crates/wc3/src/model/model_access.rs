use crate::model::animation::{GeosetAnimation, Sequence, TextureAnimation};
use crate::model::emitters::{ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter};
use crate::model::geometry::{BindPoseMatrix, CollisionShape};
use crate::model::materials::Texture;
use crate::model::scene::{Attachment, Bone, EventObject, FaceFx, Glider, ModelInfo, Node};
use crate::model::visit_model;
use crate::model::ModelDialect;
use crate::model::{DynamicModel, Model, ValueError, Vec3};
use std::fmt::Debug;

// Each runtime version keeps its concrete iterator on the stack.
enum DynamicRecords<I800, I900, I1000, I1100, I1200, I1300, I1400, I1600, I1800> {
    V800(I800),
    V900(I900),
    V1000(I1000),
    V1100(I1100),
    V1200(I1200),
    V1300(I1300),
    V1400(I1400),
    V1600(I1600),
    V1800(I1800),
}

impl<T, I800, I900, I1000, I1100, I1200, I1300, I1400, I1600, I1800> Iterator
    for DynamicRecords<I800, I900, I1000, I1100, I1200, I1300, I1400, I1600, I1800>
where
    I800: Iterator<Item = T>,
    I900: Iterator<Item = T>,
    I1000: Iterator<Item = T>,
    I1100: Iterator<Item = T>,
    I1200: Iterator<Item = T>,
    I1300: Iterator<Item = T>,
    I1400: Iterator<Item = T>,
    I1600: Iterator<Item = T>,
    I1800: Iterator<Item = T>,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self {
            Self::V800(records) => records.next(),
            Self::V900(records) => records.next(),
            Self::V1000(records) => records.next(),
            Self::V1100(records) => records.next(),
            Self::V1200(records) => records.next(),
            Self::V1300(records) => records.next(),
            Self::V1400(records) => records.next(),
            Self::V1600(records) => records.next(),
            Self::V1800(records) => records.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::V800(records) => records.size_hint(),
            Self::V900(records) => records.size_hint(),
            Self::V1000(records) => records.size_hint(),
            Self::V1100(records) => records.size_hint(),
            Self::V1200(records) => records.size_hint(),
            Self::V1300(records) => records.size_hint(),
            Self::V1400(records) => records.size_hint(),
            Self::V1600(records) => records.size_hint(),
            Self::V1800(records) => records.size_hint(),
        }
    }
}

macro_rules! visit_records {
    ($value:expr, |$model:ident| $body:expr) => {
        match $value {
            DynamicModel::V800($model) => DynamicRecords::V800($body),
            DynamicModel::V900($model) => DynamicRecords::V900($body),
            DynamicModel::V1000($model) => DynamicRecords::V1000($body),
            DynamicModel::V1100($model) => DynamicRecords::V1100($body),
            DynamicModel::V1200($model) => DynamicRecords::V1200($body),
            DynamicModel::V1300($model) => DynamicRecords::V1300($body),
            DynamicModel::V1400($model) => DynamicRecords::V1400($body),
            DynamicModel::V1600($model) => DynamicRecords::V1600($body),
            DynamicModel::V1800($model) => DynamicRecords::V1800($body),
        }
    };
}

macro_rules! try_visit_records {
    ($value:expr, |$model:ident| $body:expr) => {
        match $value {
            DynamicModel::V800($model) => $body.map(DynamicRecords::V800),
            DynamicModel::V900($model) => $body.map(DynamicRecords::V900),
            DynamicModel::V1000($model) => $body.map(DynamicRecords::V1000),
            DynamicModel::V1100($model) => $body.map(DynamicRecords::V1100),
            DynamicModel::V1200($model) => $body.map(DynamicRecords::V1200),
            DynamicModel::V1300($model) => $body.map(DynamicRecords::V1300),
            DynamicModel::V1400($model) => $body.map(DynamicRecords::V1400),
            DynamicModel::V1600($model) => $body.map(DynamicRecords::V1600),
            DynamicModel::V1800($model) => $body.map(DynamicRecords::V1800),
        }
    };
}

/// Read and replace collections shared by every model version.
///
/// Import this trait to use these operations on [`DynamicModel`]. Getters for
/// borrowed record collections return iterators without temporary vectors. Setters
/// clone records, preserving the borrowed or owned state of each buffer.
/// Collection setters replace all chunks of that kind with one collection.
pub trait CommonModelAccess<'data> {
    /// Borrows glider records in file order.
    fn gliders<'s>(&'s self) -> impl Iterator<Item = &'s Glider> + 's
    where
        'data: 's;
    /// Replaces all glider chunks with one collection of the supplied records.
    fn set_gliders(&mut self, gliders: &[Glider]);
    /// Borrows sequence records in file order.
    fn sequences<'s>(&'s self) -> impl Iterator<Item = &'s Sequence> + 's
    where
        'data: 's;
    /// Replaces all sequence chunks with one collection of the supplied records.
    fn set_sequences(&mut self, sequences: &[Sequence]);
    /// Returns global-sequence durations in milliseconds, in file order.
    fn global_sequences(&self) -> Vec<u32>;
    /// Replaces all global-sequence chunks with the supplied millisecond durations.
    fn set_global_sequences(&mut self, durations: &[u32]);
    /// Borrows texture-animation records in file order.
    fn texture_animations<'s>(&'s self) -> impl Iterator<Item = &'s TextureAnimation<'data>> + 's
    where
        'data: 's;
    /// Replaces all texture-animation chunks with one collection of the supplied records.
    fn set_texture_animations(&mut self, animations: &[TextureAnimation<'data>]);
    /// Borrows geoset-animation records in file order.
    fn geoset_animations<'s>(&'s self) -> impl Iterator<Item = &'s GeosetAnimation<'data>> + 's
    where
        'data: 's;
    /// Replaces all geoset-animation chunks with one collection of the supplied records.
    fn set_geoset_animations(&mut self, animations: &[GeosetAnimation<'data>]);
    /// Borrows texture records in file order.
    fn textures<'s>(&'s self) -> impl Iterator<Item = &'s Texture> + 's
    where
        'data: 's;
    /// Replaces all texture chunks with one collection of the supplied records.
    fn set_textures(&mut self, textures: &[Texture]);
    /// Borrows bone records in file order.
    fn bones<'s>(&'s self) -> impl Iterator<Item = &'s Bone<'data>> + 's
    where
        'data: 's;
    /// Replaces all bone chunks with one collection of the supplied records.
    fn set_bones(&mut self, bones: &[Bone<'data>]);
    /// Borrows helper records in file order.
    fn helpers<'s>(&'s self) -> impl Iterator<Item = &'s Node<'data>> + 's
    where
        'data: 's;
    /// Replaces all helper chunks with one collection of the supplied records.
    fn set_helpers(&mut self, helpers: &[Node<'data>]);
    /// Borrows attachment records in file order.
    fn attachments<'s>(&'s self) -> impl Iterator<Item = &'s Attachment<'data>> + 's
    where
        'data: 's;
    /// Replaces all attachment chunks with one collection of the supplied records.
    fn set_attachments(&mut self, attachments: &[Attachment<'data>]);
    /// Borrows event-object records in file order.
    fn event_objects<'s>(&'s self) -> impl Iterator<Item = &'s EventObject<'data>> + 's
    where
        'data: 's;
    /// Replaces all event-object chunks with one collection of the supplied records.
    fn set_event_objects(&mut self, events: &[EventObject<'data>]);
    /// Borrows collision-shape records in file order.
    fn collision_shapes<'s>(&'s self) -> impl Iterator<Item = &'s CollisionShape<'data>> + 's
    where
        'data: 's;
    /// Replaces all collision-shape chunks with one collection of the supplied records.
    fn set_collision_shapes(&mut self, shapes: &[CollisionShape<'data>]);
    /// Returns copies of pivot-point records in file order.
    fn pivot_points(&self) -> Vec<Vec3>;
    /// Replaces all pivot-point chunks with one collection of the supplied records.
    fn set_pivot_points(&mut self, points: &[Vec3]);
    /// Borrows particle-emitter records in file order.
    fn particle_emitters<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter<'data>> + 's
    where
        'data: 's;
    /// Replaces all particle-emitter chunks with one collection of the supplied records.
    fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter<'data>]);
    /// Borrows particle-emitter-2 records in file order.
    fn particle_emitters2<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter2<'data>> + 's
    where
        'data: 's;
    /// Replaces all particle-emitter-2 chunks with one collection of the supplied records.
    fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2<'data>]);
    /// Borrows ribbon-emitter records in file order.
    fn ribbon_emitters<'s>(&'s self) -> impl Iterator<Item = &'s RibbonEmitter<'data>> + 's
    where
        'data: 's;
    /// Replaces all ribbon-emitter chunks with one collection of the supplied records.
    fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter<'data>]);
    /// Returns an owned copy of the first model-info record, or `None` if absent.
    fn model_info(&self) -> Option<ModelInfo>;
    /// Replaces model metadata with an owned copy, creating a chunk if absent.
    /// Removes duplicate metadata chunks and discards MDX extension bytes.
    fn set_model_info(&mut self, info: &ModelInfo);
}

impl<'data, D: ModelDialect> CommonModelAccess<'data> for Model<'data, D> {
    fn gliders<'s>(&'s self) -> impl Iterator<Item = &'s Glider> + 's
    where
        'data: 's,
    {
        Model::gliders(self)
    }
    fn set_gliders(&mut self, gliders: &[Glider]) {
        Model::set_gliders(self, gliders)
    }

    fn sequences<'s>(&'s self) -> impl Iterator<Item = &'s Sequence> + 's
    where
        'data: 's,
    {
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
    fn texture_animations<'s>(&'s self) -> impl Iterator<Item = &'s TextureAnimation<'data>> + 's
    where
        'data: 's,
    {
        Model::texture_animations(self)
    }
    fn set_texture_animations(&mut self, animations: &[TextureAnimation<'data>]) {
        Model::set_texture_animations(self, animations)
    }
    fn geoset_animations<'s>(&'s self) -> impl Iterator<Item = &'s GeosetAnimation<'data>> + 's
    where
        'data: 's,
    {
        Model::geoset_animations(self)
    }
    fn set_geoset_animations(&mut self, animations: &[GeosetAnimation<'data>]) {
        Model::set_geoset_animations(self, animations)
    }
    fn textures<'s>(&'s self) -> impl Iterator<Item = &'s Texture> + 's
    where
        'data: 's,
    {
        Model::textures(self)
    }
    fn set_textures(&mut self, textures: &[Texture]) {
        Model::set_textures(self, textures)
    }
    fn bones<'s>(&'s self) -> impl Iterator<Item = &'s Bone<'data>> + 's
    where
        'data: 's,
    {
        Model::bones(self)
    }
    fn set_bones(&mut self, bones: &[Bone<'data>]) {
        Model::set_bones(self, bones)
    }
    fn helpers<'s>(&'s self) -> impl Iterator<Item = &'s Node<'data>> + 's
    where
        'data: 's,
    {
        Model::helpers(self)
    }
    fn set_helpers(&mut self, helpers: &[Node<'data>]) {
        Model::set_helpers(self, helpers)
    }
    fn attachments<'s>(&'s self) -> impl Iterator<Item = &'s Attachment<'data>> + 's
    where
        'data: 's,
    {
        Model::attachments(self)
    }
    fn set_attachments(&mut self, attachments: &[Attachment<'data>]) {
        Model::set_attachments(self, attachments)
    }
    fn event_objects<'s>(&'s self) -> impl Iterator<Item = &'s EventObject<'data>> + 's
    where
        'data: 's,
    {
        Model::event_objects(self)
    }
    fn set_event_objects(&mut self, events: &[EventObject<'data>]) {
        Model::set_event_objects(self, events)
    }
    fn collision_shapes<'s>(&'s self) -> impl Iterator<Item = &'s CollisionShape<'data>> + 's
    where
        'data: 's,
    {
        Model::collision_shapes(self)
    }
    fn set_collision_shapes(&mut self, shapes: &[CollisionShape<'data>]) {
        Model::set_collision_shapes(self, shapes)
    }
    fn pivot_points(&self) -> Vec<Vec3> {
        Model::pivot_points(self)
    }
    fn set_pivot_points(&mut self, points: &[Vec3]) {
        Model::set_pivot_points(self, points)
    }
    fn particle_emitters<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter<'data>> + 's
    where
        'data: 's,
    {
        Model::particle_emitters(self)
    }
    fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter<'data>]) {
        Model::set_particle_emitters(self, emitters)
    }
    fn particle_emitters2<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter2<'data>> + 's
    where
        'data: 's,
    {
        Model::particle_emitters2(self)
    }
    fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2<'data>]) {
        Model::set_particle_emitters2(self, emitters)
    }
    fn ribbon_emitters<'s>(&'s self) -> impl Iterator<Item = &'s RibbonEmitter<'data>> + 's
    where
        'data: 's,
    {
        Model::ribbon_emitters(self)
    }
    fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter<'data>]) {
        Model::set_ribbon_emitters(self, emitters)
    }
    fn model_info(&self) -> Option<ModelInfo> {
        Model::model_info(self)
    }
    fn set_model_info(&mut self, info: &ModelInfo) {
        Model::set_model_info(self, info)
    }
}

impl<'data, E: Clone + Debug> CommonModelAccess<'data> for DynamicModel<'data, E> {
    fn gliders<'s>(&'s self) -> impl Iterator<Item = &'s Glider> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.gliders())
    }
    fn set_gliders(&mut self, gliders: &[Glider]) {
        visit_model!(self, |model| model.set_gliders(gliders))
    }

    fn sequences<'s>(&'s self) -> impl Iterator<Item = &'s Sequence> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.sequences())
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
    fn texture_animations<'s>(&'s self) -> impl Iterator<Item = &'s TextureAnimation<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.texture_animations())
    }
    fn set_texture_animations(&mut self, animations: &[TextureAnimation<'data>]) {
        visit_model!(self, |model| model.set_texture_animations(animations))
    }
    fn geoset_animations<'s>(&'s self) -> impl Iterator<Item = &'s GeosetAnimation<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.geoset_animations())
    }
    fn set_geoset_animations(&mut self, animations: &[GeosetAnimation<'data>]) {
        visit_model!(self, |model| model.set_geoset_animations(animations))
    }
    fn textures<'s>(&'s self) -> impl Iterator<Item = &'s Texture> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.textures())
    }
    fn set_textures(&mut self, textures: &[Texture]) {
        visit_model!(self, |model| model.set_textures(textures))
    }
    fn bones<'s>(&'s self) -> impl Iterator<Item = &'s Bone<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.bones())
    }
    fn set_bones(&mut self, bones: &[Bone<'data>]) {
        visit_model!(self, |model| model.set_bones(bones))
    }
    fn helpers<'s>(&'s self) -> impl Iterator<Item = &'s Node<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.helpers())
    }
    fn set_helpers(&mut self, helpers: &[Node<'data>]) {
        visit_model!(self, |model| model.set_helpers(helpers))
    }
    fn attachments<'s>(&'s self) -> impl Iterator<Item = &'s Attachment<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.attachments())
    }
    fn set_attachments(&mut self, attachments: &[Attachment<'data>]) {
        visit_model!(self, |model| model.set_attachments(attachments))
    }
    fn event_objects<'s>(&'s self) -> impl Iterator<Item = &'s EventObject<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.event_objects())
    }
    fn set_event_objects(&mut self, events: &[EventObject<'data>]) {
        visit_model!(self, |model| model.set_event_objects(events))
    }
    fn collision_shapes<'s>(&'s self) -> impl Iterator<Item = &'s CollisionShape<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.collision_shapes())
    }
    fn set_collision_shapes(&mut self, shapes: &[CollisionShape<'data>]) {
        visit_model!(self, |model| model.set_collision_shapes(shapes))
    }
    fn pivot_points(&self) -> Vec<Vec3> {
        visit_model!(self, |model| model.pivot_points())
    }
    fn set_pivot_points(&mut self, points: &[Vec3]) {
        visit_model!(self, |model| model.set_pivot_points(points))
    }
    fn particle_emitters<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.particle_emitters())
    }
    fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter<'data>]) {
        visit_model!(self, |model| model.set_particle_emitters(emitters))
    }
    fn particle_emitters2<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter2<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.particle_emitters2())
    }
    fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2<'data>]) {
        visit_model!(self, |model| model.set_particle_emitters2(emitters))
    }
    fn ribbon_emitters<'s>(&'s self) -> impl Iterator<Item = &'s RibbonEmitter<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.ribbon_emitters())
    }
    fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter<'data>]) {
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
/// Methods return `ValueError` for versions before 900. Getters borrow records;
/// setters replace collections.
pub trait TryModelAccess<'data> {
    /// Borrows bind-pose records in file order.
    /// Returns an error if the model version does not support this collection.
    fn try_bind_poses<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s BindPoseMatrix> + 's, ValueError>
    where
        'data: 's;
    /// Replaces all bind-pose chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError>;
    /// Borrows FaceFX records in file order.
    /// Returns an error if the model version does not support this collection.
    fn try_face_fx<'s>(&'s self) -> Result<impl Iterator<Item = &'s FaceFx> + 's, ValueError>
    where
        'data: 's;
    /// Replaces all FaceFX chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError>;
    /// Borrows Popcorn-emitter records in file order.
    /// Returns an error if the model version does not support this collection.
    fn try_popcorn_emitters<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's;
    /// Replaces all Popcorn-emitter chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_popcorn_emitters(
        &mut self,
        emitters: &[PopcornEmitter<'data>],
    ) -> Result<(), ValueError>;
}

impl<'data, D: ModelDialect> TryModelAccess<'data> for Model<'data, D> {
    fn try_bind_poses<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s BindPoseMatrix> + 's, ValueError>
    where
        'data: 's,
    {
        Model::try_bind_poses(self)
    }
    fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError> {
        Model::try_set_bind_poses(self, poses)
    }
    fn try_face_fx<'s>(&'s self) -> Result<impl Iterator<Item = &'s FaceFx> + 's, ValueError>
    where
        'data: 's,
    {
        Model::try_face_fx(self)
    }
    fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError> {
        Model::try_set_face_fx(self, entries)
    }
    fn try_popcorn_emitters<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's,
    {
        Model::try_popcorn_emitters(self)
    }
    fn try_set_popcorn_emitters(
        &mut self,
        emitters: &[PopcornEmitter<'data>],
    ) -> Result<(), ValueError> {
        Model::try_set_popcorn_emitters(self, emitters)
    }
}

impl<'data, E: Clone + Debug> TryModelAccess<'data> for DynamicModel<'data, E> {
    fn try_bind_poses<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s BindPoseMatrix> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_bind_poses())
    }
    fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError> {
        visit_model!(self, |model| model.try_set_bind_poses(poses))
    }
    fn try_face_fx<'s>(&'s self) -> Result<impl Iterator<Item = &'s FaceFx> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_face_fx())
    }
    fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError> {
        visit_model!(self, |model| model.try_set_face_fx(entries))
    }
    fn try_popcorn_emitters<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_popcorn_emitters())
    }
    fn try_set_popcorn_emitters(
        &mut self,
        emitters: &[PopcornEmitter<'data>],
    ) -> Result<(), ValueError> {
        visit_model!(self, |model| model.try_set_popcorn_emitters(emitters))
    }
}
