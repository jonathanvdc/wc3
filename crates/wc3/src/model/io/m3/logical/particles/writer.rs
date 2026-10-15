//! Output-only identity maps relocate logical emitter links and leaf arrays.
use super::super::{
    animation::AnimationIds,
    write::{animated, append, string},
};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::{profiles::m3 as surface, FeatureId, MaterialId, MeshId};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
pub(in crate::model::io::m3::logical) struct Writer<'out, 'draft, E: Envelope> {
    pub document: &'draft mut native::Container<'out, E>,
    pub ids: &'draft AnimationIds,
    pub context: &'draft mut Context,
    pub materials: &'draft BTreeMap<MaterialId, u32>,
    pub meshes: &'draft BTreeMap<MeshId, Vec<u32>>,
    pub particles: &'draft BTreeMap<FeatureId, u32>,
    pub ribbons: &'draft BTreeMap<FeatureId, u32>,
    pub projectors: &'draft BTreeMap<FeatureId, u32>,
}
impl<E: Envelope> Writer<'_, '_, E> {
    pub fn property_is_set<T: Default + PartialEq>(&self, value: &surface::Property<T>) -> bool {
        value.initial != T::default()
            || value.reset != T::default()
            || value.control != surface::AnimationControl::default()
            || value
                .binding
                .as_ref()
                .and_then(|binding| self.ids.get(binding.target()))
                .is_some_and(|id| id.keyed)
    }
    pub fn material(&self, id: Option<MaterialId>) -> Result<u32, Error> {
        id.and_then(|id| self.materials.get(&id).copied())
            .ok_or_else(|| {
                Error::new(
                    "m3.particle.material",
                    "emitter requires an encoded material",
                )
            })
    }
    pub fn particle(&self, id: Option<FeatureId>) -> Result<i32, Error> {
        link(id, self.particles, "particle")
    }
    pub fn ribbon(&self, id: Option<FeatureId>) -> Result<i32, Error> {
        link(id, self.ribbons, "ribbon")
    }
    pub fn projector(&self, id: Option<FeatureId>) -> Result<i32, Error> {
        link(id, self.projectors, "projector")
    }
    pub fn sections(&mut self, meshes: &[MeshId]) -> Result<Ref<nr::targets::U32>, Error> {
        let mut rows = Vec::new();
        for mesh in meshes {
            let regions = self.meshes.get(mesh).ok_or_else(|| {
                Error::new(
                    "m3.particle.shape_sections",
                    "emission mesh is not present in the first native division",
                )
            })?;
            rows.extend(regions.iter().map(|value| nr::U32 { value: *value }));
        }
        Ok(append(self.document, &rows)?.cast())
    }
    pub fn spline(
        &mut self,
        points: &[surface::Property<[f32; 3]>],
    ) -> Result<Ref<nr::targets::AnimationValueVector3>, Error> {
        let rows = points
            .iter()
            .map(|value| nr::AnimationValueVector3 {
                value: animated(value, self.ids),
            })
            .collect::<Vec<_>>();
        Ok(append(self.document, &rows)?.cast())
    }
    pub fn models(
        &mut self,
        models: &[surface::ExternalModel<'_>],
    ) -> Result<Ref<nr::targets::StringReference>, Error> {
        let mut rows = Vec::new();
        for model in models {
            if model.hash.is_some() {
                self.context.unsupported(
                    "m3.particle.models.hash",
                    "native emitter resources declare paths without numeric identities",
                )?;
            }
            rows.push(nr::StringReference {
                value: string(self.document, &model.path)?,
            });
        }
        Ok(append(self.document, &rows)?.cast())
    }
}
fn link(id: Option<FeatureId>, map: &BTreeMap<FeatureId, u32>, kind: &str) -> Result<i32, Error> {
    match id {
        None => Ok(-1),
        Some(id) => map
            .get(&id)
            .copied()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| {
                Error::new(
                    format!("m3.particle.{kind}"),
                    "feature is absent from the corresponding native table",
                )
            }),
    }
}
