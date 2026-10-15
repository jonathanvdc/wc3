use crate::model::formats::mdlx::animation::{GeosetAnimation, Sequence, TextureAnimation};
use crate::model::formats::mdlx::emitters::{
    ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter,
};
use crate::model::formats::mdlx::geometry::{BindPoseMatrix, CollisionShape};
use crate::model::formats::mdlx::materials::Texture;
use crate::model::formats::mdlx::scene::{
    Attachment, Bone, EventObject, FaceFx, Glider, ModelInfo, Node,
};
use crate::model::formats::mdlx::visit_document;
use crate::model::formats::mdlx::Dialect;
use crate::model::formats::mdlx::{Document, DynamicDocument, ValueError, Vec3};
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
            DynamicDocument::V800($model) => DynamicRecords::V800($body),
            DynamicDocument::V900($model) => DynamicRecords::V900($body),
            DynamicDocument::V1000($model) => DynamicRecords::V1000($body),
            DynamicDocument::V1100($model) => DynamicRecords::V1100($body),
            DynamicDocument::V1200($model) => DynamicRecords::V1200($body),
            DynamicDocument::V1300($model) => DynamicRecords::V1300($body),
            DynamicDocument::V1400($model) => DynamicRecords::V1400($body),
            DynamicDocument::V1600($model) => DynamicRecords::V1600($body),
            DynamicDocument::V1800($model) => DynamicRecords::V1800($body),
        }
    };
}

macro_rules! try_visit_records {
    ($value:expr, |$model:ident| $body:expr) => {
        match $value {
            DynamicDocument::V800($model) => $body.map(DynamicRecords::V800),
            DynamicDocument::V900($model) => $body.map(DynamicRecords::V900),
            DynamicDocument::V1000($model) => $body.map(DynamicRecords::V1000),
            DynamicDocument::V1100($model) => $body.map(DynamicRecords::V1100),
            DynamicDocument::V1200($model) => $body.map(DynamicRecords::V1200),
            DynamicDocument::V1300($model) => $body.map(DynamicRecords::V1300),
            DynamicDocument::V1400($model) => $body.map(DynamicRecords::V1400),
            DynamicDocument::V1600($model) => $body.map(DynamicRecords::V1600),
            DynamicDocument::V1800($model) => $body.map(DynamicRecords::V1800),
        }
    };
}

/// Read, edit, and replace collections shared by every model version.
///
/// Import this trait to use these operations on [`DynamicDocument`]. Getters for
/// borrowed record collections return iterators without temporary vectors. Mutable
/// iterators edit records in place, preserving chunk order. Record setters consume
/// an iterator of records and move their buffers into one replacement collection.
pub trait CommonModelAccess<'data> {
    /// Borrows glider records in file order.
    fn gliders<'s>(&'s self) -> impl Iterator<Item = &'s Glider> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn gliders_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Glider> + 's
    where
        'data: 's;
    /// Replaces all glider chunks with one collection of the supplied records.
    fn set_gliders(&mut self, gliders: impl IntoIterator<Item = Glider>);
    /// Borrows sequence records in file order.
    fn sequences<'s>(&'s self) -> impl Iterator<Item = &'s Sequence> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn sequences_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Sequence> + 's
    where
        'data: 's;
    /// Replaces all sequence chunks with one collection of the supplied records.
    fn set_sequences(&mut self, sequences: impl IntoIterator<Item = Sequence>);
    /// Returns global-sequence durations in milliseconds, in file order.
    fn global_sequences(&self) -> Vec<u32>;
    /// Replaces all global-sequence chunks with the supplied millisecond durations.
    fn set_global_sequences(&mut self, durations: &[u32]);
    /// Borrows texture-animation records in file order.
    fn texture_animations<'s>(&'s self) -> impl Iterator<Item = &'s TextureAnimation<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn texture_animations_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut TextureAnimation<'data>> + 's
    where
        'data: 's;
    /// Replaces all texture-animation chunks with one collection of the supplied records.
    fn set_texture_animations(
        &mut self,
        animations: impl IntoIterator<Item = TextureAnimation<'data>>,
    );
    /// Borrows geoset-animation records in file order.
    fn geoset_animations<'s>(&'s self) -> impl Iterator<Item = &'s GeosetAnimation<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn geoset_animations_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut GeosetAnimation<'data>> + 's
    where
        'data: 's;
    /// Replaces all geoset-animation chunks with one collection of the supplied records.
    fn set_geoset_animations(
        &mut self,
        animations: impl IntoIterator<Item = GeosetAnimation<'data>>,
    );
    /// Borrows texture records in file order.
    fn textures<'s>(&'s self) -> impl Iterator<Item = &'s Texture> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn textures_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Texture> + 's
    where
        'data: 's;
    /// Replaces all texture chunks with one collection of the supplied records.
    fn set_textures(&mut self, textures: impl IntoIterator<Item = Texture>);
    /// Borrows bone records in file order.
    fn bones<'s>(&'s self) -> impl Iterator<Item = &'s Bone<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn bones_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Bone<'data>> + 's
    where
        'data: 's;
    /// Replaces all bone chunks with one collection of the supplied records.
    fn set_bones(&mut self, bones: impl IntoIterator<Item = Bone<'data>>);
    /// Borrows helper records in file order.
    fn helpers<'s>(&'s self) -> impl Iterator<Item = &'s Node<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn helpers_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Node<'data>> + 's
    where
        'data: 's;
    /// Replaces all helper chunks with one collection of the supplied records.
    fn set_helpers(&mut self, helpers: impl IntoIterator<Item = Node<'data>>);
    /// Borrows attachment records in file order.
    fn attachments<'s>(&'s self) -> impl Iterator<Item = &'s Attachment<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn attachments_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Attachment<'data>> + 's
    where
        'data: 's;
    /// Replaces all attachment chunks with one collection of the supplied records.
    fn set_attachments(&mut self, attachments: impl IntoIterator<Item = Attachment<'data>>);
    /// Borrows event-object records in file order.
    fn event_objects<'s>(&'s self) -> impl Iterator<Item = &'s EventObject<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn event_objects_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut EventObject<'data>> + 's
    where
        'data: 's;
    /// Replaces all event-object chunks with one collection of the supplied records.
    fn set_event_objects(&mut self, events: impl IntoIterator<Item = EventObject<'data>>);
    /// Borrows collision-shape records in file order.
    fn collision_shapes<'s>(&'s self) -> impl Iterator<Item = &'s CollisionShape<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn collision_shapes_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut CollisionShape<'data>> + 's
    where
        'data: 's;
    /// Replaces all collision-shape chunks with one collection of the supplied records.
    fn set_collision_shapes(&mut self, shapes: impl IntoIterator<Item = CollisionShape<'data>>);
    /// Returns copies of pivot-point records in file order.
    fn pivot_points(&self) -> Vec<Vec3>;
    /// Replaces all pivot-point chunks with one collection of the supplied records.
    fn set_pivot_points(&mut self, points: &[Vec3]);
    /// Borrows particle-emitter records in file order.
    fn particle_emitters<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn particle_emitters_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut ParticleEmitter<'data>> + 's
    where
        'data: 's;
    /// Replaces all particle-emitter chunks with one collection of the supplied records.
    fn set_particle_emitters(&mut self, emitters: impl IntoIterator<Item = ParticleEmitter<'data>>);
    /// Borrows particle-emitter-2 records in file order.
    fn particle_emitters2<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter2<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn particle_emitters2_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut ParticleEmitter2<'data>> + 's
    where
        'data: 's;
    /// Replaces all particle-emitter-2 chunks with one collection of the supplied records.
    fn set_particle_emitters2(
        &mut self,
        emitters: impl IntoIterator<Item = ParticleEmitter2<'data>>,
    );
    /// Borrows ribbon-emitter records in file order.
    fn ribbon_emitters<'s>(&'s self) -> impl Iterator<Item = &'s RibbonEmitter<'data>> + 's
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn ribbon_emitters_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut RibbonEmitter<'data>> + 's
    where
        'data: 's;
    /// Replaces all ribbon-emitter chunks with one collection of the supplied records.
    fn set_ribbon_emitters(&mut self, emitters: impl IntoIterator<Item = RibbonEmitter<'data>>);
    /// Returns an owned copy of the first model-info record, or `None` if absent.
    fn model_info(&self) -> Option<ModelInfo>;
    /// Replaces model metadata with an owned copy, creating a chunk if absent.
    /// Removes duplicate metadata chunks and discards MDX extension bytes.
    fn set_model_info(&mut self, info: &ModelInfo);
}

impl<'data, D: Dialect> CommonModelAccess<'data> for Document<'data, D> {
    fn gliders<'s>(&'s self) -> impl Iterator<Item = &'s Glider> + 's
    where
        'data: 's,
    {
        Document::gliders(self)
    }
    fn gliders_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Glider> + 's
    where
        'data: 's,
    {
        Document::gliders_mut(self)
    }
    fn set_gliders(&mut self, gliders: impl IntoIterator<Item = Glider>) {
        Document::set_gliders(self, gliders)
    }

    fn sequences<'s>(&'s self) -> impl Iterator<Item = &'s Sequence> + 's
    where
        'data: 's,
    {
        Document::sequences(self)
    }
    fn sequences_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Sequence> + 's
    where
        'data: 's,
    {
        Document::sequences_mut(self)
    }
    fn set_sequences(&mut self, sequences: impl IntoIterator<Item = Sequence>) {
        Document::set_sequences(self, sequences)
    }
    fn global_sequences(&self) -> Vec<u32> {
        Document::global_sequences(self)
    }
    fn set_global_sequences(&mut self, durations: &[u32]) {
        Document::set_global_sequences(self, durations)
    }
    fn texture_animations<'s>(&'s self) -> impl Iterator<Item = &'s TextureAnimation<'data>> + 's
    where
        'data: 's,
    {
        Document::texture_animations(self)
    }
    fn texture_animations_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut TextureAnimation<'data>> + 's
    where
        'data: 's,
    {
        Document::texture_animations_mut(self)
    }
    fn set_texture_animations(
        &mut self,
        animations: impl IntoIterator<Item = TextureAnimation<'data>>,
    ) {
        Document::set_texture_animations(self, animations)
    }
    fn geoset_animations<'s>(&'s self) -> impl Iterator<Item = &'s GeosetAnimation<'data>> + 's
    where
        'data: 's,
    {
        Document::geoset_animations(self)
    }
    fn geoset_animations_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut GeosetAnimation<'data>> + 's
    where
        'data: 's,
    {
        Document::geoset_animations_mut(self)
    }
    fn set_geoset_animations(
        &mut self,
        animations: impl IntoIterator<Item = GeosetAnimation<'data>>,
    ) {
        Document::set_geoset_animations(self, animations)
    }
    fn textures<'s>(&'s self) -> impl Iterator<Item = &'s Texture> + 's
    where
        'data: 's,
    {
        Document::textures(self)
    }
    fn textures_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Texture> + 's
    where
        'data: 's,
    {
        Document::textures_mut(self)
    }
    fn set_textures(&mut self, textures: impl IntoIterator<Item = Texture>) {
        Document::set_textures(self, textures)
    }
    fn bones<'s>(&'s self) -> impl Iterator<Item = &'s Bone<'data>> + 's
    where
        'data: 's,
    {
        Document::bones(self)
    }
    fn bones_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Bone<'data>> + 's
    where
        'data: 's,
    {
        Document::bones_mut(self)
    }
    fn set_bones(&mut self, bones: impl IntoIterator<Item = Bone<'data>>) {
        Document::set_bones(self, bones)
    }
    fn helpers<'s>(&'s self) -> impl Iterator<Item = &'s Node<'data>> + 's
    where
        'data: 's,
    {
        Document::helpers(self)
    }
    fn helpers_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Node<'data>> + 's
    where
        'data: 's,
    {
        Document::helpers_mut(self)
    }
    fn set_helpers(&mut self, helpers: impl IntoIterator<Item = Node<'data>>) {
        Document::set_helpers(self, helpers)
    }
    fn attachments<'s>(&'s self) -> impl Iterator<Item = &'s Attachment<'data>> + 's
    where
        'data: 's,
    {
        Document::attachments(self)
    }
    fn attachments_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Attachment<'data>> + 's
    where
        'data: 's,
    {
        Document::attachments_mut(self)
    }
    fn set_attachments(&mut self, attachments: impl IntoIterator<Item = Attachment<'data>>) {
        Document::set_attachments(self, attachments)
    }
    fn event_objects<'s>(&'s self) -> impl Iterator<Item = &'s EventObject<'data>> + 's
    where
        'data: 's,
    {
        Document::event_objects(self)
    }
    fn event_objects_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut EventObject<'data>> + 's
    where
        'data: 's,
    {
        Document::event_objects_mut(self)
    }
    fn set_event_objects(&mut self, events: impl IntoIterator<Item = EventObject<'data>>) {
        Document::set_event_objects(self, events)
    }
    fn collision_shapes<'s>(&'s self) -> impl Iterator<Item = &'s CollisionShape<'data>> + 's
    where
        'data: 's,
    {
        Document::collision_shapes(self)
    }
    fn collision_shapes_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut CollisionShape<'data>> + 's
    where
        'data: 's,
    {
        Document::collision_shapes_mut(self)
    }
    fn set_collision_shapes(&mut self, shapes: impl IntoIterator<Item = CollisionShape<'data>>) {
        Document::set_collision_shapes(self, shapes)
    }
    fn pivot_points(&self) -> Vec<Vec3> {
        Document::pivot_points(self)
    }
    fn set_pivot_points(&mut self, points: &[Vec3]) {
        Document::set_pivot_points(self, points)
    }
    fn particle_emitters<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter<'data>> + 's
    where
        'data: 's,
    {
        Document::particle_emitters(self)
    }
    fn particle_emitters_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut ParticleEmitter<'data>> + 's
    where
        'data: 's,
    {
        Document::particle_emitters_mut(self)
    }
    fn set_particle_emitters(
        &mut self,
        emitters: impl IntoIterator<Item = ParticleEmitter<'data>>,
    ) {
        Document::set_particle_emitters(self, emitters)
    }
    fn particle_emitters2<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter2<'data>> + 's
    where
        'data: 's,
    {
        Document::particle_emitters2(self)
    }
    fn particle_emitters2_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut ParticleEmitter2<'data>> + 's
    where
        'data: 's,
    {
        Document::particle_emitters2_mut(self)
    }
    fn set_particle_emitters2(
        &mut self,
        emitters: impl IntoIterator<Item = ParticleEmitter2<'data>>,
    ) {
        Document::set_particle_emitters2(self, emitters)
    }
    fn ribbon_emitters<'s>(&'s self) -> impl Iterator<Item = &'s RibbonEmitter<'data>> + 's
    where
        'data: 's,
    {
        Document::ribbon_emitters(self)
    }
    fn ribbon_emitters_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut RibbonEmitter<'data>> + 's
    where
        'data: 's,
    {
        Document::ribbon_emitters_mut(self)
    }
    fn set_ribbon_emitters(&mut self, emitters: impl IntoIterator<Item = RibbonEmitter<'data>>) {
        Document::set_ribbon_emitters(self, emitters)
    }
    fn model_info(&self) -> Option<ModelInfo> {
        Document::model_info(self)
    }
    fn set_model_info(&mut self, info: &ModelInfo) {
        Document::set_model_info(self, info)
    }
}

impl<'data, E: Clone + Debug> CommonModelAccess<'data> for DynamicDocument<'data, E> {
    fn gliders<'s>(&'s self) -> impl Iterator<Item = &'s Glider> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.gliders())
    }
    fn gliders_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Glider> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.gliders_mut())
    }
    fn set_gliders(&mut self, gliders: impl IntoIterator<Item = Glider>) {
        visit_document!(self, |model| model.set_gliders(gliders))
    }

    fn sequences<'s>(&'s self) -> impl Iterator<Item = &'s Sequence> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.sequences())
    }
    fn sequences_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Sequence> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.sequences_mut())
    }
    fn set_sequences(&mut self, sequences: impl IntoIterator<Item = Sequence>) {
        visit_document!(self, |model| model.set_sequences(sequences))
    }
    fn global_sequences(&self) -> Vec<u32> {
        visit_document!(self, |model| model.global_sequences())
    }
    fn set_global_sequences(&mut self, durations: &[u32]) {
        visit_document!(self, |model| model.set_global_sequences(durations))
    }
    fn texture_animations<'s>(&'s self) -> impl Iterator<Item = &'s TextureAnimation<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.texture_animations())
    }
    fn texture_animations_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut TextureAnimation<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.texture_animations_mut())
    }
    fn set_texture_animations(
        &mut self,
        animations: impl IntoIterator<Item = TextureAnimation<'data>>,
    ) {
        visit_document!(self, |model| model.set_texture_animations(animations))
    }
    fn geoset_animations<'s>(&'s self) -> impl Iterator<Item = &'s GeosetAnimation<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.geoset_animations())
    }
    fn geoset_animations_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut GeosetAnimation<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.geoset_animations_mut())
    }
    fn set_geoset_animations(
        &mut self,
        animations: impl IntoIterator<Item = GeosetAnimation<'data>>,
    ) {
        visit_document!(self, |model| model.set_geoset_animations(animations))
    }
    fn textures<'s>(&'s self) -> impl Iterator<Item = &'s Texture> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.textures())
    }
    fn textures_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Texture> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.textures_mut())
    }
    fn set_textures(&mut self, textures: impl IntoIterator<Item = Texture>) {
        visit_document!(self, |model| model.set_textures(textures))
    }
    fn bones<'s>(&'s self) -> impl Iterator<Item = &'s Bone<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.bones())
    }
    fn bones_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Bone<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.bones_mut())
    }
    fn set_bones(&mut self, bones: impl IntoIterator<Item = Bone<'data>>) {
        visit_document!(self, |model| model.set_bones(bones))
    }
    fn helpers<'s>(&'s self) -> impl Iterator<Item = &'s Node<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.helpers())
    }
    fn helpers_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Node<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.helpers_mut())
    }
    fn set_helpers(&mut self, helpers: impl IntoIterator<Item = Node<'data>>) {
        visit_document!(self, |model| model.set_helpers(helpers))
    }
    fn attachments<'s>(&'s self) -> impl Iterator<Item = &'s Attachment<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.attachments())
    }
    fn attachments_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut Attachment<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.attachments_mut())
    }
    fn set_attachments(&mut self, attachments: impl IntoIterator<Item = Attachment<'data>>) {
        visit_document!(self, |model| model.set_attachments(attachments))
    }
    fn event_objects<'s>(&'s self) -> impl Iterator<Item = &'s EventObject<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.event_objects())
    }
    fn event_objects_mut<'s>(&'s mut self) -> impl Iterator<Item = &'s mut EventObject<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.event_objects_mut())
    }
    fn set_event_objects(&mut self, events: impl IntoIterator<Item = EventObject<'data>>) {
        visit_document!(self, |model| model.set_event_objects(events))
    }
    fn collision_shapes<'s>(&'s self) -> impl Iterator<Item = &'s CollisionShape<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.collision_shapes())
    }
    fn collision_shapes_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut CollisionShape<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.collision_shapes_mut())
    }
    fn set_collision_shapes(&mut self, shapes: impl IntoIterator<Item = CollisionShape<'data>>) {
        visit_document!(self, |model| model.set_collision_shapes(shapes))
    }
    fn pivot_points(&self) -> Vec<Vec3> {
        visit_document!(self, |model| model.pivot_points())
    }
    fn set_pivot_points(&mut self, points: &[Vec3]) {
        visit_document!(self, |model| model.set_pivot_points(points))
    }
    fn particle_emitters<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.particle_emitters())
    }
    fn particle_emitters_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut ParticleEmitter<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.particle_emitters_mut())
    }
    fn set_particle_emitters(
        &mut self,
        emitters: impl IntoIterator<Item = ParticleEmitter<'data>>,
    ) {
        visit_document!(self, |model| model.set_particle_emitters(emitters))
    }
    fn particle_emitters2<'s>(&'s self) -> impl Iterator<Item = &'s ParticleEmitter2<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.particle_emitters2())
    }
    fn particle_emitters2_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut ParticleEmitter2<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.particle_emitters2_mut())
    }
    fn set_particle_emitters2(
        &mut self,
        emitters: impl IntoIterator<Item = ParticleEmitter2<'data>>,
    ) {
        visit_document!(self, |model| model.set_particle_emitters2(emitters))
    }
    fn ribbon_emitters<'s>(&'s self) -> impl Iterator<Item = &'s RibbonEmitter<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.ribbon_emitters())
    }
    fn ribbon_emitters_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut RibbonEmitter<'data>> + 's
    where
        'data: 's,
    {
        visit_records!(self, |model| model.ribbon_emitters_mut())
    }
    fn set_ribbon_emitters(&mut self, emitters: impl IntoIterator<Item = RibbonEmitter<'data>>) {
        visit_document!(self, |model| model.set_ribbon_emitters(emitters))
    }
    fn model_info(&self) -> Option<ModelInfo> {
        visit_document!(self, |model| model.model_info())
    }
    fn set_model_info(&mut self, info: &ModelInfo) {
        visit_document!(self, |model| model.set_model_info(info))
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
    /// Mutably borrows records in file order without replacing their chunks.
    fn try_bind_poses_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut BindPoseMatrix> + 's, ValueError>
    where
        'data: 's;
    /// Replaces all bind-pose chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_bind_poses(
        &mut self,
        poses: impl IntoIterator<Item = BindPoseMatrix>,
    ) -> Result<(), ValueError>;
    /// Borrows FaceFX records in file order.
    /// Returns an error if the model version does not support this collection.
    fn try_face_fx<'s>(&'s self) -> Result<impl Iterator<Item = &'s FaceFx> + 's, ValueError>
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn try_face_fx_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut FaceFx> + 's, ValueError>
    where
        'data: 's;
    /// Replaces all FaceFX chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_face_fx(
        &mut self,
        entries: impl IntoIterator<Item = FaceFx>,
    ) -> Result<(), ValueError>;
    /// Borrows Popcorn-emitter records in file order.
    /// Returns an error if the model version does not support this collection.
    fn try_popcorn_emitters<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's;
    /// Mutably borrows records in file order without replacing their chunks.
    fn try_popcorn_emitters_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's;
    /// Replaces all Popcorn-emitter chunks with one collection of the supplied records.
    /// Returns an error if the model version does not support this collection.
    fn try_set_popcorn_emitters(
        &mut self,
        emitters: impl IntoIterator<Item = PopcornEmitter<'data>>,
    ) -> Result<(), ValueError>;
}

impl<'data, D: Dialect> TryModelAccess<'data> for Document<'data, D> {
    fn try_bind_poses<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s BindPoseMatrix> + 's, ValueError>
    where
        'data: 's,
    {
        Document::try_bind_poses(self)
    }
    fn try_bind_poses_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut BindPoseMatrix> + 's, ValueError>
    where
        'data: 's,
    {
        Document::try_bind_poses_mut(self)
    }
    fn try_set_bind_poses(
        &mut self,
        poses: impl IntoIterator<Item = BindPoseMatrix>,
    ) -> Result<(), ValueError> {
        Document::try_set_bind_poses(self, poses)
    }
    fn try_face_fx<'s>(&'s self) -> Result<impl Iterator<Item = &'s FaceFx> + 's, ValueError>
    where
        'data: 's,
    {
        Document::try_face_fx(self)
    }
    fn try_face_fx_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut FaceFx> + 's, ValueError>
    where
        'data: 's,
    {
        Document::try_face_fx_mut(self)
    }
    fn try_set_face_fx(
        &mut self,
        entries: impl IntoIterator<Item = FaceFx>,
    ) -> Result<(), ValueError> {
        Document::try_set_face_fx(self, entries)
    }
    fn try_popcorn_emitters<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's,
    {
        Document::try_popcorn_emitters(self)
    }
    fn try_popcorn_emitters_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's,
    {
        Document::try_popcorn_emitters_mut(self)
    }
    fn try_set_popcorn_emitters(
        &mut self,
        emitters: impl IntoIterator<Item = PopcornEmitter<'data>>,
    ) -> Result<(), ValueError> {
        Document::try_set_popcorn_emitters(self, emitters)
    }
}

impl<'data, E: Clone + Debug> TryModelAccess<'data> for DynamicDocument<'data, E> {
    fn try_bind_poses<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s BindPoseMatrix> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_bind_poses())
    }
    fn try_bind_poses_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut BindPoseMatrix> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_bind_poses_mut())
    }
    fn try_set_bind_poses(
        &mut self,
        poses: impl IntoIterator<Item = BindPoseMatrix>,
    ) -> Result<(), ValueError> {
        visit_document!(self, |model| model.try_set_bind_poses(poses))
    }
    fn try_face_fx<'s>(&'s self) -> Result<impl Iterator<Item = &'s FaceFx> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_face_fx())
    }
    fn try_face_fx_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut FaceFx> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_face_fx_mut())
    }
    fn try_set_face_fx(
        &mut self,
        entries: impl IntoIterator<Item = FaceFx>,
    ) -> Result<(), ValueError> {
        visit_document!(self, |model| model.try_set_face_fx(entries))
    }
    fn try_popcorn_emitters<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_popcorn_emitters())
    }
    fn try_popcorn_emitters_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut PopcornEmitter<'data>> + 's, ValueError>
    where
        'data: 's,
    {
        try_visit_records!(self, |model| model.try_popcorn_emitters_mut())
    }
    fn try_set_popcorn_emitters(
        &mut self,
        emitters: impl IntoIterator<Item = PopcornEmitter<'data>>,
    ) -> Result<(), ValueError> {
        visit_document!(self, |model| model.try_set_popcorn_emitters(emitters))
    }
}
