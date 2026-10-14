//! Transient import identities resolve field links into the checked graph.
use super::super::{
    bindings::{self, Bindings, Value},
    materials::{anim, color, fixed_color, read_error, string},
};
use crate::model::conversion::{Context, Error};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{
    profiles::m3 as surface, FeatureId, MaterialId, MeshId, OwnerId, Values,
};
use native::{Envelope, Ref};
pub(in crate::model::conversion::m3::logical) struct Reader<'view, 'data, 'draft, E: Envelope> {
    pub source: &'view native::Document<'data, E>,
    pub owner: FeatureId,
    pub particles: &'draft [FeatureId],
    pub ribbons: &'draft [FeatureId],
    pub projectors: &'draft [FeatureId],
    pub materials: &'draft [MaterialId],
    pub meshes: &'draft [MeshId],
    pub bindings: &'draft mut Bindings,
    pub context: &'draft mut Context,
}
impl<'view, E: Envelope> Reader<'view, '_, '_, E> {
    pub fn property<T: Value>(
        &mut self,
        path: &str,
        value: nr::AnimationValue<T>,
    ) -> surface::Property<T> {
        let mut result = anim(value);
        bindings::field(
            &mut result,
            OwnerId::Feature(self.owner),
            path,
            self.bindings,
        );
        result
    }
    pub fn color(
        &mut self,
        path: &str,
        value: nr::AnimationValue<[u8; 4]>,
    ) -> surface::Property<[f32; 4]> {
        let mut result = color(value);
        bindings::field(
            &mut result,
            OwnerId::Feature(self.owner),
            path,
            self.bindings,
        );
        result
    }
    pub fn fixed_color(
        &mut self,
        path: &str,
        value: nr::AnimationValue<nr::FixedBytes<4>>,
    ) -> surface::Property<[f32; 4]> {
        let mut result = fixed_color(value);
        bindings::field(
            &mut result,
            OwnerId::Feature(self.owner),
            path,
            self.bindings,
        );
        result
    }
    pub fn material(&self, index: u32) -> Result<MaterialId, Error> {
        self.materials
            .get(index as usize)
            .copied()
            .ok_or_else(|| Error::new("m3.particle.material", "emitter material exceeds table"))
    }
    pub fn particle(&self, index: i32) -> Result<Option<FeatureId>, Error> {
        link(index, self.particles, "particle")
    }
    pub fn ribbon(&self, index: i32) -> Result<Option<FeatureId>, Error> {
        link(index, self.ribbons, "ribbon")
    }
    pub fn projector(&self, index: i32) -> Result<Option<FeatureId>, Error> {
        link(index, self.projectors, "projector")
    }
    pub fn sections(&self, reference: Ref<nr::targets::U32>) -> Result<Vec<MeshId>, Error> {
        self.source
            .resolve::<nr::U32>(reference.cast())
            .map_err(read_error)?
            .iter()
            .map(|row| {
                let index = row.map_err(read_error)?.value;
                self.meshes.get(index as usize).copied().ok_or_else(|| {
                    Error::new(
                        "m3.particle.shape_sections",
                        "emission region exceeds first geometry division",
                    )
                })
            })
            .collect()
    }
    pub fn spline(
        &mut self,
        reference: Ref<nr::targets::AnimationValueVector3>,
    ) -> Result<Values<'view, surface::Property<[f32; 3]>>, Error> {
        let mut points = Vec::new();
        for (index, row) in self
            .source
            .resolve::<nr::AnimationValueVector3>(reference.cast())
            .map_err(read_error)?
            .iter()
            .enumerate()
        {
            points.push(self.property(
                &format!("particle.spline_points[{index}]"),
                row.map_err(read_error)?.value,
            ));
        }
        Ok(Values::owned(points))
    }
    pub fn models(
        &self,
        reference: Ref<nr::targets::StringReference>,
    ) -> Result<Vec<surface::ExternalModel<'view>>, Error> {
        self.source
            .resolve::<nr::StringReference>(reference.cast())
            .map_err(read_error)?
            .iter()
            .map(|row| {
                Ok(surface::ExternalModel {
                    path: string(self.source, row.map_err(read_error)?.value)?,
                    hash: None,
                })
            })
            .collect()
    }
}
fn link(index: i32, values: &[FeatureId], kind: &str) -> Result<Option<FeatureId>, Error> {
    if index == -1 {
        return Ok(None);
    }
    let index = usize::try_from(index).map_err(|_| {
        Error::new(
            format!("m3.particle.{kind}"),
            "negative link is not the native absent sentinel",
        )
    })?;
    values.get(index).copied().map(Some).ok_or_else(|| {
        Error::new(
            format!("m3.particle.{kind}"),
            "linked feature exceeds its native namespace",
        )
    })
}
